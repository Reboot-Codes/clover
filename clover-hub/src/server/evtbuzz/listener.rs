use log::{debug, error, info, warn};
use regex::Regex;
use tokio::sync::Mutex;
use url::Url;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use std::sync::Arc;
use std::time::SystemTime;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use warp::{Filter, http::StatusCode};
use crate::server::arbiter::models::{ApiKeyWithKey, UserWithId};
use crate::utils::{gen_cid_with_check, gen_ipc_message, iso8601};
use crate::server::evtbuzz::models::{Client, ClientWithId, CoreUserConfig, IPCMessageWithId, Session, Store};
use crate::server::evtbuzz::websockets::handle_ws_client;

// example error response
#[derive(Serialize, Debug)]
struct ApiErrorResult {
  detail: String,
}

// errors thrown by handlers and custom filters,
// such as `ensure_authentication` filter
#[derive(Error, Debug)]
enum ApiErrors {
  #[error("user not authorized")]
  NotAuthorized(String),
}

// ensure that warp`s Reject recognizes `ApiErrors`
impl warp::reject::Reject for ApiErrors {}

// generic errors handler for all api errors
// ensures unified error structure
async fn handle_rejection(err: warp::reject::Rejection) -> std::result::Result<impl warp::reply::Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
      code = StatusCode::NOT_FOUND;
      message = "Not found";
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
      code = StatusCode::BAD_REQUEST;
      message = "Invalid Body";
    } else if let Some(e) = err.find::<ApiErrors>() {
      match e {
        ApiErrors::NotAuthorized(_error_message) => {
          code = StatusCode::UNAUTHORIZED;
          message = "Action not authorized";
        }
      }
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
      code = StatusCode::METHOD_NOT_ALLOWED;
      message = "Method not allowed";
    } else {
      // We should have expected this... Just log and say its a 500
      error!("unhandled rejection: {:?}", err);
      code = StatusCode::INTERNAL_SERVER_ERROR;
      message = "Internal server error";
    }

    let json = warp::reply::json(&ApiErrorResult { detail: message.into() });

    Ok(warp::reply::with_status(json, code))
}

