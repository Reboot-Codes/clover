use crate::utils::send_ipc_message;
use log::{
  debug,
  info,
};
use std::sync::Arc;
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
  unbounded_channel,
};
use tokio_util::sync::CancellationToken;
use url::Url;
use nexus::server::models::{
  ApiKeyWithKey,
  IPCMessageWithId,
};
use nexus::{server::models::UserConfig, arbiter::models::ApiKeyWithoutUID, user::NexusUser};

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.inference-engine",
    pretty_name: "Clover: Inference Engine",
    api_keys: vec![
      ApiKeyWithoutUID {
        allowed_events_to: "^nexus://com.reboot-codes.clover.inference-engine(\\.(.*))*(\\/.*)*$"
        allowed_events_from: "^nexus://com.reboot-codes.clover.inference-engine(\\.(.*))*(\\/.*)*$",
        echo: false,
        proxy: false
      }
    ]
  }
}

pub async fn inference_engine_main(
  inference_engine_store: Arc<InferenceEngineStore>,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Inference Engine...");

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user.clone());
  let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      init_user.send(
        "nexus://com.reboot-codes.clover.inference-engine/status".to_string(),
        "finished-init".to_string(),
      )
      .await;
    })
    .await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let (ipc_rx, ipc_handle) = client.subscribe();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Ok(msg) = ipc_rx.recv().await {
          let kind = Url::parse(&msg.kind.clone()).unwrap();

          // Verify that we care about this event.
          if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.inference-engine") {
            debug!("Processing: {}", msg.kind.clone());
          }
        }
      } => {}
    }
  });

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_handle.abort();

      info!("Cleaning up networks...");
      // TODO: Clean up registered networks when server is shutting down.

      cancellation_tokens.1.cancel();
    }
  }

  info!("Inference Engine has stopped!");
}
