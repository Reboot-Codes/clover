use super::{
  config::models::Config,
  repos::models::Manifest,
};
use std::{
  collections::HashMap,
  sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct WarehouseStore {
  pub repos: Arc<Mutex<HashMap<String, Manifest>>>,
  pub config: Arc<Mutex<Config>>,
}

impl WarehouseStore {
  pub fn new(optional_config: Option<Config>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      None => Config::default(),
    };

    WarehouseStore {
      repos: Arc::new(Mutex::new(HashMap::new())),
      config: Arc::new(Mutex::new(config)),
    }
  }
}
