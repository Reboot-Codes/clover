pub mod config;
pub mod db;
pub mod ipc;
pub mod models;
pub mod repos;

use config::models::Config;
use ipc::handle_ipc_msg;
use log::{
  debug,
  error,
  info,
};
use models::WarehouseStore;
use nexus::user::NexusUser;
use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
};
use os_path::OsPath;
use repos::{
  download_repo_updates,
  update_repo_dir_structure,
};
use sea_orm::Database;
use simple_error::SimpleError;
use std::sync::Arc;
use tokio::fs;
use tokio::io::{
  AsyncReadExt,
  AsyncWriteExt,
};
use tokio_util::sync::CancellationToken;

// TODO: Move to snafu crate.
#[derive(Debug, Clone)]
pub enum Error {
  FailedToCreateDataDir { error: SimpleError },
  FailedToOpenConfigFile { error: SimpleError },
  FailedToCreateConfigFile { error: SimpleError },
  FailedToWriteToConfigFile { error: SimpleError },
  FailedToParseConfigFile { error: SimpleError },
  FailedToReadConfigFile { error: SimpleError },
  FailedToCreateReposDir { error: SimpleError },
  FailedToDownloadAndRegisterRepos { error: SimpleError },
  FailedToUpdateRepoDirectoryStructure { error: SimpleError },
}

pub async fn setup_warehouse(data_dir: String, store: Arc<WarehouseStore>) -> Result<(), Error> {
  let mut err = None;
  let mut data_dir_path = OsPath::new().join(data_dir.clone());
  data_dir_path.resolve();

  debug!("Setting up Warehouse in {}...", data_dir.clone());

  // Ensure that the data dir is valid.
  if !data_dir_path.exists() {
    match fs::create_dir_all(data_dir.clone()).await {
      Ok(_) => {
        if !data_dir_path.exists() {
          err = Some(Error::FailedToCreateDataDir {
            error: SimpleError::new("Check failed after creation!"),
          });
        }
      }
      Err(e) => {
        err = Some(Error::FailedToCreateDataDir {
          error: SimpleError::from(e),
        });
      }
    }
  }

  let warehouse_path = OsPath::from(data_dir.clone());

  // Read configuration and load defaults otherwise
  let config_file_path = warehouse_path.join("/config.json");
  match err {
    Some(_) => {}
    None => {
      if !config_file_path.exists() {
        match fs::File::create(config_file_path.clone()).await {
          Ok(mut file) => {
            match file
              .write_all(
                serde_json_lenient::to_string_pretty::<Config>(&Default::default())
                  .unwrap()
                  .as_bytes(),
              )
              .await
            {
              Ok(_) => {
                info!("Wrote default config!");
              }
              Err(e) => {
                err = Some(Error::FailedToWriteToConfigFile {
                  error: SimpleError::from(e),
                })
              }
            }
          }
          Err(e) => {
            err = Some(Error::FailedToCreateConfigFile {
              error: SimpleError::from(e),
            })
          }
        }
      }
    }
  }

  match err {
    Some(_) => {}
    None => {
      match fs::File::open(config_file_path).await {
        Ok(mut config_file) => {
          let mut contents = String::new();

          // TODO: Add repair option to fix broken config files.
          match config_file.read_to_string(&mut contents).await {
            Ok(_) => match serde_json_lenient::from_str::<Config>(&contents) {
              Ok(config_values) => {
                *store.config.lock().await = config_values;
                debug!("Loaded config!");
              }
              Err(e) => {
                err = Some(Error::FailedToParseConfigFile {
                  error: SimpleError::from(e),
                })
              }
            },
            Err(e) => {
              err = Some(Error::FailedToReadConfigFile {
                error: SimpleError::from(e),
              })
            }
          }

          std::mem::drop(config_file);
        }
        Err(e) => {
          err = Some(Error::FailedToOpenConfigFile {
            error: SimpleError::from(e),
          })
        }
      }
    }
  }

  debug!("Running repo dir bootstrap...");

  // Read repo data and load into applicable areas in the store.
  let repo_dir_path = warehouse_path.join("/repos/");
  match err {
    Some(_) => {}
    None => {
      if !repo_dir_path.exists() {
        match fs::create_dir(repo_dir_path.clone()).await {
          Ok(_) => {
            debug!("Created repo directory!");
          }
          Err(e) => {
            err = Some(Error::FailedToCreateReposDir {
              error: SimpleError::from(e),
            });
          }
        }
      }
    }
  }

  debug!("Updating repo dir structure...");

  match err {
    Some(_) => {}
    None => match update_repo_dir_structure(repo_dir_path.clone(), store.clone()).await {
      Ok(_) => {
        debug!("Updated repo dir structure, downloading repo updates...");

        match download_repo_updates(store.clone(), repo_dir_path.clone()).await {
          Ok(_) => {
            info!("Loaded {} repo(s)!", store.repos.lock().await.len());
          }
          Err(error) => {
            err = Some(Error::FailedToDownloadAndRegisterRepos { error: error.0 });
          }
        }
      }
      Err(error) => {
        err = Some(Error::FailedToUpdateRepoDirectoryStructure { error: error.0 });
      }
    },
  }

  store.clone().config.lock().await.data_dir = OsPath::new().join(data_dir.clone());

  drop(store);

  // Return any errors if they occurred
  match err {
    Some(e) => Err(e),
    None => Ok(()),
  }
}

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.warehouse".to_string(),
    pretty_name: "Clover: Warehouse".to_string(),
    api_keys: vec![ApiKeyWithoutUID {
      allowed_events_to: vec![
        "^nexus://com.reboot-codes.clover.warehouse(\\.(.*))*(\\/.*)*$".to_string(),
      ],
      allowed_events_from: vec![
        "^nexus://com.reboot-codes.clover.warehouse(\\.(.*))*(\\/.*)*$".to_string(),
      ],
      echo: false,
      proxy: false,
    }],
  }
}

