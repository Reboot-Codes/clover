use nexus::utils;
use os_path::OsPath;
use sea_orm::DatabaseConnection;
use serde::{
  Deserialize,
  Serialize,
};
use std::{
  collections::HashMap,
  sync::Arc,
};

use crate::server::modman::models::ModManConfig;
use crate::server::renderer::models::RendererConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
  pub docker_daemon: String,
  pub repos: HashMap<String, RepoSpec>,
  #[serde(default)]
  pub data_dir: OsPath,
  #[serde(skip)]
  pub db: Option<Arc<DatabaseConnection>>,
  pub primary_api_key: String,
  /// Default gesture pack to use
  pub default_gesture_pack: String,
  pub modman: ModManConfig,
  pub renderer: RendererConfig,
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
      repos: HashMap::new(),
      data_dir: OsPath::new(),
      db: None,
      primary_api_key: utils::gen_api_key(),
      default_gesture_pack: "com.reboot-codes.clover.CORE.default".to_string(),
      modman: Default::default(),
      renderer: Default::default(),
    }
  }
}
