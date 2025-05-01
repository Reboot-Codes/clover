use crate::server::modman::{
  components::{
    models::{
      CloverComponentTrait,
      ProxiedConnection,
      StreamingConnection,
    },
    video::VideoResolution,
  },
  models::GestureConfig,
};
use bevy::prelude::Component;
use decorum::Real;
use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;
use strum::VariantNames;

pub trait DisplayComponent {}

#[derive(Debug, Clone)]
pub struct PhysicalDisplayComponent {
  /// Required to initialize rendering pipeline for this display.
  pub resolution: VideoResolution,
  /// Required to calculate PPI (and optionally pixel aspect ratio).
  pub size: DisplaySize,
  pub connection: ConnectionType,
  /// If configured, the Entity ID of the VDisplay this physical display is a part of, like for a video wall.
  pub virtual_display: Option<u64>,
  /// If configured, how the display should react to gesture events. Will be overriden if this display is part of a virtual display.
  pub gesture_config: Option<GestureConfig>,
  pub internal: bool,
}

impl DisplayComponent for PhysicalDisplayComponent {}

#[derive(Debug, Clone, Component)]
pub struct VirtualDisplayComponent {
  /// Position and Component ID of the displays that make up this Virtual Display
  pub displays: HashMap<String, DisplayPosition>,
  pub resolution: VideoResolution,
  /// If configured, how the display should react to gesture events.
  pub gesture_config: Option<GestureConfig>,
  pub internal: bool,
}

impl DisplayComponent for VirtualDisplayComponent {}

#[derive(Debug, Clone)]
pub struct DisplayPosition {
  pub x: f64,
  pub y: f64,
}

#[derive(Debug, Clone)]
/// All measurements in CM. Only one is required for square pixels. Otherwise, use both values for rectangular pixels (e.g. 1:2 pixel aspect ratio vs 1:1).
pub struct DisplaySize {
  pub height: Option<Real<f64>>,
  pub width: Option<Real<f64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, VariantNames)]
pub enum ConnectionType {
  #[cfg(feature = "compositor")]
  #[serde(rename = "direct")]
  #[strum(serialize = "direct")]
  Direct(DirectConnection),
  #[serde(rename = "modman-proxy")]
  #[strum(serialize = "modman-proxy")]
  ModManProxy(ProxiedConnection),
  #[serde(rename = "stream")]
  #[strum(serialize = "stream")]
  Stream(StreamingConnection),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg(feature = "compositor")]
pub struct DirectConnection {
  /// Set to `@primary` to use the primary display configured by wayland/x11.
  pub display_id: String,
  pub windowed: bool,
}
