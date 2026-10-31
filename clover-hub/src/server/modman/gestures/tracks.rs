//! # Gesture Tracks & Playhead Runtime
//!
//! Defines data structures for gesture tracks, keyframe channels, animation curves,
//! and runtime playhead tracking used by the 60 Hz gesture command generator.
//!
//! ## Overview
//! - [`GesturePlayhead`]: Local playback state tracking elapsed playback time ($\Delta t \times \text{speed}$),
//!   intensity blending, and previously dispatched component setpoints for deadband comparison.
//! - [`TrackAnimation`]: Timeline animation metadata including duration, playback curves ([`CurveType`]),
//!   and behavior modes ([`AnimationType`]).
//! - [`Keyframe`]: A snapshot along the animation timeline mapping channel names (e.g. `x`, `y`)
//!   to either numeric targets or contextual tokens (such as `@initial`).

use serde::{
  Deserialize,
  Serialize,
};
use std::collections::HashMap;
use std::time::Duration;

use carbon_fiber::commands::models::components::ComponentTarget;

/// Top-level declaration for a bundle of gesture definitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GesturePack {
  /// Unique human-readable or RFQDN identifier for this gesture pack.
  pub name: String,
  /// Descriptive summary of the gestures included in this pack.
  #[serde(default)]
  pub description: String,
  /// Map of gesture identifiers to their corresponding definitions.
  pub gestures: HashMap<String, GestureDefinition>,
}

/// Definition of an individual gesture, mapping robot archetypes to target body areas.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GestureDefinition {
  /// Name or display label of this gesture.
  pub name: String,
  /// Nested mapping of archetype patterns (e.g. `*` or `com.reboot-codes.clover.CORE.*`)
  /// to body area target definitions.
  pub configs: HashMap<String, HashMap<String, AreaTarget>>,
}

/// Target setpoint specification for an individual body area.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AreaTarget {
  /// Static scalar angle or position target.
  Static(f64),
  /// Static map of channel names to position targets.
  StaticMap(HashMap<String, f64>),
  /// Dynamic keyframe-based track animation.
  Animation {
    /// Keyframe animation specification.
    #[serde(rename = "@animation")]
    animation: TrackAnimation,
  },
}

/// Track animation specification with duration, playback curve, and keyframe points.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackAnimation {
  /// Playback mode: `once`, `loop`, or `bounce`.
  #[serde(rename = "type", default = "default_anim_type")]
  pub anim_type: AnimationType,
  /// Interpolation curve: `linear`, `cubic`, or `ease_in_out`.
  #[serde(default = "default_curve_type")]
  pub curve: CurveType,
  /// Total duration of the animation track in milliseconds.
  pub duration: f64,
  /// Ordered list of keyframes along the animation timeline.
  pub keyframes: Vec<Keyframe>,
}

fn default_anim_type() -> AnimationType {
  AnimationType::Once
}

fn default_curve_type() -> CurveType {
  CurveType::Linear
}

/// Playback repetition behavior for an animation track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimationType {
  /// Ping-pong back and forth between start and end keyframes continuously.
  Bounce,
  /// Continuously repeat playback from beginning upon reaching the end.
  Loop,
  /// Play forward once and clamp at the final keyframe upon completion.
  Once,
}

/// Mathematical curve used to interpolate values between adjacent keyframes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CurveType {
  /// Constant-velocity linear interpolation ($f(\alpha) = \alpha$).
  Linear,
  /// Cubic easing ($f(\alpha) = \alpha^3$).
  Cubic,
  /// Smooth Hermite / S-curve ease-in-out ($f(\alpha) = \alpha^2 (3 - 2\alpha)$).
  EaseInOut,
}

/// An individual keyframe point defining values across one or more actuator channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe {
  /// Channel setpoint mapping (e.g. `"x" => 45.0`, `"y" => "@initial"`).
  #[serde(flatten)]
  pub channels: HashMap<String, KeyframeChannelValue>,
}

/// Value of a keyframe channel, either a concrete floating-point angle/position
/// or a contextual reference token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum KeyframeChannelValue {
  /// Concrete target setpoint in degrees or normalized units.
  Value(f64),
  /// Contextual reference string, such as `"@initial"` resolving to the component's rest position.
  Named(String),
}

/// Runtime playhead state tracked inside the 60 Hz generator loop for an active gesture.
#[derive(Debug, Clone)]
pub struct GesturePlayhead {
  /// Cumulative playback time elapsed since this gesture was started or looped.
  pub elapsed: Duration,
  /// Playback speed multiplier (1.0 = nominal realtime, 2.0 = double speed, 0.5 = half speed).
  pub speed: f64,
  /// Blend intensity weight clamped to `[0.0, 1.0]`.
  pub intensity: f64,
  /// True if this gesture is running on the background ambient priority stack.
  pub background: bool,
  /// Cache of previously dispatched actuator targets per component, used for deadband filtering.
  pub last_dispatched_targets: HashMap<String, ComponentTarget>,
}

impl GesturePlayhead {
  /// Creates a new [`GesturePlayhead`] initialized with zero elapsed time.
  pub fn new(speed: f64, intensity: f64, background: bool) -> Self {
    Self {
      elapsed: Duration::ZERO,
      speed,
      intensity,
      background,
      last_dispatched_targets: HashMap::new(),
    }
  }

  /// Advances playhead elapsed time by `delta`, scaled by the current playback [`Self::speed`].
  ///
  /// If [`Self::speed`] is negative or zero, elapsed time does not advance backwards.
  pub fn advance(&mut self, delta: Duration) {
    let scaled_delta_nanos = (delta.as_nanos() as f64 * self.speed.max(0.0)) as u64;
    self.elapsed += Duration::from_nanos(scaled_delta_nanos);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_playhead_advance_normal_and_scaled() {
    let mut playhead = GesturePlayhead::new(1.0, 1.0, false);
    let dt = Duration::from_millis(50);

    playhead.advance(dt);
    assert_eq!(playhead.elapsed, Duration::from_millis(50));

    // Double speed
    playhead.speed = 2.0;
    playhead.advance(dt);
    assert_eq!(playhead.elapsed, Duration::from_millis(150));

    // Zero speed (frozen)
    playhead.speed = 0.0;
    playhead.advance(dt);
    assert_eq!(playhead.elapsed, Duration::from_millis(150));

    // Negative speed clamped to 0
    playhead.speed = -1.0;
    playhead.advance(dt);
    assert_eq!(playhead.elapsed, Duration::from_millis(150));
  }
}
