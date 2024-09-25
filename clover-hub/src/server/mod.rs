pub mod evtbuzz;
pub mod arbiter;
pub mod renderer;
pub mod modman;
pub mod inference_engine;
pub mod appd;

use std::sync::Arc;
use appd::appd_main;
use tokio::sync::mpsc;
use log::{debug, error, info};
use evtbuzz::models::Store;
use evtbuzz::models::IPCMessageWithId;
use evtbuzz::listener::evtbuzz_listener;
use arbiter::arbiter_main;
use renderer::renderer_main;
use modman::modman_main;
use inference_engine::inference_engine_main;

fn handle_ipc_send(sender: &mpsc::UnboundedSender<IPCMessageWithId>, msg: IPCMessageWithId, target_name: String) {
  match sender.send(msg.clone()) {
    Ok(_) => {},
    Err(e) => {
      error!("Failed to send message: {{ \"author\": \"{}\", \"kind\": \"{}\", \"message\": \"{}\" }}, to IPC channel: {}, due to:\n{}", msg.author.clone(), msg.kind.clone(), msg.message.clone(), target_name, e);
    }
  };
}

pub async fn server_main(port: u16) {
  info!("Running Backend Server Threads...");

  // TODO: Pass creds to each core service.
  let (store, master_user_config, core_users_config) = Store::new_configured_store().await;
  let master_user_id = master_user_config.id.clone();
  debug!("Master user id: {}, primary api key: {}", master_user_id.clone(), master_user_config.api_key.clone());

  let (evtbuzz_from_tx, mut evtbuzz_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (evtbuzz_to_tx, evtbuzz_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let evtbuzz_port = Arc::new(port);
  let evtbuzz_store = Arc::new(store.clone());
  let evtbuzz_handler = tokio::task::spawn(async move { 
    evtbuzz_listener(*evtbuzz_port.to_owned(), evtbuzz_from_tx, evtbuzz_to_rx, evtbuzz_store.clone()).await;
  });

  // Start Arbiter
  let (arbiter_from_tx, mut arbiter_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (arbiter_to_tx, arbiter_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let arbiter_store = Arc::new(store.clone());
  let arbiter_handler = tokio::task::spawn(async move {
    arbiter_main(arbiter_from_tx, arbiter_from_rx, arbiter_store.clone()).await;
  });

  // Start Renderer
  let (renderer_from_tx, mut renderer_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (renderer_to_tx, renderer_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let renderer_store = Arc::new(store.clone());
  let renderer_handler = tokio::task::spawn(async move {
    renderer_main(renderer_from_tx, renderer_to_rx, renderer_store.clone()).await;
  });

  // Start ModMan
  let (modman_from_tx, mut modman_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (modman_to_tx, modman_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let modman_store = Arc::new(store.clone());
  let modman_handler = tokio::task::spawn(async move {
    modman_main(modman_from_tx, modman_to_rx, modman_store.clone()).await;
  });

  // Start InferenceEngine
  let (inference_engine_from_tx, mut inference_engine_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (inference_engine_to_tx, inference_engine_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let inference_engine_store = Arc::new(store.clone());
  let inference_engine_handler = tokio::task::spawn(async move {
    inference_engine_main(inference_engine_from_tx, inference_engine_to_rx, inference_engine_store.clone()).await;
  });

  // Start AppDaemon
  let (appd_from_tx, mut appd_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (appd_to_tx, appd_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let appd_store = Arc::new(store.clone());
  let appd_handler = tokio::task::spawn(async move {
    appd_main(appd_from_tx, appd_to_rx, appd_store.clone()).await;
  });

  let ipc_from_listener_dispatch = tokio::task::spawn(async move {
    while let Some(msg) = evtbuzz_from_rx.recv().await {
      // TODO: Add all IPC channels to this list.
      handle_ipc_send(&evtbuzz_to_tx, msg.clone(), "clover::server::evtbuzz".to_string());
      handle_ipc_send(&renderer_to_tx, msg.clone(), "clover::server::renderer".to_string());
    }
  });

  // TODO: Figure out routing between all threads... send all messages to EvtBuzz and have it send events out?

  futures::future::join_all(vec![
    evtbuzz_handler, 
    ipc_from_listener_dispatch, 
    arbiter_handler, 
    renderer_handler, 
    modman_handler, 
    inference_engine_handler, 
    appd_handler
  ]).await;
}
