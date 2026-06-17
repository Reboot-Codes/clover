//! # AppDaemon
//!
//! The Application Daemon (a.k.a. AppD), handles external Podman applications and utility scripts in coordination with [`super::warehouse`]. Primary thread execution starts with [`appd_main`].
//!

pub mod docker;
pub mod ipc;
pub mod models;

use crate::server::appd::ipc::handle_ipc;

use self::docker::init_app;
use bollard::{
  Docker,
  API_DEFAULT_VERSION,
};
use docker::remove_app;
use models::AppDStore;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
  warn,
};
use zenoh_ext::{
  AdvancedPublisherBuilderExt,
  CacheConfig,
};

// TODO: Create application manifest schema/models

pub const MODULE_EVT_ID: &str = "com/reboot-codes/clover/hub/appdaemon";

#[instrument(skip(store, cancellation_tokens))]
pub async fn appd_main(
  store: AppDStore,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting AppDaemon...");

  let docker_path = store.config.lock().await.docker_daemon.clone();
  // TODO: Move to parameterized config from store!
  match Docker::connect_with_unix(&docker_path, 120, API_DEFAULT_VERSION) {
    Ok(docker_conn) => {
      info!("Connected to docker on {}!", docker_path.clone());
      let docker = Arc::new(docker_conn);

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

      let init_session = session.clone();
      let init_store = Arc::new(store.clone());
      let init_docker = docker.clone();
      cancellation_tokens
        .0
        .run_until_cancelled(async move {
          let mut init_apps = init_store.applications.lock().await;
          let mut apps_initialized = 0;

          if init_apps.len() == 0 {
            info!("No pre-configured applications to initialize.");
          } else {
            for (id, spec) in init_apps.iter_mut() {
              match init_app(init_docker.clone(), id, spec).await {
                Ok(_) => {
                  apps_initialized += 1;
                }
                Err(_e) => {
                  error!(
                    "Failed to initialize application {} ({})!",
                    spec.name.clone(),
                    id.clone()
                  );
                }
              }

              // Update the application state.
              init_store
                .applications
                .lock()
                .await
                .insert(id.clone(), spec.clone());
            }
          }

          if apps_initialized != init_apps.len() {
            warn!(
              "Initialized {} apps out of {}!",
              apps_initialized,
              init_apps.len()
            );
            status_publisher
              .put("ready:incomplete")
              .await
              .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
          } else {
            if apps_initialized != 0 {
              info!("Initialized all {} apps!", apps_initialized);
            }
            status_publisher
              .put("ready")
              .await
              .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
          }

          info!("AppDaemon Ready!");
        })
        .await;

      let cleanup_store = store.clone();
      let cleanup_docker = docker.clone();
      let cleanup_token = cancellation_tokens.0.clone();
      tokio::select! {
        _ = cleanup_token.cancelled() => {
          ipc_handle.abort();

          info!("Cleaning up applications...");

          let mut cleanup_apps = cleanup_store.applications.lock().await;
          let mut apps_removed = 0;

          if cleanup_apps.len() == 0 {
            info!("No applications to remove.");
          } else {
            for (id, spec) in cleanup_apps.iter_mut() {
              match remove_app(cleanup_docker.clone(), id, spec).await {
                Ok(_) => { apps_removed += 1; },
                Err(_e) => {
                  error!("Failed to remove application {} ({})!", spec.name.clone(), id.clone());
                }
              }

              // Update the application state.
              cleanup_store.applications.lock().await.insert(id.clone(), spec.clone());
            }
          }

          if apps_removed != cleanup_apps.len() {
            warn!("Only removed {} apps out of {}!", apps_removed, cleanup_apps.len());
          }

          std::mem::drop(store);

          cancellation_tokens.1.cancel();
        }
      }
    }
    Err(e) => {
      error!(
        "Failed to setup docker connection on {}, due to:\n{}",
        docker_path, e
      );
    }
  }

  info!("AppD has stopped!");
}
