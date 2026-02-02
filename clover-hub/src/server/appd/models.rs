use crate::server::warehouse::repos::models::ManifestCompilationFrom;
use crate::server::warehouse::repos::models::RawBuildConfig;
use crate::server::warehouse::repos::models::RawRepoCreds;
use crate::server::warehouse::repos::models::ResolutionCtx;
use crate::server::warehouse::{
  config::models::Config,
  repos::models::{
    Optional,
    OptionalString,
    RequiredString,
  },
};
use bollard::container;
use clover_hub_macros::ManifestCompile;
use os_path::OsPath;
use serde::{
  Deserialize,
  Serialize,
};
use simple_error::SimpleError;
use std::{
  collections::HashMap,
  sync::Arc,
};
use tokio::sync::Mutex;

// TODO: Define defaults via `Default` trait impl.

// Code taken from the bollard crate to add deserialization via derive macro.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(remote = "container::CreateContainerOptions")]
pub struct CreateContainerOptionsDef<T>
where
  T: Into<String> + Serialize,
{
  /// Assign the specified name to the container.
  pub name: T,

  /// The platform to use for the container.
  /// Added in API v1.41.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub platform: Option<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
  /// Unique application ID
  pub id: String,
  /// Unique application sem-ver for this image
  pub version: String,
  /// Pretty name for the Application.
  pub name: String,
  /// Container specs
  pub containers: HashMap<String, ContainerConfig>,
  /// Is this application initialized by ModMan yet?
  pub initialized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
  #[serde(with = "CreateContainerOptionsDef")]
  pub options: container::CreateContainerOptions<String>,
  pub config: container::Config<String>,
  pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct BuildConfig {
  /// Url to either container repo, or source git repo
  pub url: RequiredString,
  /// Optional repository creds
  pub creds: Optional<RepoCreds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct RepoCreds {
  /// Optional username if the login scheme requires it.
  pub username: OptionalString,
  /// Either API key or password.
  pub key: RequiredString,
}

#[derive(Debug, Clone)]
pub struct AppDStore {
  pub applications: Arc<Mutex<HashMap<String, Application>>>,
  pub config: Arc<Mutex<Config>>,
}

impl AppDStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      None => Arc::new(Mutex::new(Config::default())),
    };

    AppDStore {
      applications: Arc::new(Mutex::new(HashMap::new())),
      config,
    }
  }
}
