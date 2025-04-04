use log::{
  debug,
  info,
};
use nexus::server::models::IPCMessageWithId;
use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
  user::NexusUser,
};
use std::sync::Arc;
use tokio::sync::mpsc::unbounded_channel;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use url::Url;

use super::warehouse::config::models::Config;

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.inference-engine".to_string(),
    pretty_name: "Clover: Inference Engine".to_string(),
    api_keys: vec![ApiKeyWithoutUID {
      allowed_events_to: vec![
        "^nexus://com.reboot-codes.clover.inference-engine(\\.(.*))*(\\/.*)*$".to_string(),
      ],
      allowed_events_from: vec![
        "^nexus://com.reboot-codes.clover.inference-engine(\\.(.*))*(\\/.*)*$".to_string(),
      ],
      echo: false,
      proxy: false,
    }],
  }
}

#[derive(Debug, Clone)]
pub struct InferenceEngineStore {
  config: Arc<Mutex<Config>>,
}

impl InferenceEngineStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      None => Arc::new(Mutex::new(Config::default())),
    };

    InferenceEngineStore { config }
  }
}

pub async fn inference_engine_main(
  inference_engine_store: InferenceEngineStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Inference Engine...");

  let init_user = Arc::new(user.clone());
  let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      init_user.send(
        &"nexus://com.reboot-codes.clover.inference-engine/status".to_string(),
        &"finished-init".to_string(),
      );
    })
    .await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let (mut ipc_rx, ipc_handle) = user.subscribe();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Some(msg) = ipc_rx.recv().await {
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
