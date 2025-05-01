use crate::server::modman::{
  components::models::{
    CloverComponentTrait,
    ProxiedConnection,
    StreamingConnection,
  },
  models::GestureConfig,
};
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;

#[derive(Debug, Clone)]
pub struct AudioOutputComponent {
  pub connection: ConnectionType,
  pub gesture_config: Option<GestureConfig>,
  pub internal: bool,
}

#[derive(Debug, Clone)]
pub struct AudioInputComponent {
  pub connection: ConnectionType,
  pub internal: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, VariantNames)]
pub enum ConnectionType {
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
pub struct DirectConnection {
  /// ALSA device ID to use.
  pub device_id: String,
}
