mod evtbuzz;
mod arbiter;
mod renderer;
mod modman;
mod inference_engine;
mod appd;

use std::sync::Arc;
use appd::appd_main;
use tokio::sync::mpsc;
use log::{debug, error, info};
use evtbuzz::models::Store;
use uuid::Uuid;
use evtbuzz::models::IPCMessageWithId;
use evtbuzz::listener::server_listener;
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

  let master_user_id = Uuid::new_v4().to_string();
  let store = Store::new_with_set_master_user(master_user_id.clone()).await;
  debug!("Master user id: {}, primary api key: {}", master_user_id.clone(), store.users.lock().await.get(&master_user_id.clone()).unwrap().api_keys.get(0).unwrap());

  // Start EVTBuzz
  // TODO: Rename to EVTBuzz
  let (listener_from_tx, mut listener_from_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let (listener_to_tx, listener_to_rx) = mpsc::unbounded_channel::<IPCMessageWithId>();
  let listener_port = Arc::new(port);
  let listener_store = Arc::new(store.clone());
  let listener_handler = tokio::task::spawn(async move { 
    server_listener(*listener_port.to_owned(), listener_from_tx, listener_to_rx, listener_store.clone()).await;
  });

  let ipc_from_listener_dispatch = tokio::task::spawn(async move {
    while let Some(msg) = listener_from_rx.recv().await {
      // TODO: Add all IPC channels to this list.
      handle_ipc_send(&listener_to_tx, msg, "clover::server::listener".to_string());
    }
  });

  // Start Arbiter
  let arbiter_handler = tokio::task::spawn(async move {
    arbiter_main().await;
  });

  // Start Renderer
  let renderer_handler = tokio::task::spawn(async move {
    renderer_main().await;
  });

  // Start ModMan
  let modman_handler = tokio::task::spawn(async move {
    modman_main().await;
  });

  // Start InferenceEngine
  let inference_engine_handler = tokio::task::spawn(async move {
    inference_engine_main().await;
  });

  // Start AppDaemon
  let appd_handler = tokio::task::spawn(async move {
    appd_main().await;
  });

  futures::future::join_all(vec![
    listener_handler, 
    ipc_from_listener_dispatch, 
    arbiter_handler, 
    renderer_handler, 
    modman_handler, 
    inference_engine_handler, 
    appd_handler
  ]).await;
}
