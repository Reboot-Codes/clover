//! # CloverHub Server
//!
//! Contains the modular, core logic to run a Clover instance. Process control is found here in [`server_main`].
//!
//! Primary server components in startup order are:
//!
//! - **[`warehouse`]:** Manages external repositories, and handles configuration parsing from them and the base configuration.
//! - **[`modman`]:** Manages communication with non-networked [Modules](modman::models::modules::Module) and their [Components](modman::models::components::CloverComponentMeta).
//! - **[`renderer`]:** Uses graphical acceleration to render and display graphics on any connected [display components](modman::components::video::displays::models::PhysicalDisplayComponent).
//! - **[`inference_engine`]:** Manages Machine Learning models and their respective accelerators.
//! - **[`appd`]:** The Application Daemon (a.k.a. AppDaemon, AppD), handles external Podman applications and utility scripts in coordination with [`warehouse`].
//!
//! Generally, threads spawned here have a similar structure with IPC recv/send sub-threads, and Startup/Shutdown functions. Following this pattern ensures maintainability and ease of use.
//!
//! ## Zenoh API Topology
//!
//! All services communicate asynchronously over an embedded Zenoh network topology.
//!
//! | Service | Key Expression | Type | Payload / Schema |
//! |---|---|---|---|
//! | **Logging** | `com/reboot-codes/clover/hub/logs` | Pub | JSON [`crate::logging::CloverLogRecord`] |
//! | **Warehouse** | `com/reboot-codes/clover/hub/warehouse/status` | Pub | `"started"`, `"stopping"` |
//! | **ModMan** | `com/reboot-codes/clover/hub/modman/status` | Pub | `"started"`, `"stopping"` |
//! | | `.../modman/modules/all` | Queryable | JSON `Vec<ModuleSummary>` |
//! | | `.../modman/ports/all` | Queryable | JSON `{"uart": ..., "can_2": ...}` |
//! | | `.../modman/components/by-type/video/displays/all` | Queryable | JSON `Vec<(String, AnyDisplayComponent)>` |
//! | | `.../modman/gestures/all` | Queryable | JSON `Vec<GestureSummary>` |
//! | | `.../modman/gestures/active` | Queryable | JSON `{"foreground": ..., "background": ..., "states": ...}` |
//! | | `.../modman/gestures/update` | Queryable | JSON `GestureUpdatePayload` |
//! | | `.../modman/gestures/status` | Pub | JSON `{"gesture_id": ..., "status": ...}` |
//! | **Renderer** | `com/reboot-codes/clover/hub/renderer/status` | Pub | `"ready:engine"`, `"started"`, `"ready:incomplete"` |
//! | | `.../components/by-id/{display_id}/display/frame` | Pub | JSON [`modman::components::video::displays::models::DisplayFrameEnvelope`] |
//! | **Inference Engine** | `com/reboot-codes/clover/server/inference_engine/status` | Pub | `"ready"`, `"stopping"` |
//! | **AppDaemon** | `com/reboot-codes/clover/server/appdaemon/status` | Pub | `"started"`, `"stopping"` |


pub mod appd;
pub mod inference_engine;
pub mod modman;
pub mod renderer;
pub mod warehouse;

use appd::{
  appd_main,
  models::AppDStore,
};
use inference_engine::{
  inference_engine_main,
  InferenceEngineStore,
};
use modman::{
  models::store::ModManStore,
  modman_main,
};
use renderer::{
  renderer_main,
  RendererStore,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  info_span,
  Instrument,
};
use warehouse::{
  db::connect_db,
  models::WarehouseStore,
  setup_warehouse,
  warehouse_main,
};

/// Tracks services that are run by the main clover-hub process.
pub struct InitializedService {
  /// Pretty name for the service
  pub name: &'static str,
  /// Triggers this specific service's shutdown loop
  pub shutdown_trigger: CancellationToken,
  /// Acknowledges when this service has completely stopped running
  pub shutdown_ack: CancellationToken,
}

pub fn spawn_cleanup_task(
  global_cancellation_token: CancellationToken,
  services: Vec<InitializedService>,
) -> tokio::task::JoinHandle<()> {
  let shutdown_span = info_span!("server_shutdown");

  tokio::task::spawn(
    async move {
      global_cancellation_token.cancelled().await;
      info!("Server graceful shutdown initiated. Stopping services in reverse order...");

      let services_iter = services.into_iter();

      for service in services_iter.rev() {
        info!("Shutting down {}...", service.name);
        service.shutdown_trigger.cancel();

        match tokio::time::timeout(std::time::Duration::from_secs(5), service.shutdown_ack.cancelled()).await {
          Ok(_) => info!("{} Shutdown Complete", service.name),
          Err(_) => tracing::warn!("{} shutdown timed out after 5s; proceeding with graceful shutdown", service.name),
        }
      }

      info!("Graceful shutdown successful!");
    }
    .instrument(shutdown_span),
  )
}

