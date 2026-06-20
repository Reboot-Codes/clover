use crate::server::modman::models::ModManStore;
use serde::{
  Deserialize,
  Serialize,
};
use std::sync::Arc;

/// All components must implement this trait, ensures standardization between component types, etc.
pub trait CloverComponentTrait: Sized {
  /// Should initalize the component in the store, and ensure that 2-way communication is setup.
  fn init(
    &mut self,
    store: Arc<ModManStore>,
  ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
  /// Tells the component that it will not be used in the *near* future, and may even power it down.
  fn deinit(
    &mut self,
    store: Arc<ModManStore>,
  ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
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
