use crate::server::modman::models::gestures::GestureConfig;
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorComponent {
  pub connection: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndicatorComponent {
  pub gesture_config: Option<GestureConfig>,
  pub connection: ConnectionType,
}

#[derive(Debug, Clone, Serialize, Deserialize, VariantNames)]
pub enum ConnectionType {
  #[serde(rename = "modman-proxy")]
  #[strum(serialize = "modman-proxy")]
  ModManProxy,
}
