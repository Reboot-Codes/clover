use crate::server::warehouse::repos::models::{
  Optional,
  OptionalString,
  RequiredString,
};
use bollard::container;
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildConfig {
  /// Url to either container repo, or source git repo
  pub url: RequiredString,
  /// Optional repository creds
  pub creds: Optional<RepoCreds>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCreds {
  /// Optional username if the login scheme requires it.
  pub username: OptionalString,
  /// Either API key or password.
  pub key: RequiredString,
}

pub struct AppDStore {
  pub applications: Arc<Mutex<HashMap<String, Application>>>,
}
