use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub docker_daemon: String,
  pub repos: HashMap<String, RepoSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoSpec {
  /// Friendly user-set name. (Can override the repo-name if the user so chooses.)
  pub name: Option<String>,
  /// The source for this repo, be it a local directory, or a remote git repository via HTTP(S) or SSH.
  pub src: String,
  /// The branch to clone and merge
  pub branch: String,
}

impl Default for Config {
  fn default() -> Self {
    Config { 
      docker_daemon: "/run/user/1000/podman/podman.sock".to_string(),
      repos: HashMap::new()
    }
  }
}
