use crate::server::modman::{
  busses::models::BusTypes,
  components::models::CloverComponent,
};
use bevy::prelude::Component;
use decorum::Real;
use std::{
  collections::HashMap,
  num::NonZero,
};

#[derive(Debug, Clone)]
pub struct DisplayComponent {
  /// Required to initialize rendering pipeline for this display.
  pub resolution: DisplayResolution,
  /// Required to calculate PPI (and optionally pixel aspect ratio).
  pub size: DisplaySize,
  pub connection: ConnectionType,
  // If configured, the Entity ID of the VDisplay this physical display is a part of.
  pub virtual_display: Option<u64>,
}

impl CloverComponent for DisplayComponent {}

#[derive(Debug, Clone, Component)]
pub struct VirtualDisplayComponent {
  /// Position and Component ID of the displays that make up this Virtual Display
  pub displays: HashMap<String, DisplayPosition>,
  pub resolution: DisplayResolution,
}

impl CloverComponent for VirtualDisplayComponent {}

#[derive(Debug, Clone)]
pub struct DisplayPosition {
  pub x: f64,
  pub y: f64,
}

#[derive(Debug, Clone)]
pub struct DisplayResolution {
  pub height: NonZero<u64>,
  pub width: NonZero<u64>,
}

#[derive(Debug, Clone)]
/// All measurements in CM. Only one is required for square pixels. Otherwise, use both values for rectangular pixels (e.g. 1:2 pixel aspect ratio vs 1:1).
pub struct DisplaySize {
  pub height: Option<Real<f64>>,
  pub width: Option<Real<f64>>,
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
  #[cfg(feature = "compositor")]
  GPUDirect(DirectConnection),
  ModManProxy(ProxiedConnection),
}

#[derive(Debug, Clone)]
#[cfg(feature = "compositor")]
pub struct DirectConnection {
  /// Set to `@primary` to use the primary display configured by wayland.
  pub display_id: String,
  pub windowed: bool,
}

#[derive(Debug, Clone)]
pub struct ProxiedConnection {
  pub bus_type: BusTypes,
  pub component_id: String,
}
