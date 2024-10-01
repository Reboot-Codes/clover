use log::{debug, info};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use url::Url;
use std::sync::Arc;
use crate::server::evtbuzz::models::{IPCMessageWithId, Store, CoreUserConfig};

pub async fn renderer_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_token: CancellationToken
) {
  info!("Starting Renderer...");
  // TODO: Setup EGL ctx for each display we're handling.
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
      info!("Cleaning up displays...");
      // TODO: Clean up registered displays when server is shutting down.

      std::mem::drop(store);
    }
  }

  info!("Renderer has stopped!");
}
