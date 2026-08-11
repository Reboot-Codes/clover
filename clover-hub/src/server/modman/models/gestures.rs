use std::collections::HashMap;

use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;

#[derive(Serialize, Deserialize, Clone, Debug, VariantNames, PartialEq)]
#[serde(tag = "command")]
pub enum GestureState {
  #[serde(rename = "begin")]
  #[strum(serialize = "begin")]
  Begin {
    /// Multiplier over the value (y) axis of the gesture's smoothing curve.
    intensity: f64,
    /// Multiplier over the time (x) axis of the gesture's smoothing curve.
    speed: f64,
    /// Background gestures will be calculated before foreground gestures. Foreground gestures will always override background gestures.
    ///
    /// For example, you can have a background gesture that displays intermittent blinking, but you can start a temporary foreground gesture to show a different shape of the eyes to display an emotion.
    background: Option<bool>,
  },
  /// Resume calculation of module commands.
  ///
  /// This state is not saved at all.
  #[serde(rename = "unpause")]
  #[strum(serialize = "unpause")]
  UnPause,
  /// Temporarilly freeze the values that were already calculated for module commands as a result of this gesture.
  ///
  /// This state is not saved. Upon shutdown, if a foreground gesture is paused, the gesture will be removed from the stack upon shutdown; otherwise, background gestures will be saved in the `Begin` state to be automatically resumed upon startup.
  #[serde(rename = "pause")]
  #[strum(serialize = "end")]
  Pause,
  /// Removes the gesture from module command calculations entierly.
  #[serde(rename = "end")]
  #[strum(serialize = "end")]
  End,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct GestureCommand {
  pub state: GestureState,
  pub auto_switch: Option<f64>,
  pub is_from_system: bool,
  pub areas: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureConfig {
  /// The primary gesture pack to use for this component
  pub primary_gesture_pack: Option<String>,
  /// The default gesture for the component to use when idle.
  pub default_gesture: Option<String>,
  pub gesture_parameters: Option<GestureParameters>,
  /// Override gestures (use different pack, adjust gesture parameters, etc)
  pub gesture_overrides: HashMap<String, GestureOverride>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureOverride {
  /// Gesture id in `gesture_RFQDN@gesture_pack_RFQDN` format, where `@` and everything after can be ommitted to use the default gesture pack.
  pub gesture_preset_id: String,
  pub gesture_parameters: Option<GestureParameters>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureStates {
  /// Previous state of the gesture if we'd like to temporarily move back.
  pub prev_state: Option<GestureState>,
  /// Is this gesture paused?
  pub paused: bool,
  /// The current state of the gesture.
  pub current_state: GestureState,
  /// The next state to switch to (pre-loaded into Renderer and Modules if it loads resources), with a delay if we want to switch automatically.
  pub next_state: Option<(f64, GestureState)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Turns a gesture intensity (-1.0 to 1.0) into a value that the component can understand in accordance with the following formula
/// ```rust
/// let value = (
///   (
///     (
///       (intensity + 1) * (max - min)
///     ) + min
///   ) * multiplier
/// ) + offset
/// ```
pub struct GestureParameters {
  pub min: f64,
  pub max: f64,
  pub multiplier: f64,
  pub offset: f64,
}

impl GestureParameters {
  pub fn calculate_intensity(&self, intensity: f64) -> f64 {
    return ((((intensity + 1.0) * (self.max - self.min)) + self.min) * self.multiplier)
      + self.offset;
  }
}
