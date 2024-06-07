use log::{debug, error, info, warn};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use url::Url;
use std::collections::HashMap;
use tokio::sync::mpsc::{self, UnboundedSender};
use std::sync::Arc;
use std::time::SystemTime;
use futures::{SinkExt, StreamExt};
use warp::filters::ws::{Message, WebSocket};
use crate::utils::iso8601;
use crate::server::models::{ApiKeyWithKey, ClientWithId, Client, IPCMessage, Session, Store, UserWithId};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct WsIn {
  pub kind: String,
  pub message: String,
}

pub async fn handle_ws_client(auth: (UserWithId, ApiKeyWithKey, ClientWithId, Session), ws: warp::ws::Ws, store: Arc<Arc<Store>>, to_clients_tx: Arc<Arc<Mutex<HashMap<String, UnboundedSender<IPCMessage>>>>>, from_clients_tx: Arc<UnboundedSender<IPCMessage>>) -> Result<impl warp::Reply, warp::Rejection> {
  let user = auth.0.clone();
  let api_key = auth.1.clone();
  let client = auth.2.clone();
  let session = auth.3.clone();

  info!("Upgrading client: {}, to websocket connection...", client.id.clone());

  let ws_client = Arc::new(client.clone());
  Ok(ws.on_upgrade(move |websocket: WebSocket| async move {
    info!("Upgraded client: {}, to websocket connection!", ws_client.id.clone());

    let (mut sender, mut receiver) = websocket.split();
    let (to_client_tx, mut to_client_rx) = mpsc::unbounded_channel::<IPCMessage>();
    let mut deauthed = false;

    to_clients_tx.lock().await.insert(ws_client.id.clone(), to_client_tx);

    let recv_client = Arc::new(client.clone());
    let recv_handle = tokio::task::spawn(async move {
      while !deauthed && let Some(body) = receiver.next().await {
        match body {
          Ok(msg) => {
            // Skip any non-Text messages...
            let message = if let Ok(s) = msg.to_str() {
              s
            } else {
              info!("ping-pong");
              return;
            };

            match serde_json::from_str::<WsIn>(message) {
              Ok(msg) => {
                debug!("Client: {}, send message: {{ \"kind\": \"{}\", \"message\": \"{}\" }}...", ws_client.id.clone(), msg.kind.clone(), msg.message.clone());
                
                let mut allowed_to_send = false;
                for allowed_send_pattern in api_key.allowed_events_from.clone() {
                  match Regex::new(&allowed_send_pattern) {
                    Ok(pattern) => {
                      if pattern.is_match(&msg.kind.clone()) {
                        allowed_to_send = true;
                        break;
                      }
                    },
                    Err(e) => {
                      warn!("Allowed send from pattern: {}, is not valid, due to:\n{}", allowed_send_pattern.clone(), e);
                    }
                  }
                }

                if allowed_to_send {
                  match from_clients_tx.send(IPCMessage { author: format!("ws:{}?client={}", api_key.user_id.clone(), ws_client.id.clone()), kind: msg.kind.clone(), message: msg.message.clone() }) {
                    Ok(_) => {
                      debug!("Client: {}, successfully sent message: {{ \"kind\": \"{}\", \"message\": \"{}\" }}, over to dispatch IPC thread!", recv_client.id.clone(), msg.kind.clone(), msg.message.clone());
                    },
                    Err(e) => {
                      error!("Client: {}, failed to send message: {{ \"kind\": \"{}\", \"message\": \"{}\" }}, over to dispatch IPC thread, due to:\n  {}", recv_client.id.clone(), msg.kind.clone(), msg.message.clone(), e);
                    }
                  };
                } else {
                  warn!("Client: {}, attempted to send message of ", recv_client.id.clone());
                }
              },
              Err(e) => {
                warn!("Client: {}, error reading message: \"{}\", due to:\n  {}", recv_client.id.clone(), message, e);
              }
            };
          },
          Err(e) => {
            error!("error reading message on websocket: {}", e);
            // TODO: send error?
          }
        };
      }

      // Client closed connection.
      (#[allow(unused_assignments)]
      deauthed) = true;
    });

    let send_client = Arc::new(client.clone());
    let send_handle = tokio::task::spawn(async move {
      while let Some(msg) = to_client_rx.recv().await {
        if msg.kind == Url::parse("clover://hub/server/listener/clients/unauthorize")
        .unwrap()
        .query_pairs_mut()
        .append_pair("id", send_client.id.clone().as_str())
        .finish()
        .to_string() {
          match sender.close().await {
            Ok(_) => {},
            Err(e) => {
              error!("Client: {}, failed to close connection due to:\n  {}", send_client.id.clone(), e);
            }
          };

          (#[allow(unused_assignments)]
          deauthed) = true;
          break;
        } else if deauthed {
          break;
        } else {
          let response = serde_json::to_string(&IPCMessage {
            author: msg.author.clone(),
            kind: msg.kind.clone(),
            message: msg.message.clone(),
          })
          .unwrap();
          sender.send(Message::text(response)).await.unwrap();
        }
      }
    });

    let clean_up_client = Arc::new(client.clone());
    let clean_up_handle = tokio::task::spawn(async move {
      while !deauthed {if deauthed { break; }}

      info!("Client: {}, disconnected, cleaning up...", clean_up_client.id.clone());
      debug!("Ending session for: {}...", clean_up_client.id.clone());
      store.users.clone().lock().await.get(&user.id.clone()).unwrap().sessions.lock().await.insert(clean_up_client.id.clone(), Session { start_time: session.start_time.clone(), end_time: iso8601(&SystemTime::now()), api_key: api_key.key.clone() });
      debug!("Deactivating client: {}...", clean_up_client.id.clone());
      store.clients.clone().lock().await.insert(clean_up_client.id.clone(), Client { api_key: api_key.key.clone(), user_id: user.id.clone(), active: false });
    });

    futures::future::join_all(vec![recv_handle, send_handle, clean_up_handle]).await;
  }))
}
