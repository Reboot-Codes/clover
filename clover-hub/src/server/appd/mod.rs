use std::sync::Arc;
use log::{debug, info};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use url::Url;
use super::evtbuzz::models::{IPCMessageWithId, CoreUserConfig, Store};

// TODO: Create application manifest schema/models

pub async fn appd_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_token: CancellationToken
) {
  info!("Starting AppDaemon...");
  // TODO: Add docker crate to manage applications.

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
          if kind.host().unwrap() == url::Host::Domain("appd.clover.reboot-codes.com") {
            debug!("Processing: {}", msg.kind.clone());
          }
        }
      } => {}
    }
  });

  let cleanup_token = cancellation_token.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      info!("Cleaning up applications...");
      // TODO: Clean up registered applications when server is shutting down.

      cleanup_token.cancel();
    }
  }

  info!("AppD has stopped!");
}
