use crate::server::modman::{
  busses::models::BusTypes,
  models::ModManStore,
};
use serde::{
  Deserialize,
  Serialize,
};
use std::sync::Arc;

pub trait CloverComponentTrait: Sized {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error>;
  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error>;
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxiedConnection {
  pub bus_type: BusTypes,
  pub component_id: String,
}
