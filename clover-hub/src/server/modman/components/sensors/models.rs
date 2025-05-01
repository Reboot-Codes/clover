use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::GestureConfig,
};

#[derive(Debug, Clone)]
pub struct InputSensorComponent {
  pub internal: bool,
}

#[derive(Debug, Clone)]
pub struct OutputSensorComponent {
  pub gesture_config: Option<GestureConfig>,
  pub internal: bool,
}
