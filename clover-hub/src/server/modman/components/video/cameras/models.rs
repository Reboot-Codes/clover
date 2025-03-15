use crate::server::modman::components::models::CloverComponent;
use serde::{
  Deserialize,
  Serialize,
};

#[derive(Serialize, Deserialize)]
pub enum CameraType {
  Video4Linux(String),
  RTMP(String),
  RTSP(String),
}

pub struct CameraComponent {
  camera_type: CameraType,
}

impl CloverComponent for CameraComponent {}
