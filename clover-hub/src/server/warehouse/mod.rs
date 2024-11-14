pub mod config;
pub mod manifest;

use std::sync::Arc;
use std::fs;
use log::debug;
use simple_error::SimpleError;

use crate::server::evtbuzz::models::Store;

#[derive(Debug, Clone)]
pub enum Error {
  FailedToCheckDataDir {
    error: SimpleError
  },
  FailedToCreateDataDir {
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

  // Read configuration and load defaults otherwise
  

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
