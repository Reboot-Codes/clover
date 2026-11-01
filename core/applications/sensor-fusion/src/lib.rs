//! # SensorFusion
//!
//! Automatically turning sensor data into gesture events!
//!
//! SensorFusion is the default primary activity for external and internal displays.
//! It's also the primary example of a SystemUI plugin. SystemUI plugins are a special
//! type of App which execute directly in the SystemUI bevy ECS environment due to needing
//! direct GPU acceleration without any additional overhead.
//!
//! SensorFusion provides the standard CORE Facial Feature implementation which provides
//! emotion state guesses sourced from various Sensors and Cameras in a Clover setup. These
//! are then mixed into gestures which then go through the Gesture Command Generation process.
//!

use bevy::prelude::*;

pub struct SensorFusion;

impl Plugin for SensorFusion {
  fn build(&self, app: &mut App) {
    // TODO.
  }
}
