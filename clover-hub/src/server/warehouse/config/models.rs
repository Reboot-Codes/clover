use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::server::warehouse::manifest::models::Manifest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub docker_daemon: String,
  pub repos: HashMap<String, Manifest>,
}

impl Default for Config {
  fn default() -> Self {
    Config { 
      docker_daemon: "/run/user/1000/podman/podman.sock".to_string(),
      repos: HashMap::new()
    }
  }
}
