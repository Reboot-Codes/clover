use crate::server::modman::components::models::CloverComponentTrait;
use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CameraType {
  Video4Linux(String),
  RTMP(String),
  RTSP(String),
}

#[derive(Debug, Clone)]
pub struct CameraComponent {
  camera_type: CameraType,
}

impl CloverComponentTrait for CameraComponent {}
