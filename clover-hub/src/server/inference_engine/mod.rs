use std::sync::Arc;
use log::{debug, info};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use url::Url;
use crate::utils::send_ipc_message;

use super::evtbuzz::models::{IPCMessageWithId, CoreUserConfig, Store};

pub async fn inference_engine_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_tokens: (CancellationToken, CancellationToken)
) {
  info!("Starting Inference Engine...");

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user_config.clone());
  let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
  cancellation_tokens.0.run_until_cancelled(async move {
    let _ = send_ipc_message(
      &init_store, 
      &init_user, 
      Arc::new(init_from_tx), 
      "clover://inference-engine.clover.reboot-codes.com/status".to_string(), 
      "finished-init".to_string()
    ).await;
  }).await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Some(msg) = ipc_rx.recv().await {
          let kind = Url::parse(&msg.kind.clone()).unwrap();

          // Verify that we care about this event.
          if kind.host().unwrap() == url::Host::Domain("inference-engine.clover.reboot-codes.com") {
            debug!("Processing: {}", msg.kind.clone());
          }
        }
      } => {}
    }
  });

  let ipc_trans_token = cancellation_tokens.0.clone();
  let ipc_trans_tx = Arc::new(ipc_tx.clone());
  let ipc_trans_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = async move {
        while let Some(msg) = init_from_rx.recv().await {
          match ipc_trans_tx.send(msg) {
            Ok(_) => {},
            Err(_) => {
              debug!("Failed to send message to IPC bus!");
            }
          }
        }
      } => {},
      _ = ipc_trans_token.cancelled() => {
        debug!("ipc_trans exited");
      }
    }
  });

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_trans_handle.abort();

      info!("Cleaning up networks...");
      // TODO: Clean up registered networks when server is shutting down.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Inference Engine has stopped!");
}
