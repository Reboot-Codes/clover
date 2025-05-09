use crate::server::modman::{
  components::models::ProxiedConnection,
  models::GestureConfig,
};
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;

#[derive(Debug, Clone)]
pub struct InputSensorComponent {
  pub connection: ConnectionType,
}

#[derive(Debug, Clone)]
pub struct OutputSensorComponent {
  pub gesture_config: Option<GestureConfig>,
  pub connection: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, VariantNames)]
pub enum ConnectionType {
  #[serde(rename = "modman-proxy")]
  #[strum(serialize = "modman-proxy")]
  ModManProxy(ProxiedConnection),
}