pub async fn warehouse_main(
  store: Arc<WarehouseStore>,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Warehouse...");

  let db_raw = Database::connect(format!(
    "sqlite://{}?mode=rwc",
    store.config.lock().await.data_dir.join("/db.sqlite")
  ))
  .await;

  let (mut ipc_rx, nexus_recv_handle) = user.subscribe();
  let ipc_recv_token = cancellation_tokens.0.clone();
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
  let init_tokens = cancellation_tokens.clone();
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      match db_raw {
        Ok(db) => {
          init_store.config.lock().await.db = Some(Arc::new(db));
        }
        Err(e) => {
          error!("Failed to access db file, due to:\n{}", e);
          init_tokens.0.cancel();
        }
      }

      match init_user.send(
        &"nexus://com.reboot-codes.clover.warehouse/status".to_string(),
        &"finished-init".to_string(),
        &None,
      ) {
        Err(e) => {
          error!(
            "Error when letting peers know about completed init state: {}",
            e
          );
        }
        _ => {}
      }
    })
    .await;

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      nexus_recv_handle.abort();

      info!("Buttoning up storage...");
      // TODO: Lock db and clean up when done.
      debug!("Writing Config File...");
      let config = store.config.lock().await;
      match fs::File::open(config.data_dir.join("/config.json")).await {
        Ok(mut config_file) => {
          debug!("Config file opened!");

          match config_file
            .write_all(
              serde_json_lenient::to_string_pretty::<Config>(&config.clone() as &Config)
                .unwrap()
                .as_bytes(),
            )
            .await
          {
            Ok(_) => {
              debug!("Wrote config from store state!");
            }
            Err(e) => {
              error!("Failed to write config file");
              // TODO!
            }
          }

          std::mem::drop(config_file);
        }
        Err(e) => {
          error!("Unable to open config file");
          // TODO!
        }
      }

      std::mem::drop(config);
      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Warehouse has stopped!");
}
