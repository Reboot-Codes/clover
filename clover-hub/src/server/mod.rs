pub mod evtbuzz;
pub mod arbiter;
pub mod renderer;
pub mod modman;
pub mod inference_engine;
pub mod appd;

use std::env;
use std::sync::Arc;
use appd::appd_main;
use evtbuzz::models::CoreUserConfig;
use regex::Regex;
use tokio::sync::mpsc;
use log::{debug, error, info};
use evtbuzz::models::Store;
use evtbuzz::models::IPCMessageWithId;
use evtbuzz::listener::evtbuzz_listener;
use arbiter::arbiter_main;
use renderer::renderer_main;
use modman::modman_main;
use inference_engine::inference_engine_main;
use tokio_util::sync::CancellationToken;

async fn handle_ipc_send(sender: mpsc::UnboundedSender<IPCMessageWithId>, msg: IPCMessageWithId, user_config: &CoreUserConfig, store: &Store) {
  let users_mutex = &store.users.to_owned();
  let users = users_mutex.lock().await;
  let user_conf = users.get(&user_config.id.clone()).expect(&format!("ERROR: Core user not found: {}", user_config.id.clone()));
  let keys_mutex = &store.api_keys.to_owned();
  let keys = keys_mutex.lock().await;
  let api_key_conf = keys.get(&user_config.api_key.clone()).expect(&format!("ERROR: Core user api_key not found: {}", user_config.api_key.clone()));
  let mut event_sent = false;

  for allowed_event_regex in api_key_conf.allowed_events_to.clone() {
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
        error!("Core user: {}, api key's \"allowed events to\", regex: {}, is invalid! Regex Error: {}", user_config.id.clone(), allowed_event_regex.clone(), e);
      }
    }
  }

  if !event_sent {
    debug!("Core user: {}, event \"{}\" not sent.", user_config.id, msg.kind.clone());
  }
}

