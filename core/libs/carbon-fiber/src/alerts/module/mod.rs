//! # Module Specific Alert Component Types and Code

pub mod components;
pub mod sensitivity;

use crate::alerts::models::components::AlertMessage;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModuleAlertContext {}

pub trait AlertComponent {
  /// Get the ID of the component
  fn id(&self) -> u8;

  /// Get the current status of the component if the status has sufficently changed.
  fn get_status(&self) -> Option<AlertMessage>;

  /// Advance clock and poll alert events.
  fn step(&mut self, dt_ms: u32) -> Option<AlertMessage> {
    let _ = dt_ms;
    self.get_status()
  }
}

pub trait AlertComponentBuilder {}
