//! # Inference Engine
//!
//! Manages Machine Learning models and their respective accelerators.
//!
//! Uses `onnx` and `candle` to handle Analytical Models and Language Models respectively. Primary thread execution starts with [`inference_engine_main`].
//!
//! ## Zenoh Endpoints
//!
//! ### Publishers
//! - `com/reboot-codes/clover/server/inference_engine/status`: Emits Inference Engine readiness states.
//!   Implemented by [`inference_engine_main`].
//!
//! ### Subscribers
//! - `com/reboot-codes/clover/server/inference_engine/**`: Ingests model inference requests and tensor payloads.
//!   Implemented by [`ipc::handle_ipc`].


pub mod ipc;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
};
use zenoh_ext::{
  AdvancedPublisherBuilderExt,
  CacheConfig,
};

use crate::{
  server::inference_engine::ipc::handle_ipc,
  utils::configure_zenoh,
};

use super::warehouse::config::models::Config;

pub const MODULE_EVT_ID: &str = "com/reboot-codes/clover/hub/inference_engine";

#[derive(Debug, Clone)]
pub struct InferenceEngineStore {
  pub config: Arc<Mutex<Config>>,
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

/// Primary service entrypoint for the Inference Engine.
///
/// Connects to Zenoh, initializes ONNX / Candle model runtime environments, and handles inference tasks.
///
/// ### Zenoh Publishers
/// - **Key Expression**: `com/reboot-codes/clover/server/inference_engine/status`
/// - **Payload Format**: String status tokens (`"ready"`, `"stopping"`)
///
/// #### Payload Example
/// ```text
/// "ready"
/// ```
#[instrument(skip(inference_engine_store, cancellation_tokens))]
pub async fn inference_engine_main(

  inference_engine_store: InferenceEngineStore,
  cancellation_tokens: (CancellationToken, CancellationToken),
  port: u16,
) {
  info!("Starting Inference Engine...");

  let endpoint = format!("[\"tcp/localhost:{port}\"]");
  let config_ret = configure_zenoh(vec![
    ("connect/endpoints", &endpoint),
    (
      "timestamping/enabled",
      r#"{ router: true, peer: true, client: true }"#,
    ),
    ("mode", "\"client\""),
  ]);

  match config_ret {
    Ok(zenoh_config) => {
      debug!("Connecting to Zenoh...");

      match zenoh::open(zenoh_config).await {
        Ok(raw_session) => {
          let session = Arc::new(raw_session);
          debug!("Connected to Zenoh!");

          let status_publisher = session
            .declare_publisher(format!("{MODULE_EVT_ID}/status"))
            .cache(CacheConfig::default().max_samples(1))
            .await
            .unwrap();

          let ipc_token = cancellation_tokens.0.clone();
          let ipc_session = session.clone();
          let ipc_handle = tokio::task::spawn(handle_ipc(ipc_token, ipc_session));

          cancellation_tokens
            .0
            .run_until_cancelled(async move {
              status_publisher
                .put("ready")
                .await
                .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));

              info!("InferenceEngine Ready!");
            })
            .await;

          let cleanup_token = cancellation_tokens.0.clone();
          tokio::select! {
            _ = cleanup_token.cancelled() => {
              ipc_handle.abort();

              info!("Cleaning up networks...");
              // TODO: Clean up registered networks when server is shutting down.
            }
          }
        }
        Err(err) => {
          error!("FATAL: Failed to connect to zenoh due to:\n{err}");
        }
      }
    }
    Err(err) => {
      error!("FATAL: Failed to write configuration for zenoh connection due to:\n{err}");
    }
  }

  cancellation_tokens.1.cancel();

  info!("Inference Engine has stopped!");
}
