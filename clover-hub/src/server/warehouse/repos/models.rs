use crate::server::appd::models::BuildConfig;
#[cfg(feature = "core")]
use clover_hub_macros::ManifestCompile;
use os_path::OsPath;
use serde::{
  Deserialize,
  Serialize,
};
use simple_error::SimpleError;
use std::collections::HashMap;

// TODO: Define defaults via `Default` trait impl for enums that returns its none variant.
pub enum Resolution {
  /// Raw file content from a resolved `@import`. Should be deserialized prior to use!
  ImportedSingle((OsPath, String)),
  /// Multiple files were read from a resolved, glob `@import`, and the resolved glob is the key, while the value is the raw file content.
  /// Should be deserialized prior to use!
  ImportedMultiple((OsPath, HashMap<String, String>)),
  /// Every other case in which there was no `@import`.
  /// If there were other directives, they've been replaced with the correct value if provided in the ResolutionCtx.
  NoImport(String),
}
#[derive(Debug, Clone)]
pub struct ResolutionCtx {
  /// Used for the `@base` directive, if configured in the repo manifest, the base RFQDN for this repo.
  pub base: Option<String>,
  /// Used for the `@here` directive, should contain the FS path to the manifest file being currently parsed, **NOT** to the repo.
  pub here: OsPath,
  /// Used for the `@builtin` directive, should contain the built-in context for the current key (either CloverHub or CORE).
  pub builtin: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredSingleManifestEntry<T> {
  Some(T),
  ImportString(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalSingleManifestSpecEntry<T> {
  Some(T),
  ImportString(String),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalStrTHashMap<T> {
  Some(HashMap<String, T>),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredStrTHashMap<T>(pub HashMap<String, RequiredSingleManifestEntry<T>>);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalBoolean {
  Some(bool),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredBoolean {
  Some(bool),
  ImportString(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalListManifestSpecEntry<T> {
  Some(HashMap<String, RequiredSingleManifestEntry<T>>),
  ImportString(String),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequiredListManifestSpecEntry<T> {
  Some(HashMap<String, RequiredSingleManifestEntry<T>>),
  ImportString(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ManifestEntry<T> {
  RequiredSingleManifestEntry(RequiredSingleManifestEntry<T>),
  OptionalSingleManifestEntry(OptionalSingleManifestSpecEntry<T>),
  RequiredListManifestEntry(RequiredListManifestSpecEntry<T>),
  OptionalListManifestEntry(OptionalListManifestSpecEntry<T>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalStrStrHashMap {
  Some(HashMap<String, String>),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalStringListManifestSpecEntry {
  Some(HashMap<String, String>),
  ImportString(String),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum OptionalString {
  Some(String),
  #[default]
  None,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(untagged)]
pub enum Optional<T> {
  Some(T),
  ImportString(String),
  #[default]
  None,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RequiredString(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManifestSpec {
  pub name: Option<String>,
  pub version: String,
  pub base: Option<String>,
  #[serde(default)]
  pub directory: OptionalSingleManifestSpecEntry<RawDirectorySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawDirectorySpec {
  #[serde(default)]
  pub modules: OptionalListManifestSpecEntry<RawModuleSpec>,
  #[serde(default)]
  pub applications: OptionalListManifestSpecEntry<RawApplicationSpec>,
  #[cfg(feature = "core")]
  #[serde(rename = "gesture-packs", default)]
  pub gesture_packs: OptionalListManifestSpecEntry<RawGesturePackSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawModuleSpec {
  pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawApplicationSpec {
  pub name: String,
  pub version: String,
  #[serde(default)]
  pub intents: OptionalStringListManifestSpecEntry,
  #[serde(default)]
  pub containers: OptionalListManifestSpecEntry<RawContainerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawContainerSpec {
  #[serde(default)]
  pub interface: OptionalSingleManifestSpecEntry<bool>,
  #[serde(default)]
  pub build: OptionalSingleManifestSpecEntry<RawBuildConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawBuildConfig {
  /// Url to either container repo, or source git repo
  pub url: String,
  /// Optional repository creds
  #[serde(default)]
  pub creds: OptionalSingleManifestSpecEntry<RawRepoCreds>,
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
pub struct RawGesturePackSpec {
  pub name: Option<String>,
  #[serde(default)]
  pub gestures: OptionalListManifestSpecEntry<RawGestureSpec>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RawGestureSpec {
  RawStaticGestureSpec(RawStaticGestureSpec),
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawStaticGestureSpec {
  #[serde(rename = "static-url")]
  pub static_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
  #[serde(default)]
  pub name: OptionalString,
  pub version: RequiredString,
  #[serde(default)]
  pub base: OptionalString,
  #[serde(default)]
  pub directory: Optional<DirectorySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct DirectorySpec {
  #[serde(default)]
  pub modules: OptionalStrTHashMap<ModuleSpec>,
  #[serde(default)]
  pub applications: OptionalStrTHashMap<ApplicationSpec>,
  #[cfg(feature = "core")]
  #[serde(default)]
  pub gesture_packs: OptionalStrTHashMap<GesturePackSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct ModuleSpec {
  #[serde(default)]
  pub name: OptionalString,
}

#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct ApplicationSpec {
  pub name: RequiredString,
  pub version: RequiredString,
  #[serde(default)]
  pub intents: OptionalStrStrHashMap,
  #[serde(default)]
  pub containers: OptionalStrTHashMap<ContainerSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct ContainerSpec {
  #[serde(default)]
  pub interface: OptionalBoolean,
  #[serde(default)]
  pub build: Optional<BuildConfig>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct GesturePackSpec {
  #[serde(default)]
  pub name: OptionalString,
  #[serde(default)]
  pub gestures: OptionalStrTHashMap<GestureSpec>,
}

#[cfg(feature = "core")]
#[derive(Debug, Clone, Serialize, Deserialize, ManifestCompile)]
#[serde(untagged)]
pub enum GestureSpec {
  StaticGestureSpec(StaticGestureSpec),
}

#[cfg(feature = "core")]
#[derive(Default, Debug, Clone, Serialize, Deserialize, ManifestCompile)]
pub struct StaticGestureSpec {
  pub static_url: RequiredString,
}

pub trait ManifestCompilationFrom<T> {
  async fn compile(
    spec: T,
    resolution_ctx: ResolutionCtx,
    repo_dir_path: OsPath,
  ) -> Result<Self, SimpleError>
  where
    Self: Sized,
    T: for<'a> Deserialize<'a>;
}