// middleware that looks for authorization header and validates it
async fn ensure_authentication(path: String, store: Arc<Arc<Store>>, auth_header: Option<String>) -> Result<(UserWithId, ApiKeyWithKey, ClientWithId, Session), warp::reject::Rejection> {
  let client_id = gen_cid_with_check(&store).await;
  let mut client = ClientWithId { api_key: "".to_string(), user_id: "".to_string(), id: client_id.clone(), active: true };
  store.clients.lock().await.insert(client_id.clone(), client.clone().into());

  info!("Client: {}, hit secure path: {}, attempting authentication...", client.id.clone(), path.clone());

  match auth_header {
    Some(header) => {
      debug!("got auth header, verifying: {}", header);
      let parts: Vec<&str> = header.split(" ").collect();
      let mut authenticated = false;
      let mut api_key_str = "".to_string();

      if parts.len() == 2 && parts[0] == "Token" {
        api_key_str = parts[1].to_string();
        debug!("parsed key: {}", api_key_str.clone());
        for registered_api_key in store.clone().api_keys.lock().await.clone().into_iter() {
          debug!("testing against: {}", registered_api_key.0.clone());
          if api_key_str == registered_api_key.0 {
            authenticated = true;
            break;
          }
        }
      }

      if authenticated {
        debug!("Running through client registration for api_key: {}", api_key_str.clone());
        let api_keys = store.clone().api_keys.clone();
        let api_keys_locked = api_keys.lock().await;
        let api_key = api_keys_locked.get(&api_key_str.clone()).unwrap().clone().to_api_key_with_key(api_key_str.clone());

        let user_id = api_key.clone().user_id;
        debug!("Registering as client: {}", client_id.clone());
        client = ClientWithId { api_key: api_key_str.clone(), user_id: user_id.clone(), id: client_id.clone(), active: true };
        store.clients.lock().await.insert(client_id.clone(), client.clone().into());
        
        let user = store.users.clone().lock().await.get(&user_id.clone()).unwrap().clone().to_user_with_id(user_id.clone());
        let session = Session { start_time: iso8601(&SystemTime::now()), end_time: "".to_string(), api_key: api_key.key.clone() };
        user.sessions.lock().await.insert(client_id.clone(), session.clone());

        debug!("Registered: {}!", client_id.clone());

        info!("Client: {}, authenticated as user: {}!", client_id.clone(), api_key.clone().user_id);
        return Ok((user.clone(), api_key.clone(), client.clone(), session.clone()));
      } else {
        warn!("Client: {}, attempted to connect with an invalid api key, disconnecting...", client.id.clone());
        client = ClientWithId { api_key: client.api_key, user_id: client.user_id, id: client.id, active: false };
        store.clients.lock().await.insert(client_id.clone(), client.clone().into());
        return Err(warp::reject::custom(ApiErrors::NotAuthorized(
          "api key not registered".to_string(),
        )));
      }
    },
    None => {
      warn!("Client: {}, attempted to connect without an api key, disconnecting...", client.id.clone());
      client = ClientWithId { api_key: client.api_key, user_id: client.user_id, id: client.id, active: false };
      store.clients.lock().await.insert(client_id.clone(), client.clone().into());
      Err(warp::reject::custom(ApiErrors::NotAuthorized(
        "no authorization header".to_string(),
      )))
    }
  }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ServerHealth {
  up_since: String
}

async fn handle_ipc_send(sender: &mpsc::UnboundedSender<IPCMessageWithId>, msg: IPCMessageWithId, user_config: &Arc<CoreUserConfig>, store: &Store) {
  let users_mutex = &store.users.to_owned();
  let users = users_mutex.lock().await;
  let user_conf = users.get(&user_config.id.clone()).expect(&format!("ERROR: Core user not found: {}", user_config.id.clone()));
  let keys_mutex = &store.api_keys.to_owned();
  let keys = keys_mutex.lock().await;
  let api_key_conf = keys.get(&user_config.api_key.clone()).expect(&format!("ERROR: Core user api_key not found: {}", user_config.api_key.clone()));
  let mut event_sent = false;

  for allowed_event_regex in api_key_conf.allowed_events_from.clone() {
    match Regex::new(&allowed_event_regex.clone()) {
      Ok(regex) => {
        if regex.is_match(&msg.kind.clone()) {
          match sender.send(msg.clone()) {
            Ok(_) => {
              event_sent = true;
            },
            Err(e) => {
              error!("Core user: {}, IPC channel: {}, Failed to send message: {{ \"author\": \"{}\", \"kind\": \"{}\", \"message\": \"{}\" }}, due to:\n{}", user_config.id.clone(), user_conf.user_type.clone(), msg.author.clone(), msg.kind.clone(), msg.message.clone(), e);
            }
          };
        }
      },
      Err(e) => {
        error!("Core user: {}, api key's \"allowed events from\", regex: {}, is invalid! Regex Error: {}", user_config.id.clone(), allowed_event_regex.clone(), e);
      }
    }
  }

  if !event_sent {
    debug!("Core user: {}, event \"{}\" not sent.", user_config.id, msg.kind.clone());
  }
}

pub async fn evtbuzz_listener(
  port: u16, 
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>,
  mut arbiter_ipc: (&CoreUserConfig, UnboundedReceiver<IPCMessageWithId>),
  mut renderer_ipc: (&CoreUserConfig, UnboundedReceiver<IPCMessageWithId>),
  mut modman_ipc: (&CoreUserConfig, UnboundedReceiver<IPCMessageWithId>),
  mut inference_engine_ipc: (&CoreUserConfig, UnboundedReceiver<IPCMessageWithId>),
  mut appd_ipc: (&CoreUserConfig, UnboundedReceiver<IPCMessageWithId>),
  evtbuzz_user_config: Arc<CoreUserConfig>
) {
  info!("Starting EvtBuzz on port: {}...", port);
  
  let clients_tx: Arc<Mutex<HashMap<String, UnboundedSender<IPCMessageWithId>>>> = Arc::new(Mutex::new(HashMap::new()));

  let arbiter_cfg = arbiter_ipc.0;
  let renderer_cfg = renderer_ipc.0;
  let modman_cfg = modman_ipc.0;
  let inference_engine_cfg = inference_engine_ipc.0;
  let appd_cfg = appd_ipc.0;
  for client in vec![
    arbiter_cfg,
    renderer_cfg,
    modman_cfg,
    inference_engine_cfg,
    appd_cfg
  ] {
    let cid = gen_cid_with_check(&store).await;
    store.clients.lock().await.insert(
      cid, 
      Client { 
        api_key: client.api_key.clone(), 
        user_id: client.id.clone(),
        active: true 
      }
    );
  }

  let (from_client_tx, mut from_client_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();

  let filter_to_clients_tx = Arc::new(clients_tx.clone());
  let to_clients_tx_filter = warp::any().map(move || filter_to_clients_tx.clone());

  let filter_from_clients_tx = Arc::new(from_client_tx.clone());
  let from_clients_tx_filter = warp::any().map(move || filter_from_clients_tx.clone());

  let filter_store = Arc::new(store.clone());
  let store_filter = warp::any().map(move || filter_store.clone());

  let start_up_time = iso8601(&SystemTime::now());
  let health_check_path = warp::path("health-check")
    .map(move || {
      let current_health = ServerHealth{up_since: start_up_time.clone()};
      warp::reply::json(&current_health)
    });

  let ws_path = warp::path("ws")
    .and(warp::any().map(|| "/ws".to_string()))
    .and(store_filter.clone())
    .and(warp::header::optional::<String>("Authorization"))
    .and_then(ensure_authentication)
    .and(warp::ws())
    .and(store_filter.clone())
    .and(to_clients_tx_filter.clone())
    .and(from_clients_tx_filter.clone())
    .and_then(handle_ws_client);

  let routes = health_check_path
    .or(ws_path)
    .with(warp::cors().allow_any_origin())
    .recover(handle_rejection);

  // TODO: Add control REST API for start up and shut down.

  // TODO: Start creating GQL API endpoint.
  
  let server_port = Arc::new(port.clone());
  let http_handle = tokio::task::spawn(async move {
    warp::serve(routes)
      // TODO: Add option for listening address.
      .try_bind(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), *server_port)).await;
  });

  let ipc_dispatch_store = Arc::new(store.clone());
  let ipc_dispatch_clients_tx = Arc::new(clients_tx.clone());
  let ipc_dispatch_user_config = Arc::new(evtbuzz_user_config.clone());
  let ipc_dispatch_handle = tokio::task::spawn(async move {
    while let Some(message) = ipc_rx.recv().await {
      debug!("Got message type: {}, with data:\n  {}", message.kind.clone(), message.message.clone());
      for client in ipc_dispatch_store.clients.lock().await.clone().into_iter() {
        let client_id = Arc::new(client.0);
        let mutex = &ipc_dispatch_clients_tx.to_owned();
        let client_senders = mutex.lock();
        let hash_map = &client_senders.await;
        let mut message_sent = false;

        match hash_map.get(&client_id.to_string()) {
          Some(client_sender) => {
            if client.1.active {
              match ipc_dispatch_store.clone().api_keys.lock().await.get(&client.1.api_key) {
                Some(api_key) => {
                  for allowed_event_regex in &api_key.allowed_events_to {
                    match Regex::new(&allowed_event_regex) {
                      Ok(regex) => {
                        if regex.is_match(&allowed_event_regex) && !(message.author.clone().split("?client=").collect::<Vec<_>>()[1] == *client_id.clone()) {
                          debug!("Sending event: \"{}\", to client: {}...", message.kind.clone(), client_id.clone());
                          match client_sender.send(message.clone()) {
                            Ok(_) => {
                              message_sent = true;
                            },
                            Err(e) => {
                              error!("Failed to send message to client: {}, due to:\n{}", client_id.clone(), e);
                            }
                          };
                          
                          break;
                        }
                      },
                      Err(e) => {
                        error!("Message: \"{}\", failed, allowed event regular expression for client: {}, errored with: {}", message.kind, client_id.clone(), e);
                      }
                    }
                  }

                  if (!message_sent) && api_key.echo && (message.author.clone().split("?client=").collect::<Vec<_>>()[1] == *client_id.clone()) {
                    debug!("Echoing event: \"{}\", to client: {}...", message.kind.clone(), client_id.clone());
                    match client_sender.send(message.clone()) {
                      Ok(_) => {
                        message_sent = true;
                      },
                      Err(e) => {
                        error!("Failed to send message to client: {}, due to:\n{}", client_id.clone(), e);
                      }

                    };
                  }
                },
                None => {
                  error!("DANGER! Client: {}, had API key removed from store without closing connection on removal, THIS IS BAD; please report this! Closing connection...", client_id.clone());

                  let kind = Url::parse("clover://hub/server/listener/clients/unauthorize")
                    .unwrap()
                    .query_pairs_mut()
                    .append_pair("id", &client_id.clone())
                    .finish()
                    .to_string();

                  let generated_message = gen_ipc_message(
                    &ipc_dispatch_store.clone(),
                    &ipc_dispatch_user_config.clone(), 
                    kind, 
                    "api key removed from store".to_string()
                  ).await;
                  ipc_dispatch_store.messages.lock().await.insert(generated_message.id.clone(), generated_message.clone().into());

                  let _ = client_sender.send(generated_message.clone());
                }
              }
            }
          },
          None => {
            error!("Client: {}, does not exist in the client map!", client_id.clone());
          }
        }

        if !message_sent { debug!("Message: \"{}\", not sent to client: {}", message.kind.clone(), client_id.clone()); }
      }
    }
  });

  // IPC Handle for data from WS clients.
  let ipc_receive_handle = tokio::task::spawn(async move {
    while let Some(msg) = from_client_rx.recv().await {
      debug!("Got message: {{ \"author\": \"{}\", \"kind\": \"{}\", \"message\": \"{}\" }}", msg.author.clone(), msg.kind.clone(), msg.message.clone());
      match ipc_tx.send(msg.clone()) {
        Ok(_) => {},
        Err(e) => {
          error!("Failed to send message: {{ \"author\": \"{}\", \"kind\": \"{}\", \"message\": \"{}\" }}, due to:\n{}", msg.author.clone(), msg.kind.clone(), msg.message.clone(), e);
        }
      };
    }
  });

  // Internal IPC Handles
  let from_arbiter_cfg = Arc::new(arbiter_cfg.clone());
  let from_arbiter_store = Arc::new(store.clone());
  let from_arbiter_tx = Arc::new(from_client_tx.clone());
  let from_arbiter_handle = tokio::task::spawn(async move {
    while let Some(msg) = arbiter_ipc.1.recv().await {
      handle_ipc_send(&from_arbiter_tx, msg, &from_arbiter_cfg.clone(), &from_arbiter_store.clone()).await;
    }
  });

  let from_renderer_cfg = Arc::new(renderer_cfg.clone());
  let from_renderer_store = Arc::new(store.clone());
  let from_renderer_tx = Arc::new(from_client_tx.clone());
  let from_renderer_handle = tokio::task::spawn(async move {
    while let Some(msg) = renderer_ipc.1.recv().await {
      handle_ipc_send(&from_renderer_tx, msg, &from_renderer_cfg.clone(), &from_renderer_store.clone()).await;
    }
  });

  let from_modman_cfg = Arc::new(modman_cfg.clone());
  let from_modman_store = Arc::new(store.clone());
  let from_modman_tx = Arc::new(from_client_tx.clone());
  let from_modman_handle = tokio::task::spawn(async move {
    while let Some(msg) = modman_ipc.1.recv().await {
      handle_ipc_send(&from_modman_tx, msg, &from_modman_cfg.clone(), &from_modman_store.clone()).await;
    }
  });

  let from_inference_engine_cfg = Arc::new(inference_engine_cfg.clone());
  let from_inference_engine_store = Arc::new(store.clone());
  let from_inference_engine_tx = Arc::new(from_client_tx.clone());
  let from_inference_engine_handle = tokio::task::spawn(async move {
    while let Some(msg) = inference_engine_ipc.1.recv().await {
      handle_ipc_send(&from_inference_engine_tx, msg, &from_inference_engine_cfg.clone(), &from_inference_engine_store.clone()).await;
    }
  });

  let from_appd_cfg = Arc::new(appd_cfg.clone());
  let from_appd_store = Arc::new(store.clone());
  let from_appd_tx = Arc::new(from_client_tx.clone());
  let from_appd_handle = tokio::task::spawn(async move {
    while let Some(msg) = appd_ipc.1.recv().await {
      handle_ipc_send(&from_appd_tx, msg, &from_appd_cfg.clone(), &from_appd_store.clone()).await;
    }
  });

  futures::future::join_all(vec![
    http_handle, 
    ipc_dispatch_handle, 
    ipc_receive_handle, 
    from_arbiter_handle,
    from_renderer_handle,
    from_modman_handle,
    from_inference_engine_handle,
    from_appd_handle
  ]).await;

  info!("Shutting down EvtBuzz...");
  // TODO: Clean up registered sessions when server is shutting down.
}
