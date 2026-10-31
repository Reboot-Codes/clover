//! # All Command Components

pub mod display;
pub mod indicators;
pub mod movement;
pub mod speaker;

use serde::{
  Deserialize,
  Serialize,
};

use snafu::Snafu;

use crate::commands::models::components::movement::MovementTarget;

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
pub enum ComponentTarget {
  DisplayClear,
  DisplayBlitChunk {
    offset: u16,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    len: u8,
    data: [u8; 32],
  },
  IndicatorBinary(bool),
  IndicatorDimmer(u8),
  IndicatorRGB(u8, u8, u8),
  IndicatorNeoPixels {
    count: u8,
    colors: [(u8, u8, u8); 8],
  },
  SpeakerTone {
    frequency_hz: u16,
    volume_percent: u8,
  },
  SpeakerSilence,
  Movement(MovementTarget),
}

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

#[derive(Debug, Snafu, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComponentFaliure {
  #[snafu(display("Lookahead buffer full."))]
  LookaheadFull,
  #[snafu(display("Invalid component index or ID."))]
  InvalidComponent,
  #[snafu(display("Mismatched target variant for this component."))]
  MismatchedTarget,
  #[snafu(display("Peripheral driver write fault."))]
  DriverFault,
}

pub type ComponentFailure = ComponentFaliure;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ComponentTelemetry {
  Movement { position: [u32; 6] },
  DisplayReady,
  IndicatorReady,
  SpeakerReady,
  Raw([u8; 4]),
}

pub type ComponentFeedback = ComponentTelemetry;
