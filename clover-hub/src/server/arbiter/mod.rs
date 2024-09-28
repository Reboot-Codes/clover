pub mod models;

use std::sync::Arc;
use log::{debug, info};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use url::Url;
use super::evtbuzz::models::{IPCMessageWithId, CoreUserConfig, Store};

pub async fn arbiter_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_token: CancellationToken
) {
  info!("Starting Arbiter...");

  let ipc_recv_token = cancellation_token.clone();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Some(msg) = ipc_rx.recv().await {
          let kind = Url::parse(&msg.kind.clone()).unwrap();

          // Verify that we care about this event.
          if kind.host().unwrap() == url::Host::Domain("arbiter.clover.reboot-codes.com") {
            debug!("Processing: {}", msg.kind.clone());
          }
        }
      } => {}
    }
  });

  let cleanup_token = cancellation_token.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      info!("Cleaning up users...");
      // TODO: Clean up registered users when server is shutting down.

      cleanup_token.cancel();
    }
  }

  tokio::select! {_ = futures::future::join_all(vec![ipc_recv_handle]) => {
    info!("Arbiter has stopped!");
  }}
}
