pub mod audio;
pub mod displays;
pub mod indicators;
pub mod movement;

use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GestureTargetCommand {
  /// Target component to adjust
  pub component_id: u8,
  /// Time window in milliseconds for the component to transition to this state.
  /// Used by actuators for interpolation, displays for crossfades, etc.
  pub transition_ms: u16,
  /// Optional operational or safety modifiers.
  pub flags: Option<GestureFlags>,
  /// Component-specific setpoint payload.
  pub target: ComponentTarget,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComponentTarget {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GestureFlags {
  Normal,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GestureTargetSuccessReply {
  pub telemetry: Option<ComponentTelemetry>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct GestureTargetFailReply {
  pub reason: ComponentFaliure,
  pub telemetry: Option<ComponentTelemetry>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComponentFaliure {
  LookaheadFull,
  InvalidComponent,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComponentTelemetry {}