pub async fn server_main(port: u16, cancellation_token: CancellationToken) {
  info!("Starting CloverHub...");

  let (
    store, 
    master_user_config, 
    (
      evtbuzz_user_config, 
      arbiter_user_config, 
      renderer_user_config,
      appd_user_config,
      modman_user_config,
      inference_engine_user_config
    )
  ) = Store::new_configured_store().await;
  let master_user_id = master_user_config.id.clone();
  if env::var("CLOVER_MASTER_PRINT").unwrap_or("false".to_string()) == "true".to_string() { 
    debug!("Master user id: {}, primary api key: {}", master_user_id.clone(), master_user_config.api_key.clone()) 
  };

  // Start Arbiter
  let (arbiter_from_tx, arbiter_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (arbiter_to_tx, arbiter_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let arbiter_store = Arc::new(store.clone());
  let arbiter_uca = Arc::new(arbiter_user_config.clone());
  let arbiter_token = CancellationToken::new();
  let arbiter_token_clone = arbiter_token.child_token();
  let arbiter_handle = tokio::task::spawn(async move {
    arbiter_main(arbiter_from_tx, arbiter_to_rx, arbiter_store.clone(), arbiter_uca.clone(), arbiter_token_clone).await;
  });

  // Start Renderer
  let (renderer_from_tx, renderer_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (renderer_to_tx, renderer_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let renderer_store = Arc::new(store.clone());
  let renderer_uca = Arc::new(renderer_user_config.clone());
  let renderer_token = CancellationToken::new();
  let renderer_token_clone  = renderer_token.child_token();
  let renderer_handle = tokio::task::spawn(async move {
    renderer_main(renderer_from_tx, renderer_to_rx, renderer_store.clone(), renderer_uca.clone(), renderer_token_clone).await;
  });

  // Start ModMan
  let (modman_from_tx, modman_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (modman_to_tx, modman_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let modman_store = Arc::new(store.clone());
  let modman_uca = Arc::new(modman_user_config.clone());
  let modman_token = CancellationToken::new();
  let modman_token_clone = modman_token.child_token();
  let modman_handle = tokio::task::spawn(async move {
    modman_main(modman_from_tx, modman_to_rx, modman_store.clone(), modman_uca.clone(), modman_token_clone).await;
  });

  // Start InferenceEngine
  let (inference_engine_from_tx, inference_engine_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (inference_engine_to_tx, inference_engine_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let inference_engine_store = Arc::new(store.clone());
  let inference_engine_uca = Arc::new(inference_engine_user_config.clone());
  let inference_engine_token = CancellationToken::new();
  let inference_engine_token_clone = inference_engine_token.child_token();
  let inference_engine_handle = tokio::task::spawn(async move {
    inference_engine_main(
      inference_engine_from_tx, 
      inference_engine_to_rx, 
      inference_engine_store.clone(), 
      inference_engine_uca.clone(), 
      inference_engine_token_clone
    ).await;
  });

  // Start AppDaemon
  let (appd_from_tx, appd_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (appd_to_tx, appd_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let appd_store = Arc::new(store.clone());
  let appd_uca = Arc::new(appd_user_config.clone());
  let appd_token = CancellationToken::new();
  let appd_token_clone = appd_token.child_token();
  let appd_handle = tokio::task::spawn(async move {
    appd_main(appd_from_tx, appd_to_rx, appd_store.clone(), appd_uca.clone(), appd_token_clone).await;
  });

  let evtbuzz_port = Arc::new(port);
  let evtbuzz_store = Arc::new(store.clone());
  let evtbuzz_uca = Arc::new(evtbuzz_user_config.clone());
  let evtbuzz_arbiter_user_config_arc = Arc::new(arbiter_user_config.clone());
  let evtbuzz_renderer_user_config_arc = Arc::new(renderer_user_config.clone());
  let evtbuzz_modman_user_config_arc = Arc::new(modman_user_config.clone());
  let evtbuzz_inference_engine_user_config_arc = Arc::new(inference_engine_user_config.clone());
  let evtbuzz_appd_user_config_arc = Arc::new(appd_user_config.clone());
  let evtbuzz_token = CancellationToken::new();
  let evtbuzz_token_clone = evtbuzz_token.child_token();
  let evtbuzz_handle = tokio::task::spawn(async move {
    evtbuzz_listener(
      *evtbuzz_port.to_owned(), 
      evtbuzz_store.clone(), 
      (&evtbuzz_arbiter_user_config_arc.clone(), arbiter_from_rx, arbiter_to_tx),
      (&evtbuzz_renderer_user_config_arc.clone(), renderer_from_rx, renderer_to_tx), 
      (&evtbuzz_modman_user_config_arc.clone(), modman_from_rx, modman_to_tx),
      (&evtbuzz_inference_engine_user_config_arc.clone(), inference_engine_from_rx, inference_engine_to_tx),
      (&evtbuzz_appd_user_config_arc.clone(), appd_from_rx, appd_to_tx),
      evtbuzz_token_clone,
      evtbuzz_uca.clone()
    ).await;
  });

  let cleanup_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        debug!("Shutting down AppD");
        appd_token.cancel();
        tokio::select! {
          _ = appd_token.cancelled() => {
            debug!("Shutting down Inference Engine");
            inference_engine_token.cancel();
            tokio::select! {
              _ = inference_engine_token.cancelled() => {
                debug!("Shutting down ModMan");
                modman_token.cancel();
                tokio::select! {
                  _ = modman_token.cancelled() => {
                    debug!("Shutting down Renderer");
                    renderer_token.cancel();
                    tokio::select! {
                      _ = renderer_token.cancelled() => {
                        debug!("Shutting down Arbiter");
                        arbiter_token.cancel();
                        tokio::select! {
                          _ = arbiter_token.cancelled() => {
                            debug!("Shutting down EvtBuzz");
                            evtbuzz_token.cancel();
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  });

  tokio::select! {_ = futures::future::join_all(vec![
    cleanup_handle,
    evtbuzz_handle,
    arbiter_handle, 
    renderer_handle,
    modman_handle,
    inference_engine_handle,
    appd_handle 
  ]) => {
    info!("CloverHub Server has exited.");
  }}
}
