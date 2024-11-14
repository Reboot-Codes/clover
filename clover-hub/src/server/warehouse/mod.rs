pub mod config;
pub mod manifest;

use config::models::Config;
use os_path::OsPath;
use std::io::{Read, Write};
use std::sync::Arc;
use std::fs;
use log::{debug, info};
use simple_error::SimpleError;

use crate::server::evtbuzz::models::Store;

// TODO: Move to snafu crate.
#[derive(Debug, Clone)]
pub enum Error {
  FailedToCheckDataDir {
    error: SimpleError
  },
  FailedToCreateDataDir {
    error: SimpleError
  },
  FailedToCheckConfigFile {
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
  }
}

pub async fn setup_warehouse(data_dir: String, store: Arc<Store>) -> Result<(), Error> {
  let mut err: Option<Result<(), Error>> = None;

  debug!("Setting up Warehouse in {}...", data_dir.clone());

  // Ensure that the data dir is valid.
  match fs::exists(data_dir.clone()) {
    Ok(data_dir_exists) => {
      if !data_dir_exists {
        match fs::create_dir_all(data_dir.clone()) {
          Ok(_) => {
            match fs::exists(data_dir.clone()) {
              Ok(exists) => {
                if !exists {
                  err = Some(Err(Error::FailedToCreateDataDir { error: SimpleError::new("Check failed after creation!") }));
                }
              },
              Err(e) => {
                err = Some(Err(Error::FailedToCreateDataDir { error: SimpleError::from(e) }));
              }
            }
          },
          Err(e) => {
            err = Some(Err(Error::FailedToCreateDataDir { error: SimpleError::from(e) }));
          }
        }
      }
    },
    Err(e) => {
      err = Some(Err(Error::FailedToCheckDataDir { error: SimpleError::from(e) }));
    }
  }

  let warehouse_path = OsPath::from(data_dir.clone());

  // Read configuration and load defaults otherwise
  let config_file_path = warehouse_path.join("/config.jsonc");
  match err {
    Some(_) => {},
    None => {
      match fs::exists(config_file_path.clone()) {
        Ok(config_file_exists) => {
          if !config_file_exists {
            match fs::File::create(config_file_path.clone()) {
              Ok(mut file) => {
                match file.write_all(serde_jsonc::to_string::<Config>(&Default::default()).unwrap().as_bytes()) {
                  Ok(_) => {
                    info!("Wrote default config!");
                  },
                  Err(e) => {
                    err = Some(Err(Error::FailedToWriteToConfigFile { error: SimpleError::from(e) }))
                  }
                }
              },
              Err(e) => {
                err = Some(Err(Error::FailedToCreateConfigFile { error: SimpleError::from(e) }))
              }
            }
          }
        },
        Err(e) => {
          err = Some(Err(Error::FailedToCheckConfigFile { error: SimpleError::from(e) }));
        }
      }
    }
  }

  match err {
    Some(_) => {},
    None => {    
      match fs::File::open(config_file_path) {
        Ok(mut config_file) => {
          let mut contents = String::new();
          match config_file.read_to_string(&mut contents) {
            Ok(_) => {
              match serde_jsonc::from_str::<Config>(&contents) {
                Ok(config_values) => {
                  *store.config.lock().await = config_values;
                  debug!("Loaded config!");
                },
                Err(e) => {
                  err = Some(Err(Error::FailedToParseConfigFile { error: SimpleError::from(e) }))
                }
              }
            },
            Err(e) => {
              err = Some(Err(Error::FailedToReadConfigFile { error: SimpleError::from(e) }))
            }
          }
        },
        Err(e) => {
          err = Some(Err(Error::FailedToOpenConfigFile { error: SimpleError::from(e) }))
        }
      }
    }
  }

  // Read repo data and load into applicable areas in the store.


  match err {
    Some(e) => {
      match e {
        Err(error) => {
          Err(error)
        },
        Ok(_) => { Err(Error::FailedToCheckDataDir { error: SimpleError::new("This state shouldn't be possible (error set with an ok value!)") }) }
      }
    },
    None => { Ok(()) }
  }
}
