use std::sync::Arc;

use crate::server::modman::{
  components::models::{
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioOutputComponent {
  pub connection: ConnectionType,
  pub gesture_config: Option<GestureConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioInputComponent {
  pub connection: ConnectionType,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct DirectConnection {
  /// ALSA device ID to use.
  pub device_id: String,
  #[serde(skip)]
  pub connection: Option<Arc<rodio::Device>>,
}

// Ugh, why does the debug macro not have something to skip with?
impl std::fmt::Debug for DirectConnection {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct DirectConnection {
      device_id: String,
    }

    let Self {
      device_id,
      connection: _,
    } = self;

    // per Chayim Friedman’s suggestion
    std::fmt::Debug::fmt(
      &DirectConnection {
        device_id: device_id.clone(),
      },
      f,
    )
  }
}
