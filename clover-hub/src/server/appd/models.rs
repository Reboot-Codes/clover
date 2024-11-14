use std::collections::HashMap;

use bollard::container;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ContainerConfig {
  pub options: container::CreateContainerOptions<String>, 
  pub config: container::Config<String>,
  pub build: Option<BuildConfig>,
}

#[derive(Debug, Clone)]
pub struct BuildConfig {
  /// Url to either container repo, or source git repo
  pub url: String,
  /// Optional repository creds
  pub creds: Option<RepoCreds>
}

#[derive(Debug, Clone)]
pub struct RepoCreds {
  /// Optional username if the login scheme requires it.
  pub username: Option<String>,
  /// Either API key or password.
  pub key: String,
}
