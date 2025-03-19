pub mod appd;
pub mod inference_engine;
pub mod modman;
pub mod renderer;
pub mod warehouse;

use appd::{models::AppDStore, appd_main};
use nexus::server::listener::nexus_listener;
use nexus::server::models::{
  IPCMessageWithId,
  NexusStore,
  UserConfig
};
use inference_engine::{InferenceEngineStore, inference_engine_main};
use log::{
  debug,
  error,
  info,
};
use modman::{models::ModManStore, modman_main};
use renderer::{RendererStore, renderer_main};
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;
use warehouse::{models::WarehouseStore, setup_warehouse, warehouse_main};

pub async fn server_main(
  data_dir: &String,
  port: u16,
  cancellation_token: CancellationToken,
  server_token: CancellationToken,
) {
  info!("Starting CloverHub...");

  let warehouse_store = WarehouseStore::new();
  let renderer_store = RendererStore::new();
  let modman_store = ModManStore::new();
  let inference_engine_store = InferenceEngineStore::new();
  let appd_store = AppDStore::new();

  // TODO: Let each process run independantly of eachother using nexus

  let warehouse_setup_store = Arc::new(warehouse_store.clone());
  match setup_warehouse(data_dir.clone(), warehouse_setup_store).await {
    Ok(_) => {
      // Add users to nexus
      let (mut nexus_store, master_user_config) = NexusStore::new(&"Owner".to_string()).await;
      let warehouse_user_config = nexus_store.add_user(warehouse::gen_user()).await;
      let renderer_user_config = nexus_store.add_user(renderer::gen_user()).await;
      let modman_user_config = nexus_store.add_user(modman::gen_user()).await;
      let inference_engine_user_config = nexus_store.add_user(inference_engine::gen_user()).await;
      let appd_user_config = nexus_store.add_user(appd_engine::gen_user()).await;

      // Create NexusUser objects
      let (warehouse_user, from_warehouse, to_warehouse) = nexus_store.connect_user(warehouse_user_config.keys[0].clone()).await.unwrap();
      let (renderer_user, from_renderer, to_renderer) = nexus_store.connect_user(renderer_user_config.keys[0].clone()).await.unwrap();
      let (modman_user, from_modman, to_modman) = nexus_store.connect_user(modman_user_config.keys[0].clone()).await.unwrap();
      let (inference_engine_user, from_inference_engine, to_inference_engine) = nexus_store.connect_user(inference_engine_user_config.keys[0].clone()).await.unwrap();
      let (appd_user, from_appd, to_appd) = nexus_store.connect_user(appd_user_config.keys[0].clone()).await.unwrap();

      // Start Nexus
      let nexus_port = Arc::new(port);
      let nexus_store_clone = Arc::new(nexus_store.clone());
      let nexus_tokens = (CancellationToken::new(), CancellationToken::new());
      let nexus_tokens_clone = nexus_tokens.clone();
      let nexus_handle = tokio::task::spawn(async move {
        nexus_listener(
          *nexus_port.to_owned(),
          nexus_store,
          vec![
            (&warehouse_user_config, 0, to_warehouse)
            (&renderer_user_config, 0, to_renderer)
            (&modman_user_config, 0, to_modman)
            (&inference_engine_user_config, 0, to_inference_engine)
            (&appd_user_config, 0, to_appd)
          ],
          vec![
            from_warehouse,
            from_renderer,
            from_modman,
            from_inference_engine,
            from_appd
          ],
          nexus_tokens_clone,
        )
        .await;
      });

      // Start Warehouse
      let warehouse_store_clone = Arc::new(warehouse_store.clone());
      let warehouse_tokens = (CancellationToken::new(), CancellationToken::new());
      let warehouse_tokens_clone = warehouse_tokens.clone();
      let warehouse_handle = tokio::task::spawn(async move {
        warehouse_main(
          warehouse_store_clone.clone(),
          warehouse_user,
          warehouse_tokens_clone,
        )
        .await;
      });

      // Start Renderer
      let renderer_store_clone = Arc::new(renderer_store.clone());
      let renderer_tokens = (CancellationToken::new(), CancellationToken::new());
      let renderer_tokens_clone = renderer_tokens.clone();
      let renderer_handle = tokio::task::spawn(async move {
        renderer_main(
          renderer_store,
          renderer_user,
          renderer_tokens_clone,
        )
        .await;
      });

      // Start ModMan
      let modman_store_clone = Arc::new(modman_store.clone());
      let modman_tokens = (CancellationToken::new(), CancellationToken::new());
      let modman_tokens_clone = modman_tokens.clone();
      let modman_handle = tokio::task::spawn(async move {
        modman_main(
          modman_store,
          modman_user,
          modman_tokens_clone,
        )
        .await;
      });

      // Start InferenceEngine
      let inference_engine_store_clone = Arc::new(inference_engine_store.clone());
      let inference_engine_tokens = (CancellationToken::new(), CancellationToken::new());
      let inference_engine_tokens_clone = inference_engine_tokens.clone();
      let inference_engine_handle = tokio::task::spawn(async move {
        inference_engine_main(
          inference_engine_store_clone,
          inference_engine_user,
          inference_engine_tokens_clone,
        )
        .await;
      });

      // Start AppDaemon
      let appd_store_clone = Arc::new(appd_store.clone());
      let appd_tokens = (CancellationToken::new(), CancellationToken::new());
      let appd_tokens_clone = appd_tokens.clone();
      let appd_handle = tokio::task::spawn(async move {
        appd_main(
          appd_store,
          appd_user,
          appd_tokens_clone,
        )
        .await;
      });

      let cleanup_handle = tokio::task::spawn(async move {
        tokio::select! {
          _ = cancellation_token.cancelled() => {
            info!("Shutting down AppD...");
            appd_tokens.0.cancel();
            tokio::select! {
              _ = appd_tokens.1.cancelled() => {
                info!("Shutting down Inference Engine...");
                inference_engine_tokens.0.cancel();
                tokio::select! {
                  _ = inference_engine_tokens.1.cancelled() => {
                    info!("Shutting down ModMan...");
                    modman_tokens.0.cancel();
                    tokio::select! {
                      _ = modman_tokens.1.cancelled() => {
                        info!("Shutting down Renderer...");
                        renderer_tokens.0.cancel();
                        tokio::select! {
                          _ = renderer_tokens.1.cancelled() => {
                            info!("Shutting down Warehouse...");
                            warehouse_tokens.0.cancel();
                            tokio::select! {
                              _ = warehouse_tokens.1.cancelled() => {
                                info!("Shutting down Nexus...");
                                nexus_tokens.0.cancel();
                                tokio::select! {
                                  _ = nexus_tokens.1.cancelled() => {
                                    info!("Graceful shutdown successful!");
                                  }
                                }
                              }
                            }
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      });

      tokio::select! {_ = futures::future::join_all(vec![
        cleanup_handle,
        warehouse_handle,
        nexus_handle,
        renderer_handle,
        modman_handle,
        inference_engine_handle,
        appd_handle
      ]) => {
        info!("CloverHub Server has exited.");
      }}
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
