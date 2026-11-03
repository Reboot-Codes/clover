//! # Renderer
//!
//! Uses graphical acceleration to render and display graphics on any connected [display components](super::modman::components::video::displays). Primary thread execution starts with [`renderer_main`].
//!
//! The renderer service is *only* responsible for creating, managing, destroying, and writing to an arbitrary number of OpenGL/Vulkan contexts who's frames are captured and sent to displays registered with modman when permitted by the user.
//!
//! The internal bevy-based engine is found within the [System UI](system_ui) package.
//!

//! # Renderer Service
//!
//! Hosts the Bevy graphics engine, manages 3D scene compositing, offscreen Render-to-Texture (RTT),
//! and streams rendered frames to physical and virtual displays.
//!
//! ## Zenoh Endpoints
//!
//! ### Publishers
//! - `com/reboot-codes/clover/hub/renderer/status`: Emits renderer initialization and readiness states.
//!   Implemented by [`renderer_main`].
//!
//! ### Dependencies
//! - `com/reboot-codes/clover/hub/modman/status`:
//!   - **Upstream Provider**: [`crate::server::modman::modman_main`]
//!   - **Rationale**: Awaits ModMan startup before querying registered displays.
//! - `com/reboot-codes/clover/hub/modman/components/by-type/video/displays/all`:
//!   - **Upstream Provider**: [`crate::server::modman::ipc::displays::display_queryable`]
//!   - **Rationale**: Ingests display metadata and dimensions to construct Bevy offscreen cameras.

pub mod ipc;
pub mod models;
pub mod system_ui;


use self::system_ui::system_ui_main;
use crate::server::{
  modman::MODULE_EVT_ID as MODMAN_EVT_ID,
  renderer::ipc::handle_ipc,
};
use crate::utils::{
  configure_zenoh,
  RecvSync,
};
use queues::*;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use system_ui::{
  AnyDisplayComponent,
  SystemUIIPC,
};
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

/// Zenoh base topic key expression for renderer service events and commands.
pub const MODULE_EVT_ID: &str = "com/reboot-codes/clover/hub/renderer";

/// Shared state store passed to the renderer service.
#[derive(Debug, Clone)]
pub struct RendererStore {
  /// Clover server configuration handle.
  pub config: Arc<Mutex<Config>>,
  /// Whether desktop mirror windows should be spawned for proxied displays.
  pub mirror_displays: bool,
}

impl RendererStore {
  /// Creates a new `RendererStore` with an optional configuration handle and display mirroring flag.
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>, mirror_displays: bool) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      Option::None => Arc::new(Mutex::new(Config::default())),
    };

    RendererStore {
      config,
      mirror_displays,
    }
  }
}

/// Primary entrypoint and lifecycle loop for the Clover graphics renderer service.
///
/// Connects to Zenoh, queries active displays from ModMan, registers them into Bevy's ECS
/// via [`SystemUIIPC`], handles IPC commands, and launches [`system_ui_main`].
///
/// ### Zenoh Publishers
/// - **Key Expression**: `com/reboot-codes/clover/hub/renderer/status`
/// - **Payload Format**: String status tokens (`"ready:engine"`, `"started"`, `"ready:incomplete"`)
///
/// #### Payload Example
/// ```text
/// "ready:engine"
/// ```
///
/// ### Zenoh Dependencies
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/status`
///   - **Upstream Provider**: [`crate::server::modman::modman_main`]
///   - **Rationale**: Blocks display configuration until ModMan is verified running.
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/components/by-type/video/displays/all`
///   - **Upstream Provider**: [`crate::server::modman::ipc::displays::display_queryable`]
///   - **Rationale**: Fetches registered displays to allocate Bevy render textures and cameras.
#[instrument(skip(store, cancellation_tokens))]
pub async fn renderer_main(

  store: RendererStore,
  cancellation_tokens: (CancellationToken, CancellationToken),
  port: u16,
) {
  info!("Starting Renderer...");

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

          let display_registration_queue = Arc::new(StdMutex::new(queue![]));
          let event_loop_proxy = Arc::new(StdMutex::new(None));

          let (bevy_cancel_tx, bevy_cancel_rx) = std::sync::mpsc::channel();
          let system_ui_ipc = SystemUIIPC {
            exit_channel: RecvSync(bevy_cancel_rx),
            event_loop_proxy: event_loop_proxy.clone(),
            display_registration_queue: display_registration_queue.clone(),
            zenoh_session: session.clone(),
            tokio_handle: tokio::runtime::Handle::current(),
          };

          let custom_bevy_ipc = system_ui_ipc;
          let bevy_thread =
            std::thread::spawn(move || system_ui_main(custom_bevy_ipc));

          // Signal that the renderer engine has initialized and is accepting displays
          status_publisher
            .put("ready:engine")
            .await
            .unwrap_or_else(|e| error!("Failed to publish renderer status: {e}"));

          // let display_handles = Arc::new(HashMap::new());

          let init_session = session.clone();
          let queue_for_modman = display_registration_queue.clone();
          let renderer_config = store.config.clone();
          let cli_mirror_override = store.mirror_displays;
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
                  .get(format!(
                    "{MODMAN_EVT_ID}/components/by-type/video/displays/all"
                  ))
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
                  match serde_json_lenient::from_str::<Vec<(String, AnyDisplayComponent)>>(&payload) {
                    Ok(displays) => {
                      let config_mirror = renderer_config.lock().await.renderer.mirror_displays;
                      let global_mirror = cli_mirror_override || config_mirror;
                      for (display_id, mut display) in displays {
                        if let AnyDisplayComponent::Physical(ref mut phys) = display {
                          if global_mirror && phys.mirror.is_none() {
                            phys.mirror = Some(true);
                          }
                        }
                        info!("Enqueuing display '{display_id}' into SystemUI registration queue");
                        let _ = queue_for_modman.lock().unwrap().add((display_id, display));
                      }
                    }
                    Err(e) => {
                      error!("Failed to deserialize displays payload: {e}");
                    }
                  }
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
              if let Ok(guard) = event_loop_proxy.lock() {
                if let Some(proxy) = guard.as_ref() {
                  let _ = proxy.send_event(bevy::winit::WakeUp);
                }
              }

              info!("Waiting for Bevy to shut down...");
              let (join_tx, join_rx) = tokio::sync::oneshot::channel();
              std::thread::spawn(move || {
                let res = bevy_thread.join();
                let _ = join_tx.send(res);
              });

              match tokio::time::timeout(std::time::Duration::from_millis(3000), join_rx).await {
                Ok(Ok(Ok(_))) => {
                  info!("Bevy thread exited cleanly");
                }
                Ok(Ok(Err(e))) => {
                  error!("Bevy thread panicked during shutdown: {:?}", e);
                }
                _ => {
                  tracing::warn!("Bevy thread did not finish within 3s; proceeding with server shutdown");
                }
              }

              // TODO: Clean up registered displays when server is shutting down.

              std::mem::drop(store);
            }
          }
        }
        Err(err) => {
          error!("FATAL: Failed to connect to zenoh due to:\n{err}");
        }
      }
    }
    Err(err) => {
      error!("FATAL: Failed to write configuration for zenoh connection due to:\n{err}")
    }
  }

  cancellation_tokens.1.cancel();

  info!("Renderer has stopped!");
}
