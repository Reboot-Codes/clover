/// Default sensitvity definitions.
///
/// Note that the sensitvity deffinition used for an
/// Alert Component will strongly affect the bandwidth
/// estimate for the module.
///
/// Realtime Sensitivity should be used sparingly as it
/// can quickly saturate the bus.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Sensitivity {
  /// Send updates at the default realtime update interval for
  /// the MCU, regardless if there have been any changes.
  ///
  /// This setting will use a ***lot*** of bandwidth, so be mindful
  /// about using it on Modules which have more than about 1-3 components.
  Realtime,
  /// Send updates on any change in the component's state at all.
  /// This will use a lot of bandwidth on components which do not have
  /// smoothing circuitry attached.
  High,
  /// Sensible default sensitivity, will only send alerts after being
  /// reasonably sure that the component should Alert with a little extra
  /// padding and biasing over time to improve performance.
  Medium,
  /// Requires a large amount of change to make the module Alert. Binary switches
  /// should default to low sensitivity since their data rates are just `1` or `0`
  /// and will probably not send much data over the bus.
  Low,
  /// Makes this component act like a command module, but only in terms of sending
  /// telemetry when asked. Useful for things like barometric pressure, altimeters, etc.
  Polled,
}

impl Sensitivity {
  pub const fn bandwidth_multiplier(&self) -> f32 {
    match self {
      Sensitivity::Realtime => 1.0,
      Sensitivity::High => 0.65,
      Sensitivity::Medium => 0.2,
      Sensitivity::Low => 0.15,
      Sensitivity::Polled => 0.05,
    }
  }
}
