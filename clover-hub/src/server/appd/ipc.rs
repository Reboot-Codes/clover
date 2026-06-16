use std::sync::Arc;

use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  instrument,
};

#[instrument(skip(ipc_token, ipc_session))]
pub async fn handle_ipc(ipc_token: CancellationToken, ipc_session: Arc<zenoh::Session>) {
  let subscriber = ipc_session
    .declare_subscriber("com/reboot-codes/clover/server/appdaemon/**")
    .await
    .unwrap();

  while !ipc_token.is_cancelled() {
    match subscriber.recv_async().await {
      Ok(sample) => {
        // Refer to z_bytes.rs to see how to deserialize different types of message
        let payload = sample
          .payload()
          .try_to_string()
          .unwrap_or_else(|e| e.to_string().into());

        debug!(
          ">> [Subscriber] Received {} ('{}': '{}')",
          sample.kind(),
          sample.key_expr().as_str(),
          payload
        );
        if let Some(att) = sample.attachment() {
          let att = att.try_to_string().unwrap_or_else(|e| e.to_string().into());
          debug!(" ({att})");
        }
      }
      Err(msg) => {
        error!("{}", msg);
      }
    }
  }
}
