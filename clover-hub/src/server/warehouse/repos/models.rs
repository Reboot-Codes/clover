use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use os_path::OsPath;
use crate::server::appd::models::BuildConfig;

// TODO: Define defaults via `Default` trait impl.

pub enum Resolution {
  /// Raw file content from a resolved `@import`. Should be deserialized prior to use!
  ImportedSingle((OsPath, String)),
  /// Multiple files were read from a resolved, glob `@import`, and the resolved glob is the key, while the value is the raw file content.
  /// Should be deserialized prior to use!
  ImportedMultiple((OsPath, HashMap<String, String>)),
  /// Every other case in which there was no `@import`.
  /// If there were other directives, they've been replaced with the correct value if provided in the ResolutionCtx.
  NoImport(String)
}
#[derive(Debug, Clone)]
pub struct ResolutionCtx {
  /// Used for the `@base` directive, if configured in the repo manifest, the base RFQDN for this repo.
  pub base: Option<String>,
  /// Used for the `@here` directive, should contain the FS path to the manifest file being currently parsed, **NOT** to the repo.
  pub here: OsPath
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredSingleManifestEntry<T> {
  Some(T),
  ImportString(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalSingleManifestSpecEntry<T> {
  Some(T),
  ImportString(String),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalStrTHashMap<T> {
  Some(HashMap<String, T>),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredStrTHashMap<T>(pub HashMap<String, RequiredSingleManifestEntry<T>>);

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalBoolean {
  Some(bool),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredBoolean {
  Some(bool),
  ImportString(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalListManifestSpecEntry<T> {
  Some(HashMap<String, RequiredSingleManifestEntry<T>>),
  ImportString(String),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredListManifestSpecEntry<T> {
  Some(HashMap<String, RequiredSingleManifestEntry<T>>),
  ImportString(String)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ManifestEntry<T> {
  RequiredSingleManifestEntry(RequiredSingleManifestEntry<T>),
  OptionalSingleManifestEntry(OptionalSingleManifestSpecEntry<T>),
  RequiredListManifestEntry(RequiredListManifestSpecEntry<T>),
  OptionalListManifestEntry(OptionalListManifestSpecEntry<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalStrStrHashMap {
  Some(HashMap<String, String>),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalStringListManifestSpecEntry {
  Some(HashMap<String, String>),
  ImportString(String),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OptionalString {
  Some(String),
  None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Optional<T> {
  Some(T),
  ImportString(String),
  None
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RequiredString(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestSpec {
  pub name: Option<String>,
  pub version: String,
  pub base: Option<String>,
  pub modules: OptionalListManifestSpecEntry<RawModuleSpec>,
  pub applications: OptionalListManifestSpecEntry<RawApplicationSpec>,
  #[cfg(feature = "core")]
  #[serde(rename = "expression-packs")]
  pub expression_packs: OptionalListManifestSpecEntry<RawExpressionPackSpec>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawModuleSpec {
  pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawApplicationSpec {
  pub name: String,
  pub version: String,
  pub intents: OptionalStringListManifestSpecEntry,
  pub containers: OptionalListManifestSpecEntry<RawContainerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContainerSpec {
  pub interface: OptionalSingleManifestSpecEntry<bool>,
  pub build: OptionalSingleManifestSpecEntry<RawBuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawBuildConfig {
  /// Url to either container repo, or source git repo
  pub url: String,
  /// Optional repository creds
  pub creds: OptionalSingleManifestSpecEntry<RawRepoCreds>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRepoCreds {
  /// Optional username if the login scheme requires it.
  pub username: Option<String>,
  /// Either API key or password.
  pub key: String,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExpressionPackSpec {
  pub name: Option<String>,
  pub expressions: OptionalListManifestSpecEntry<RawExpressionSpec>
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawExpressionSpec {
  RawStaticExpressionSpec(RawStaticExpressionSpec),
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStaticExpressionSpec {
  #[serde(rename = "static-url")]
  pub static_url: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
  pub name: OptionalString,
  pub version: RequiredString,
  pub base: OptionalString,
  pub modules: OptionalStrTHashMap<ModuleSpec>,
  pub applications: OptionalStrTHashMap<ApplicationSpec>,
  #[cfg(feature = "core")]
  pub expression_packs: OptionalStrTHashMap<ExpressionPackSpec>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSpec {
  pub name: OptionalString,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationSpec {
  pub name: RequiredString,
  pub version: RequiredString,
  pub intents: OptionalStrStrHashMap,
  pub containers: OptionalStrTHashMap<ContainerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
  pub interface: OptionalBoolean,
  pub build: Optional<BuildConfig>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionPackSpec {
  pub name: OptionalString,
  pub expressions: OptionalStrTHashMap<ExpressionSpec>
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ExpressionSpec {
  StaticExpressionSpec(StaticExpressionSpec),
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StaticExpressionSpec {
  pub static_url: RequiredString
}

pub trait ManifestCompilationFrom<T> {
  async fn compile(spec: T, resolution_ctx: ResolutionCtx) -> Result<Self, SimpleError> where Self: Sized, T: for<'a> Deserialize<'a>;
}
