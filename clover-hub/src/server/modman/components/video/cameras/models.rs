use crate::server::modman::components::{
  models::{
    ProxiedConnection,
    StreamingConnection,
  },
  video::VideoResolution,
};
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;

#[derive(Serialize, Deserialize, Clone, Debug, VariantNames)]
pub enum ConnectionType {
  /// Path to device file, ex: `/dev/video0`
  #[serde(rename = "v4l")]
  #[strum(serialize = "v4l")]
  Video4Linux(String),
  #[serde(rename = "modman-proxy")]
  #[strum(serialize = "modman-proxy")]
  ModManProxy(ProxiedConnection),
  #[serde(rename = "stream")]
  #[strum(serialize = "stream")]
  Stream(StreamingConnection),
}

#[derive(Debug, Clone)]
pub struct CameraComponent {
  pub connection: ConnectionType,
  pub max_resolution: VideoResolution,
  /// Default resolution to scale to when accessing this video device, defaults to max_resolution if not set.
  pub default_resolution: Option<VideoResolution>,
  pub internal: bool,
}
