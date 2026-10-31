//! # All Alert Components

use serde::{
  Deserialize,
  Serialize,
};

pub mod camera;
pub mod microphone;
pub mod sensors;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlertSeverity {
  /// Nominal operational reports (e.g., touch contact made/broken, threshold reached)
  Info = 0,
  /// Approaching operational limits (e.g., driver temp elevated, minor tracking drift)
  Warning = 1,
  /// Safety or hardware failure requiring immediate host action (e.g., stall current spike, limit switch overtravel)
  Critical = 2,
  /// Bus or module emergency halt tripped locally
  Fatal = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlertPayload {
  /// Discrete digital triggers: button presses, capacitive touch pads, bumper switches
  DigitalContact { pin_index: u8, active: bool },

  /// Analog threshold crossing: battery voltage, thermistor, load cell
  ThresholdCrossed { reading_raw: i16, threshold: i16 },

  /// Actuator mechanical/electrical fault condition
  ActuatorFault {
    fault_kind: ActuatorFaultKind,
    measured_ma: u16,
  },

  /// IMU / Orientation event: tap, double-tap, freefall, tilt angle
  SpatialTrigger { event_flags: u8, magnitude: u16 },

  /// Audio / Acoustic detection: peak SPL or wake-word pulse
  AcousticPeak { peak_level_db: u8 },

  /// Custom/Generic raw payload (tag + arbitrary 3-byte data)
  Raw { tag: u8, data: [u8; 3] },
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ActuatorFaultKind {
  CurrentExceeded = 0,
  StallDetected = 1,
  ThermalLimit = 2,
  SensorDiscontinuity = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlertMessage {
  /// Originating AlertComponent ID on the local module
  pub component_id: u8,

  /// Urgency level (determines arbitration / CAN queue priority)
  pub severity: AlertSeverity,

  /// Local monotonic timestamp in milliseconds (Core 0 uptime mod 65536)
  pub timestamp_ms: u16,

  /// Event payload details
  pub payload: AlertPayload,
}
