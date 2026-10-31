//! # Video Input Component
//!

use crate::{
  alerts::module::components::AlertComponentPipeline,
  alerts::module::sensitivity::Sensitivity,
};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CameraComponent<Dr: CameraDriver> {
  pub id: u8,
  pub sensitivity: Sensitivity,
  pub pipeline: AlertComponentPipeline,
  pub driver: Dr,
}

pub trait CameraDriver {}
