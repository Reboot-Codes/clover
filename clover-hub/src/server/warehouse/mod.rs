pub mod config;
pub mod repos;

use config::models::Config;
use repos::{download_repo_updates, update_repo_dir_structure};
use os_path::OsPath;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use tokio::fs;
use log::{debug, info};
use simple_error::SimpleError;
use crate::server::evtbuzz::models::Store;

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
      match update_repo_dir_structure(store.clone()).await {
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

  drop(store);

  // Return any errors if they occurred
  match err {
    Some(e) => {
      Err(e)
    },
    None => { Ok(()) }
  }
}
