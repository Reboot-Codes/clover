//! # Clover ModMan Data Structures
//!
//! [Modules](Module) are comprised of [Components](components::CloverComponent) and their [Metadata](components::CloverComponentMeta).
//!

use serde::{
  Deserialize,
  Serialize,
};

pub mod components;
pub mod config;
pub mod gestures;
pub mod modules;
pub mod store;

// TODO: Define defaults via `Default` trait impl.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortStatus {
  /// Available but unused.
  #[serde(rename = "available")]
  Available,
  /// Requested by $MODULE_ID, but the UART bus isn't initalized yet
  #[serde(rename = "requested")]
  Requested(String),
  /// Currently being used by $MODULE_ID
  #[serde(rename = "bound")]
  Bound(String),
  /// Unavailable, but still requested by $MODULE_ID
  #[serde(rename = "unavailable")]
  Unavailable(String),
}
