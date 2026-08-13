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

/// Enum used to track the status of ports that clover knows about and can use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortStatus {
  /// Available but unused.
  ///
  /// Ports get set to this state if they're allowed in config and then aren't bound to by a proxy bus,
  /// or are unrequested due to a module being disconnected.
  #[serde(rename = "available")]
  Available,
  /// Requested by $MODULE_ID, but the bus isn't initialized yet.
  ///
  /// Usually occurs during startup.
  #[serde(rename = "requested")]
  Requested(String),
  /// Currently being used by $MODULE_ID.
  #[serde(rename = "bound")]
  Bound(String),
  /// Couldn't be bound, but still requested by $MODULE_ID
  #[serde(rename = "unavailable")]
  Unavailable(String),
  /// Is currently bound by $MODULE_ID, but the module is being deinitialized.
  #[serde(rename = "unrequested")]
  Unrequested(String),
}
