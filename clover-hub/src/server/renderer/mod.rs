//! # Renderer
//!
//! Uses graphical acceleration to render and display graphics on any connected [display components](super::modman::components::video::displays). Primary thread execution starts with [`renderer_main`].
//!
//! The renderer service is *only* responsible for creating, managing, destroying, and writing to an arbitrary number of OpenGL/Vulkan contexts who's frames are captured and sent to displays registered with modman when permitted by the user.
//!
//! The internal bevy-based engine is found within the [System UI](system_ui) package.
//!

pub mod ipc;
pub mod models;
pub mod system_ui;

use self::system_ui::system_ui_main;
use crate::server::{
  modman::MODULE_EVT_ID as MODMAN_EVT_ID,
  renderer::ipc::handle_ipc,
};
use crate::utils::RecvSync;
use queues::*;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use system_ui::SystemUIIPC;
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
  AdvancedSubscriberBuilderExt,
  CacheConfig,
  HistoryConfig,
  RecoveryConfig,
};

use super::warehouse::config::models::Config;

pub const MODULE_EVT_ID: &str = "com/reboot-codes/clover/hub/renderer";

#[derive(Debug, Clone)]
pub struct RendererStore {
  pub config: Arc<Mutex<Config>>,
}

impl RendererStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      Option::None => Arc::new(Mutex::new(Config::default())),
    };

    RendererStore { config }
  }
}

#[instrument(skip(store, cancellation_tokens))]
pub async fn renderer_main(
  store: RendererStore,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Renderer...");

  let mut zenoh_config = zenoh::Config::default();

  zenoh_config.insert_json5("connect/endpoints", "tcp/localhost:6699");
  zenoh_config
    .insert_json5(
      "timestamping/enabled",
      r#"{ router: true, peer: true, client: true }"#,
    )
    .unwrap();

  debug!("Connecting to Zenoh...");
  let session = Arc::new(zenoh::open(zenoh_config).await.unwrap());
  debug!("Connected to Zenoh!");

  let status_publisher = session
    .declare_publisher(format!("{MODULE_EVT_ID}/status"))
    .cache(CacheConfig::default().max_samples(1))
    .await
    .unwrap();

  let ipc_token = cancellation_tokens.0.clone();
  let ipc_session = session.clone();
  let ipc_handle = tokio::task::spawn(handle_ipc(ipc_token, ipc_session));

  let display_registration_queue = Arc::new(StdMutex::new(queue![]));

  let (bevy_cancel_tx, bevy_cancel_rx) = std::sync::mpsc::channel();
  let system_ui_ipc = SystemUIIPC {
    exit_channel: RecvSync(bevy_cancel_rx),
    display_registration_queue: display_registration_queue.clone(),
  };

  // TODO: Add this as a CLI/Config option!
  let custom_bevy_ipc = system_ui_ipc;
  let bevy_thread = std::thread::spawn(move || system_ui_main(custom_bevy_ipc, Some(false)));

  // let display_handles = Arc::new(HashMap::new());

  let init_session = session.clone();
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      debug!("Waiting for ModMan to be ready...");

      let modman_status_subscriber = init_session
        .declare_subscriber(format!("{MODMAN_EVT_ID}/status"))
        .history(HistoryConfig::default().detect_late_publishers())
        .recovery(RecoveryConfig::default().heartbeat())
        .await
        .unwrap();

      let modman_ready = loop {
        match tokio::time::timeout(
          std::time::Duration::from_millis(500),
          modman_status_subscriber.recv_async(),
        )
        .await
        {
          Ok(Ok(sample)) => {
            let status = sample
              .payload()
              .try_to_string()
              .unwrap_or_else(|e| e.to_string().into());
            debug!("ModMan Status: {status}");
            break status.to_string();
          }
          Ok(Err(e)) => {
            error!("Subscriber channel error: {e}");
            break "error".to_string();
          }
          Err(_) => {
            debug!("Timed out waiting for ModMan status, querying cache directly...");
            match init_session.get(format!("{MODMAN_EVT_ID}/status")).await {
              Ok(replies) => {
                if let Ok(reply) = replies.recv_async().await {
                  if let Ok(sample) = reply.into_result() {
                    let status = sample
                      .payload()
                      .try_to_string()
                      .unwrap_or_else(|e| e.to_string().into());
                    debug!("ModMan Status (from cache query): {status}");
                    break status.to_string();
                  }
                }
                debug!("No cached ModMan status yet, retrying...");
              }
              Err(e) => {
                error!("Failed to query ModMan status: {e}");
                break "error".to_string();
              }
            }
          }
        }
      };

      drop(modman_status_subscriber);

      if modman_ready == "error" {
        error!("Failed to get ModMan ready status!");
        status_publisher
          .put("ready:incomplete")
          .await
          .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
        return;
      }

      info!("Requesting displays to setup in SystemUI from ModMan...");

      let displays_payload = loop {
        match init_session
          .get(format!("{MODMAN_EVT_ID}/displays/get"))
          .await
        {
          Ok(reply_fifo) => match reply_fifo.recv_async().await {
            Ok(reply) => match reply.result() {
              Ok(sample) => {
                break Ok(
                  sample
                    .payload()
                    .try_to_string()
                    .unwrap_or_else(|e| e.to_string().into())
                    .to_string(),
                );
              }
              Err(e) => {
                break Err(
                  e.payload()
                    .try_to_string()
                    .unwrap_or_else(|e| e.to_string().into())
                    .to_string(),
                );
              }
            },
            Err(e) => {
              debug!("displays/get queryable not ready yet ({e}), retrying in 100ms...");
              tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            }
          },
          Err(e) => {
            error!("Unable to setup Querier:\n{e}");
            break Err(e.to_string());
          }
        }
      };

      match displays_payload {
        Ok(payload) => {
          debug!("Got displays: {:?}", payload);
          status_publisher
            .put("ready")
            .await
            .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
        }
        Err(payload) => {
          error!("Got error payload:\n{payload}");
          status_publisher
            .put("ready:incomplete")
            .await
            .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
        }
      }

      info!("Renderer Ready!");
    })
    .await;

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_handle.abort();

      info!("Cleaning up displays...");

      let _ = bevy_cancel_tx.send(bevy::prelude::AppExit::Success);

      info!("Waiting for Bevy to shut down...");
      match bevy_thread.join() {
        Ok(_) => {
          info!("Bevy thread exited cleanly");
        }
        Err(e) => {
          error!("Bevy thread panicked during shutdown: {:?}", e);
        }
      }

      // TODO: Clean up registered displays when server is shutting down.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Renderer has stopped!");
}
