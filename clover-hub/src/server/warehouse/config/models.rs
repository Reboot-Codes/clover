use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub docker_daemon: String,
}

impl Default for Config {
  fn default() -> Self {
    Config { 
      docker_daemon: "/run/user/1000/podman/podman.sock".to_string()
    }
  }
}
