pub mod docker;
pub mod models;

use self::docker::init_app;
use crate::utils::send_ipc_message;
use bollard::{
  API_DEFAULT_VERSION,
  Docker,
};
use docker::remove_app;
use log::{
  debug,
  error,
  info,
  warn,
};
use std::sync::Arc;
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
  unbounded_channel,
};
use tokio_util::sync::CancellationToken;
use url::Url;
use nexus::{server::models::UserConfig, arbiter::models::ApiKeyWithoutUID, user::NexusUser};

// TODO: Create application manifest schema/models

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.appd",
    pretty_name: "Clover: AppD",
    api_keys: vec![
      ApiKeyWithoutUID {
        allowed_events_to: "^nexus://com.reboot-codes.clover.appd(\\.(.*))*(\\/.*)*$"
        allowed_events_from: "^nexus://com.reboot-codes.clover.appd(\\.(.*))*(\\/.*)*$",
        echo: false,
        proxy: false
      }
    ]
  }
}

pub async fn appd_main(
  appd_store: Arc<AppDStore>,
  client: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting AppDaemon...");

  let docker_path = store.config.lock().await.docker_daemon.clone();
  // TODO: Move to parameterized config from store!
  match Docker::connect_with_unix(&docker_path, 120, API_DEFAULT_VERSION) {
    Ok(docker_conn) => {
      let docker = Arc::new(docker_conn);

      let init_store = Arc::new(store.clone());
      let init_user = Arc::new(user.clone());
      let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
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
              "Only initialized {} apps out of {}!",
              apps_initialized,
              init_apps.len()
            );

            init_user.send(
              "nexus://com.reboot-codes.clover.appd/status".to_string(),
              "incomplete-init".to_string(),
            )
            .await;
          } else {
            init_user.send(
              "nexus://com.reboot-codes.clover.appd/status".to_string(),
              "finished-init".to_string(),
            )
            .await;
          }
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
              if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.appd") {
                debug!("Processing: {}", msg.kind.clone());
              }
            }
          } => {}
        }
      });

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
      error!("Failed to setup docker connection!\n{}", e);
    }
  }

  info!("AppD has stopped!");
}
