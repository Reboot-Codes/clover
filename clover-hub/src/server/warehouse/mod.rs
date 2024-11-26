pub mod config;
pub mod repos;
pub mod db;

use config::models::Config;
use repos::{download_repo_updates, update_repo_dir_structure};
use os_path::OsPath;
use sea_orm::Database;
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}};
use tokio_util::sync::CancellationToken;
use url::Url;
use std::sync::Arc;
use tokio::fs;
use log::{debug, error, info};
use simple_error::SimpleError;
use crate::{server::evtbuzz::models::{IPCMessageWithId, Store}, utils::send_ipc_message};

use super::evtbuzz::models::CoreUserConfig;

// TODO: Move to snafu crate.
#[derive(Debug, Clone)]
pub enum Error {
  FailedToCreateDataDir {
    error: SimpleError
  },
  FailedToOpenConfigFile {
    error: SimpleError
  },
  FailedToCreateConfigFile {
    error: SimpleError
  },
  FailedToWriteToConfigFile {
    error: SimpleError
  },
  FailedToParseConfigFile {
    error: SimpleError
  },
  FailedToReadConfigFile {
    error: SimpleError
  },
  FailedToCreateReposDir {
    error: SimpleError
  },
  FailedToDownloadAndRegisterRepos {
    error: SimpleError
  },
  FailedToUpdateRepoDirectoryStructure {
    error: SimpleError
  }
}

pub async fn setup_warehouse(data_dir: String, store: Arc<Store>) -> Result<(), Error> {
  let mut err = None;
  let mut data_dir_path = OsPath::new().join(data_dir.clone());
  data_dir_path.resolve();

  debug!("Setting up Warehouse in {}...", data_dir.clone());

  // Ensure that the data dir is valid.
  if !data_dir_path.exists() {
    match fs::create_dir_all(data_dir.clone()).await {
      Ok(_) => {
        if !data_dir_path.exists() {
          err = Some(Error::FailedToCreateDataDir { error: SimpleError::new("Check failed after creation!") });
        }
      },
      Err(e) => {
        err = Some(Error::FailedToCreateDataDir { error: SimpleError::from(e) });
      }
    }
  }

  let warehouse_path = OsPath::from(data_dir.clone());

  // Read configuration and load defaults otherwise
  let config_file_path = warehouse_path.join("/config.jsonc");
  match err {
    Some(_) => {},
    None => {
      if !config_file_path.exists() {
        match fs::File::create(config_file_path.clone()).await {
          Ok(mut file) => {
            match file.write_all(serde_jsonc::to_string::<Config>(&Default::default()).unwrap().as_bytes()).await {
              Ok(_) => {
                info!("Wrote default config!");
              },
              Err(e) => {
                err = Some(Error::FailedToWriteToConfigFile { error: SimpleError::from(e) })
              }
            }
          },
          Err(e) => {
            err = Some(Error::FailedToCreateConfigFile { error: SimpleError::from(e) })
          }
        }
      }
    }
  }

  match err {
    Some(_) => {},
    None => {    
      match fs::File::open(config_file_path).await {
        Ok(mut config_file) => {
          let mut contents = String::new();

          // TODO: Add repair option to fix broken config files.
          match config_file.read_to_string(&mut contents).await {
            Ok(_) => {
              match serde_jsonc::from_str::<Config>(&contents) {
                Ok(config_values) => {
                  *store.config.lock().await = config_values;
                  debug!("Loaded config!");
                },
                Err(e) => {
                  err = Some(Error::FailedToParseConfigFile { error: SimpleError::from(e) })
                }
              }
            },
            Err(e) => {
              err = Some(Error::FailedToReadConfigFile { error: SimpleError::from(e) })
            }
          }
        },
        Err(e) => {
          err = Some(Error::FailedToOpenConfigFile { error: SimpleError::from(e) })
        }
      }
    }
  }

  debug!("Running repo dir bootstrap...");

  // Read repo data and load into applicable areas in the store.
  let repo_dir_path = warehouse_path.join("/repos/");
  match err {
    Some(_) => {},
    None => {
      if !repo_dir_path.exists() {
        match fs::create_dir(repo_dir_path.clone()).await {
          Ok(_) => {
            debug!("Created repo directory!");
          },
          Err(e) => {
            err = Some(Error::FailedToCreateReposDir { error: SimpleError::from(e) });
          }
        }
      }
    }
  }

  debug!("Updating repo dir structure...");

  match err {
    Some(_) => {},
    None => {
      match update_repo_dir_structure(repo_dir_path.clone(), store.clone()).await {
        Ok(_) => {
          debug!("Updated repo dir structure, downloading repo updates...");

          match download_repo_updates(store.clone(), repo_dir_path.clone()).await {
            Ok(_) => {
              info!("Loaded {} repo(s)!", store.repos.lock().await.len());  
            },
            Err(error) => {
              err = Some(Error::FailedToDownloadAndRegisterRepos { error: error.0 });
            }
          }
        },
        Err(error) => {
          err = Some(Error::FailedToUpdateRepoDirectoryStructure { error: error.0 });
        }
      }
    }
  }

  store.clone().config.lock().await.data_dir = OsPath::new().join(data_dir.clone());

  drop(store);

  // Return any errors if they occurred
  match err {
    Some(e) => Err(e),
    None => Ok(())
  }
}

pub async fn warehouse_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_tokens: (CancellationToken, CancellationToken)
) {
  info!("Starting Warehouse...");

  let db_raw = Database::connect(format!("sqlite://{}?mode=rwc", store.config.lock().await.data_dir.join("/db.sqlite"))).await;

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user_config.clone());
  let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
  let init_tokens = cancellation_tokens.clone();
  cancellation_tokens.0.run_until_cancelled(async move {
    match db_raw {
      Ok(db) => {
        init_store.config.lock().await.db = Some(Arc::new(db));
      },
      Err(e) => {
        error!("Failed to access db file, due to:\n{}", e);
        init_tokens.0.cancel();
      }
    }

    let _ = send_ipc_message(
      &init_store, 
      &init_user, 
      Arc::new(init_from_tx), 
      "clover://warehouse.clover.reboot-codes.com/status".to_string(), 
      "finished-init".to_string()
    ).await;
  }).await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Some(msg) = ipc_rx.recv().await {
          let kind = Url::parse(&msg.kind.clone()).unwrap();

          // Verify that we care about this event.
          if kind.host().unwrap() == url::Host::Domain("warehouse.clover.reboot-codes.com") {
            debug!("Processing: {}", msg.kind.clone());
          }
        }
      } => {}
    }
  });

  let ipc_trans_token = cancellation_tokens.0.clone();
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

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_trans_handle.abort();

      info!("Buttoning up storage...");
      // TODO: Lock db and clean up when done.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Warehouse has stopped!");
}
