//! # Inference Engine
//!
//! Manages Machine Learning models and their respective accelerators.
//!
//! Uses [`onnx`] and [`candle`] to handle Analytical Models and Language Models respectively. Primary thread execution starts with [`inference_engine_main`].
//!

pub mod ipc;

use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
  user::NexusUser,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
  span,
};

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
      Option::None => Arc::new(Mutex::new(Config::default())),
    };

    InferenceEngineStore { config }
  }
}

#[instrument(skip(inference_engine_store, user, cancellation_tokens))]
pub async fn inference_engine_main(
  inference_engine_store: InferenceEngineStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Inference Engine...");

  let mut zenoh_config = zenoh::Config::default();

  zenoh_config.insert_json5("connect/endpoints", "tcp/localhost:6699");

  debug!("Connecting to Zenoh...");
  let session = Arc::new(zenoh::open(zenoh_config).await.unwrap());
  debug!("Connected to Zenoh!");

  let ipc_token = cancellation_tokens.0.clone();
  let ipc_session = session.clone();
  let ipc_handle = tokio::task::spawn(async move {
    let subscriber = ipc_session
      .declare_subscriber("com/reboot-codes/clover/server/inference_engine/**")
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
  });
  let init_session = session.clone();
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      let init_publisher = init_session
        .declare_publisher("com/reboot-codes/clover/server/inference_engine/status")
        .await
        .unwrap();

      init_publisher.put("ready");
    })
    .await;

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_handle.abort();

      info!("Cleaning up networks...");
      // TODO: Clean up registered networks when server is shutting down.

      cancellation_tokens.1.cancel();
    }
  }

  info!("Inference Engine has stopped!");
}
