mod listener;
mod models;
mod websockets;

use std::sync::Arc;
use tokio::sync::mpsc;
use log::{debug, error, info};
use models::Store;
use uuid::Uuid;
use crate::server::models::IPCMessageWithId;
use crate::server::listener::server_listener;

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
  debug!("Master user id: {}, primary api key {}", master_user_id.clone(), store.users.lock().await.get(&master_user_id.clone()).unwrap().api_keys.get(0).unwrap());

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

  futures::future::join_all(vec![listener_handler, ipc_from_listener_dispatch]).await;
}
