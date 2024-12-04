use std::num::NonZero;
use decorum::Real;
use crate::server::modman::busses::models::BusTypes;

#[derive(Debug, Clone)]
pub struct DisplayComponent {
  /// Required to initialize rendering pipeline for this display.
  pub resolution: DisplayResolution,
  /// Required to calculate PPI (and optionally pixel aspect ratio).
  pub size: DisplaySize,
  pub connection: ConnectionType
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
  pub width: Option<Real<f64>>
}

#[derive(Debug, Clone)]
pub enum ConnectionType {
  #[cfg(feature = "compositor")]
  GPUDirect(DirectConnection),
  ModManProxy(ProxiedConnection)
}

#[derive(Debug, Clone)]
#[cfg(feature = "compositor")]
pub struct DirectConnection {
  pub display_id: String,
  pub windowed: bool
}

#[derive(Debug, Clone)]
pub struct ProxiedConnection {
  pub bus_type: BusTypes,
  pub component_id: String
}
