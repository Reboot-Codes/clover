pub mod docker;
pub mod ipc;
pub mod models;

use self::docker::init_app;
use bollard::{
  Docker,
  API_DEFAULT_VERSION,
};
use docker::remove_app;
use ipc::handle_ipc_msg;
use log::{
  debug,
  error,
  info,
  warn,
};
use models::AppDStore;
use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
  user::NexusUser,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

// TODO: Create application manifest schema/models

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.appd".to_string(),
    pretty_name: "Clover: AppD".to_string(),
    api_keys: vec![ApiKeyWithoutUID {
      allowed_events_to: vec![
        "^nexus://com.reboot-codes.clover.appd(\\.(.*))*(\\/.*)*$".to_string()
      ],
      allowed_events_from: vec![
        "^nexus://com.reboot-codes.clover.appd(\\.(.*))*(\\/.*)*$".to_string()
      ],
      echo: false,
      proxy: false,
    }],
  }
}

pub async fn appd_main(
  store: AppDStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting AppDaemon...");

  let docker_path = store.config.lock().await.docker_daemon.clone();
  // TODO: Move to parameterized config from store!
  match Docker::connect_with_unix(&docker_path, 120, API_DEFAULT_VERSION) {
    Ok(docker_conn) => {
      info!("Connected to docker on {}!", docker_path.clone());
      let docker = Arc::new(docker_conn);

      let ipc_recv_token = cancellation_tokens.0.clone();
      let (ipc_rx, ipc_handle) = user.subscribe();
      let ipc_recv_handle = tokio::task::spawn(async move {
        tokio::select! {
          _ = ipc_recv_token.cancelled() => {
            debug!("ipc_recv exited");
          },
          _ = handle_ipc_msg(ipc_rx) => {}
        }
      });

      let init_store = Arc::new(store.clone());
      let init_user = Arc::new(user.clone());
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
            match init_user.send(
              &"nexus://com.reboot-codes.clover.appd/status".to_string(),
              &"incomplete-init".to_string(),
              &None,
            ) {
              Err(e) => {
                error!(
                  "Error when letting peers know about incomplete init state: {}",
                  e
                );
              }
              _ => {}
            }
          } else {
            if apps_initialized != 0 {
              info!("Initialized all {} apps!", apps_initialized);
            }
            match init_user.send(
              &"nexus://com.reboot-codes.clover.appd/status".to_string(),
              &"finished-init".to_string(),
              &None,
            ) {
              Err(e) => {
                error!(
                  "Error when letting peers know about complete init state: {}",
                  e
                );
              }
              _ => {}
            }
          }
        })
        .await;

      let cleanup_store = store.clone();
      let cleanup_docker = docker.clone();
      let cleanup_token = cancellation_tokens.0.clone();
      tokio::select! {
        _ = cleanup_token.cancelled() => {
          ipc_recv_handle.abort();
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
