pub mod camera;
pub mod microphone;
pub mod sensor;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlertComponentPipeline {}

impl AlertComponentPipeline {
  pub fn new() -> Self {
    Self {}
  }
}