pub async fn server_main(
  data_dir: &String,
  port: u16,
  mirror_displays: bool,
  cancellation_token: CancellationToken,
  server_token: CancellationToken,
) {
  info!("Starting CloverHub...");

  let warehouse_store = WarehouseStore::new(None);

  // TODO: Let each process run independantly of eachother using nexus
  let warehouse_setup_store = Arc::new(warehouse_store.clone());
  match setup_warehouse(data_dir.clone(), warehouse_setup_store).await {
    Ok(_) => {
      match connect_db(warehouse_store.clone()).await {
        Ok(_) => {
          debug!("initializing Stores...");
          let renderer_store = RendererStore::new(
            Some(warehouse_store.config.clone()),
            mirror_displays,
          );
          let modman_store = ModManStore::new(
            Some(warehouse_store.config.clone()),
            Some(warehouse_store.repos.clone()),
          );
          let inference_engine_store =
            InferenceEngineStore::new(Some(warehouse_store.config.clone()));
          let appd_store = AppDStore::new(Some(warehouse_store.config.clone()));
          let primary_api_key = warehouse_store
            .clone()
            .config
            .clone()
            .lock()
            .await
            .primary_api_key
            .clone();
          debug!("Stores initialized!");

          let mut services = Vec::new();

          // Start Warehouse
          debug!("Starting Warehouse...");
          let warehouse_store_clone = Arc::new(warehouse_store.clone());
          let warehouse_tokens = (CancellationToken::new(), CancellationToken::new());
          let warehouse_tokens_clone = warehouse_tokens.clone();
          let warehouse_handle = tokio::task::spawn(async move {
            warehouse_main(warehouse_store_clone.clone(), warehouse_tokens_clone, port).await;
          });
          services.push(InitializedService {
            name: "Warehouse",
            shutdown_trigger: warehouse_tokens.0,
            shutdown_ack: warehouse_tokens.1,
          });

          // Start ModMan
          debug!("Starting Modman...");
          let modman_tokens = (CancellationToken::new(), CancellationToken::new());
          let modman_tokens_clone = modman_tokens.clone();
          let modman_handle = tokio::task::spawn(async move {
            modman_main(modman_store, modman_tokens_clone, port).await;
          });
          services.push(InitializedService {
            name: "ModMan",
            shutdown_trigger: modman_tokens.0,
            shutdown_ack: modman_tokens.1,
          });

          // Start Renderer
          debug!("Starting Renderer...");
          let renderer_tokens = (CancellationToken::new(), CancellationToken::new());
          let renderer_tokens_clone = renderer_tokens.clone();
          let renderer_handle = tokio::task::spawn(async move {
            renderer_main(renderer_store, renderer_tokens_clone, port).await;
          });
          services.push(InitializedService {
            name: "Renderer",
            shutdown_trigger: renderer_tokens.0,
            shutdown_ack: renderer_tokens.1,
          });

          // Start InferenceEngine
          debug!("Starting Inference Engine...");
          let inference_engine_tokens = (CancellationToken::new(), CancellationToken::new());
          let inference_engine_tokens_clone = inference_engine_tokens.clone();
          let inference_engine_handle = tokio::task::spawn(async move {
            inference_engine_main(inference_engine_store, inference_engine_tokens_clone, port).await;
          });
          services.push(InitializedService {
            name: "Inference Engine",
            shutdown_trigger: inference_engine_tokens.0,
            shutdown_ack: inference_engine_tokens.1,
          });

          // Start AppDaemon
          debug!("Starting AppDaemon...");
          let appd_tokens = (CancellationToken::new(), CancellationToken::new());
          let appd_tokens_clone = appd_tokens.clone();
          let appd_handle = tokio::task::spawn(async move {
            appd_main(appd_store, appd_tokens_clone, port).await;
          });
          services.push(InitializedService {
            name: "AppDaemon",
            shutdown_trigger: appd_tokens.0,
            shutdown_ack: appd_tokens.1,
          });

          // Start Zenoh Log Publisher concurrent to the server services
          let log_token = CancellationToken::new();
          let log_token_clone = log_token.clone();
          let log_handle = tokio::task::spawn(crate::logging::start_zenoh_log_publisher_for_port(
            port,
            log_token_clone,
          ));

          let cleanup_handle = spawn_cleanup_task(cancellation_token, services);

          // Wait for cleanup task to finish orchestrating the shutdown of all services
          let _ = cleanup_handle.await;

          info!("All service cleanups finished. Cleaning up background tasks...");
          warehouse_handle.abort();
          renderer_handle.abort();
          modman_handle.abort();
          inference_engine_handle.abort();
          appd_handle.abort();

          info!("CloverHub Server has exited.");
          log_token.cancel();
          let _ = tokio::time::timeout(std::time::Duration::from_secs(1), log_handle).await;
          server_token.cancel();
        }
        Err(e) => {
          error!("Failed to connect to database: {}", e);
          server_token.cancel();
        }
      }
    }
    Err(e) => {
      match e {
        warehouse::Error::FailedToCreateDataDir { error } => {
          error!(
            "Failed to create data directory! Please create `{}` and set the proper permissions manually, then re-run the server. Failed due to:\n{}",
            data_dir.clone(),
            error
          );
        }
        warehouse::Error::FailedToCreateConfigFile { error } => {
          error!(
            "Failed to create the configuration file, due to:\n{}",
            error
          );
        }
        warehouse::Error::FailedToOpenConfigFile { error } => {
          error!("Failed to open config file, due to:\n{}", error);
        }
        warehouse::Error::FailedToParseConfigFile { error } => {
          error!("Failed to parse configuration file, due to:\n{}", error);
        }
        warehouse::Error::FailedToReadConfigFile { error } => {
          error!("Failed to read configuration file, due to:\n{}", error);
        }
        warehouse::Error::FailedToWriteToConfigFile { error } => {
          error!(
            "Failed to write to the configuration file, due to:\n{}",
            error
          );
        }
        warehouse::Error::FailedToCreateReposDir { error } => {
          error!(
            "Failed to create the repository storage dir, due to:\n{}",
            error
          );
        }
        warehouse::Error::FailedToDownloadAndRegisterRepos { error } => {
          error!(
            "Failed to download and/or register all repositories, due to:\n{}",
            error
          );
        }
        warehouse::Error::FailedToUpdateRepoDirectoryStructure { error } => {
          error!(
            "Failed to update the repo directory structure, due to:\n{}",
            error
          );
        }
      }

      server_token.cancel();
    }
  };
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicBool, Ordering};

  #[tokio::test]
  async fn test_spawn_cleanup_task_service_sequencing() {
    let global_token = CancellationToken::new();

    let trigger_a = CancellationToken::new();
    let ack_a = CancellationToken::new();
    let trigger_b = CancellationToken::new();
    let ack_b = CancellationToken::new();

    let service_a_cleaned = Arc::new(AtomicBool::new(false));
    let service_b_cleaned = Arc::new(AtomicBool::new(false));

    let a_cleaned_clone = service_a_cleaned.clone();
    let t_a = trigger_a.clone();
    let a_a = ack_a.clone();
    tokio::task::spawn(async move {
      t_a.cancelled().await;
      a_cleaned_clone.store(true, Ordering::SeqCst);
      a_a.cancel();
    });

    let b_cleaned_clone = service_b_cleaned.clone();
    let t_b = trigger_b.clone();
    let a_b = ack_b.clone();
    tokio::task::spawn(async move {
      t_b.cancelled().await;
      b_cleaned_clone.store(true, Ordering::SeqCst);
      a_b.cancel();
    });

    let services = vec![
      InitializedService {
        name: "ServiceA",
        shutdown_trigger: trigger_a,
        shutdown_ack: ack_a,
      },
      InitializedService {
        name: "ServiceB",
        shutdown_trigger: trigger_b,
        shutdown_ack: ack_b,
      },
    ];

    let cleanup_handle = spawn_cleanup_task(global_token.clone(), services);

    // Assert services have not yet shut down before token cancellation
    assert!(!service_a_cleaned.load(Ordering::SeqCst));
    assert!(!service_b_cleaned.load(Ordering::SeqCst));

    // Cancel global token
    global_token.cancel();

    // Await cleanup task completion
    let res = tokio::time::timeout(std::time::Duration::from_secs(3), cleanup_handle).await;
    assert!(res.is_ok());

    // Both services should be acknowledged as cleaned up
    assert!(service_a_cleaned.load(Ordering::SeqCst));
    assert!(service_b_cleaned.load(Ordering::SeqCst));
  }

  #[tokio::test]
  async fn test_cleanup_task_handles_unresponsive_service_without_hanging() {
    let global_token = CancellationToken::new();

    let trigger_unresponsive = CancellationToken::new();
    let ack_never_called = CancellationToken::new();

    let services = vec![
      InitializedService {
        name: "UnresponsiveService",
        shutdown_trigger: trigger_unresponsive.clone(),
        shutdown_ack: ack_never_called,
      },
    ];

    let cleanup_handle = spawn_cleanup_task(global_token.clone(), services);

    // Cancel global token
    global_token.cancel();

    // Await cleanup task completion - should finish within ~5.5s due to timeout
    let res = tokio::time::timeout(std::time::Duration::from_secs(6), cleanup_handle).await;
    assert!(res.is_ok());
    assert!(trigger_unresponsive.is_cancelled());
  }
}

