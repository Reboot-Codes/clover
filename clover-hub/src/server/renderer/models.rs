//! # Renderer Models
//!
//! Configuration models and serialization schemas for the Clover graphics renderer.

use serde::{
  Deserialize,
  Serialize,
};

/// Configuration options for the Bevy-based graphical renderer.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RendererConfig {
  /// Whether to mirror proxied displays on dedicated desktop windows.
  #[serde(default)]
  pub mirror_displays: bool,
}
