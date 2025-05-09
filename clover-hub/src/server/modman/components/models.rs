use crate::server::modman::models::ModManStore;
use serde::{
  Deserialize,
  Serialize,
};
use std::sync::Arc;

/// All components must implement this trait, ensures standardization between component types, etc.
pub trait CloverComponentTrait: Sized {
  /// Should initalize the component in the store, and ensure that 2-way communication is setup.
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error>;
  /// Tells the component that it will not be used in the *near* future, and may even power it down.
  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error>;
}

/// Known and supported streaming protocols for Video and Audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamProtocol {
  RTMP,
  RTSP,
  HTTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConnection {
  /// IPv4 address to listen to connections from.
  pub v4_address: std::net::Ipv4Addr,
  /// Optional IPv6 address to listen to connections from.
  pub v6_address: Option<std::net::Ipv6Addr>,
  /// Port to bind to.
  pub port: u16,
  /// Either RTMP or RTSP
  pub protocol: StreamProtocol,
  pub path: Option<String>,
}

/// The Bus Proxy this component is connected through.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxiedConnection {
  /// Device ID
  Simulated(String),
  /// App ID and Device ID
  App(String, String),
  /// Bus path and Device ID
  #[cfg(feature = "can_fd")]
  CANFD(String, String),
  /// Bus path and Device ID
  #[cfg(feature = "can_2")]
  CAN2(String, String),
  /// Device ID
  #[cfg(feature = "bt_classic")]
  BT(String),
  /// Device ID
  #[cfg(feature = "bt_le")]
  BTLE(String),
  /// Bus path and Device ID
  #[cfg(feature = "spi")]
  SPI(String, String),
  /// Bus path and Device ID
  #[cfg(feature = "i2c")]
  I2C(String, String),
  /// Bus path
  #[cfg(feature = "uart")]
  UART(String),
}
