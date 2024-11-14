pub mod models;
pub mod docker;

use std::sync::Arc;
use docker::remove_app;
use log::{debug, error, info, warn};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use url::Url;
use crate::utils::send_ipc_message;
use bollard::{Docker, API_DEFAULT_VERSION};
use self::docker::init_app;

use super::evtbuzz::models::{IPCMessageWithId, CoreUserConfig, Store};

// TODO: Create application manifest schema/models

pub async fn appd_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_token: CancellationToken
) {
  info!("Starting AppDaemon...");

  // TODO: Move to parameterized config from store!
  match Docker::connect_with_unix(&store.config.docker_daemon.clone(), 120, API_DEFAULT_VERSION) {
    Ok(docker_conn) => {
      let docker = Arc::new(docker_conn);

      let init_store = Arc::new(store.clone());
      let init_user = Arc::new(user_config.clone());
      let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
      let init_docker = docker.clone();
      cancellation_token.run_until_cancelled(async move {
        let mut init_apps = init_store.applications.lock().await;
        let mut apps_initialized = 0;

        if init_apps.len() == 0 {
          info!("No pre-configured applications to initialize.");
        } else {
          for (id, spec) in init_apps.iter_mut() {
            match init_app(init_docker.clone(), id, spec).await {
              Ok(_) => { apps_initialized += 1; },
              Err(_e) => {
                error!("Failed to initialize application {} ({})!", spec.name.clone(), id.clone());
              }
            }

            // Update the application state.
            init_store.applications.lock().await.insert(id.clone(), spec.clone());
          }
        }

        if apps_initialized != init_apps.len() {
          warn!("Only initialized {} apps out of {}!", apps_initialized, init_apps.len());

          let _ = send_ipc_message(
            &init_store, 
            &init_user, 
            init_from_tx.clone(), 
            "clover://appd.clover.reboot-codes.com/status".to_string(), 
            "incomplete-init".to_string()
          ).await;
        } else {
          let _ = send_ipc_message(
            &init_store, 
            &init_user, 
            init_from_tx.clone(), 
            "clover://appd.clover.reboot-codes.com/status".to_string(), 
            "finished-init".to_string()
          ).await;
        }
      }).await;

      let ipc_recv_token = cancellation_token.clone();
      let ipc_recv_handle = tokio::task::spawn(async move {
        tokio::select! {
          _ = ipc_recv_token.cancelled() => {
            debug!("ipc_recv exited");
          },
          _ = async move {
            while let Some(msg) = ipc_rx.recv().await {
              let kind = Url::parse(&msg.kind.clone()).unwrap();

              // Verify that we care about this event.
              if kind.host().unwrap() == url::Host::Domain("appd.clover.reboot-codes.com") {
                debug!("Processing: {}", msg.kind.clone());
              }
            }
          } => {}
        }
      });

      let ipc_trans_token = cancellation_token.clone();
      let ipc_trans_tx = Arc::new(ipc_tx.clone());
      let ipc_trans_handle = tokio::task::spawn(async move {
        tokio::select! {
          _ = async move {
            while let Some(msg) = init_from_rx.recv().await {
              match ipc_trans_tx.send(msg) {
                Ok(_) => {},
                Err(_) => {
                  debug!("Failed to send message to IPC bus!");
                }
              }
            }
          } => {},
          _ = ipc_trans_token.cancelled() => {
            debug!("ipc_trans exited");
          }
        }
      });

      let cleanup_store = store.clone();
      let cleanup_docker = docker.clone();
      let cleanup_token = cancellation_token.clone();
      tokio::select! {
        _ = cleanup_token.cancelled() => {
          ipc_recv_handle.abort();
          ipc_trans_handle.abort();
          
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
        }
      }
    },
    Err(e) => {
      error!("Failed to setup docker connection!\n{}", e);
    }
  }

  info!("AppD has stopped!");
}
