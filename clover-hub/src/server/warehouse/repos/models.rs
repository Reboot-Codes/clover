use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::server::appd::models::BuildConfig;

// TODO: Define defaults via `Default` trait impl.

// TODO:Write import detection and resolution functions.
/// Used when the manifest imports another manifest file instead of actually setting the value.
pub type ImportString = String;

// TODO: Add method to get value (if it's an import, resolve the import, or just return the value)
pub trait ManifestEntryTrait<T: std::fmt::Debug + std::clone::Clone>: std::fmt::Debug + std::clone::Clone {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequiredSingleManifestEntry<T> {
  Some(T),
  ImportString
}

impl<T: std::fmt::Debug + std::clone::Clone> ManifestEntryTrait<T> for RequiredSingleManifestEntry<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionalSingleManifestEntry<T> {
  Some(T),
  ImportString,
  None
}

impl<T: std::fmt::Debug + std::clone::Clone> ManifestEntryTrait<T> for OptionalSingleManifestEntry<T> {}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionalListManifestEntry<T> {
  Some(HashMap<String, ManifestEntry<T>>),
  ImportString,
  None
}

impl<T: std::fmt::Debug + std::clone::Clone> ManifestEntryTrait<T> for OptionalListManifestEntry<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequiredListManifestEntry<T> {
  Some(HashMap<String, ManifestEntry<T>>),
  ImportString
}

impl<T: std::fmt::Debug + std::clone::Clone> ManifestEntryTrait<T> for RequiredListManifestEntry<T> {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ManifestEntry<T> {
  RequiredSingleManifestEntry(T),
  OptionalSingleManifestEntry(T),
  RequiredListManifestEntry(T),
  OptionalListManifestEntry(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
  pub name: OptionalSingleManifestEntry<String>,
  pub version: RequiredSingleManifestEntry<String>,
  pub url: String,
  pub base: OptionalSingleManifestEntry<String>,
  pub modules: OptionalListManifestEntry<ModuleSpec>,
  pub applications: OptionalListManifestEntry<ApplicationSpec>,
  #[cfg(feature = "core")]
  pub expression_packs: OptionalListManifestEntry<ExpressionPackSpec>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
  pub id: String,
  pub name: OptionalSingleManifestEntry<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSpec {
  pub intents: OptionalListManifestEntry<String>,
  pub containers: OptionalListManifestEntry<ContainerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
  pub interface: OptionalSingleManifestEntry<bool>,
  pub build: OptionalSingleManifestEntry<BuildConfig>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionPackSpec {
  pub id: String,
  pub name: OptionalSingleManifestEntry<String>,
  pub expressions: RequiredListManifestEntry<ExpressionSpec>
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionSpec {
  StaticExpressionSpec,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticExpressionSpec {
  pub static_url: RequiredSingleManifestEntry<String>
}
