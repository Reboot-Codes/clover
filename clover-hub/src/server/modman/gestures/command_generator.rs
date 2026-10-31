//! # Gesture Command Generator & Dispatch
//!
//! Provides mathematical channel evaluation, archetype and body area pattern matching,
//! and serialized command dispatch to module hardware proxies over Zenoh.
//!
//! ## Pipeline
//! 1. **Pattern Matching**: Matches declared component body locations (e.g. `head.eyes.right`)
//!    against gesture pack target rules using [`wildcard_match`].
//! 2. **Manifest Lookup**: Resolves dynamic gesture specifications from parsed Warehouse
//!    manifests via [`find_gesture_in_manifests`].
//! 3. **Channel Evaluation**: Evaluates animated channels (`x`, `y`, etc.) at the current
//!    elapsed playhead time using [`evaluate_channel`]. Computes time phase, curve easing,
//!    and handles `@initial` reference tokens.
//! 4. **CAN Dispatch**: Formats [`GestureTargetCommand`] and serializes via Postcard to dispatch
//!    to module proxies over Zenoh RPC queries via [`dispatch_target`].

use std::{
  collections::HashMap,
  time::Duration,
};

use carbon_fiber::{
  commands::models::components::{
    movement::MovementTarget,
    ComponentTarget,
    GestureTargetCommand,
  },
  comms::models::{
    BusMessage,
    RawContentMessage,
  },
};
use tracing::warn;

use crate::server::{
  modman::MODULE_EVT_ID,
  warehouse::repos::models::{
    AnimationSpec,
    DynamicGestureSpec,
    GestureSpec,
    Manifest,
    Optional,
    OptionalStrTHashMap,
    OptionalString,
  },
};

/// Evaluates if a body location or archetype string matches a wildcard pattern.
///
/// Supported patterns:
/// - `*`: Matches any target string unconditionally.
/// - `prefix*`: Matches any target string beginning with `prefix`.
/// - `exact`: Matches target string if identical.
///
/// # Examples
/// ```rust,ignore
/// assert!(wildcard_match("head.eyes.*", "head.eyes.right"));
/// assert!(!wildcard_match("head.ears.*", "head.eyes.right"));
/// ```
pub fn wildcard_match(pattern: &str, target: &str) -> bool {
  if pattern == "*" || pattern == target {
    return true;
  }
  if let Some(prefix) = pattern.strip_suffix('*') {
    return target.starts_with(prefix);
  }
  false
}

/// Searches the compiled manifests in Warehouse for a matching dynamic gesture specification.
///
/// If `default_pack_id` is non-empty, prioritization is given to that pack.
/// Matches gesture IDs by exact match or suffix match (e.g. `blink` matching `.../blink`).
pub fn find_gesture_in_manifests(
  repos: &HashMap<String, Manifest>,
  default_pack_id: &str,
  gesture_id: &str,
) -> Option<DynamicGestureSpec> {
  for (_repo_id, manifest) in repos {
    if let Optional::Some(ref dir) = manifest.directory {
      #[cfg(feature = "core")]
      if let OptionalStrTHashMap::Some(ref packs) = dir.gesture_packs {
        for (pack_id, pack) in packs {
          if !default_pack_id.is_empty() && pack_id != default_pack_id && !pack_id.ends_with(default_pack_id) {
            continue;
          }
          if let OptionalStrTHashMap::Some(ref gestures) = pack.gestures {
            for (g_id, gesture_spec) in gestures {
              if g_id == gesture_id || g_id.ends_with(gesture_id) || gesture_id.ends_with(g_id) {
                if let GestureSpec::DynamicGestureSpec(ref dyn_spec) = gesture_spec {
                  return Some(dyn_spec.clone());
                }
              }
            }
          }
        }
      }
    }
  }
  None
}

/// Evaluates a single keyframe channel for a given elapsed time.
///
/// Returns a tuple `(interpolated_value, is_completed)`:
/// - `interpolated_value`: The calculated setpoint in degrees or normalized units at `elapsed_secs`.
/// - `is_completed`: True if the animation track is a `once` animation that has reached or exceeded its duration.
///
/// # Curves Supported
/// - `linear`: Direct linear interpolation.
/// - `cubic`: $\alpha^3$ acceleration easing.
/// - `ease_in_out` / `ease-in-out`: Smooth S-curve Hermite interpolation ($\alpha^2 (3 - 2\alpha)$).
///
/// # Playback Types
/// - `loop`: Modulo wrapping around total duration. Never reports completed.
/// - `bounce`: Ping-pong cycle over $2 \times \text{duration}$. Never reports completed.
/// - `once`: Clamps to duration and reports completed when `elapsed_secs >= duration`.
pub fn evaluate_channel(
  anim: &AnimationSpec,
  elapsed_secs: f64,
  channel: &str,
  initial_val: f64,
) -> (f64, bool) {
  let duration_ms = anim.duration.unwrap_or(1000.0);
  let duration_secs = (duration_ms / 1000.0).max(0.001);

  let anim_type_str = match &anim.anim_type {
    OptionalString::Some(ref s) => s.as_str(),
    _ => "once",
  };

  let (t_eff, completed) = match anim_type_str {
    "loop" => (elapsed_secs % duration_secs, false),
    "bounce" => {
      let cycle = duration_secs * 2.0;
      let m = elapsed_secs % cycle;
      if m <= duration_secs {
        (m, false)
      } else {
        (cycle - m, false)
      }
    }
    _ => {
      // "once" or fallback
      if elapsed_secs >= duration_secs {
        (duration_secs, true)
      } else {
        (elapsed_secs, false)
      }
    }
  };

  if let Some(serde_json::Value::Array(ref kf_arr)) = anim.keyframes {
    if kf_arr.is_empty() {
      return (initial_val, completed);
    }
    if kf_arr.len() == 1 {
      let val = parse_kf_val(&kf_arr[0], channel, initial_val);
      return (val, completed);
    }

    let n = (kf_arr.len() - 1) as f64;
    let progress = (t_eff / duration_secs).clamp(0.0, 1.0);
    let float_idx = progress * n;
    let idx = (float_idx.floor() as usize).min(kf_arr.len() - 2);
    let alpha_linear = float_idx - idx as f64;

    let curve_str = match &anim.curve {
      OptionalString::Some(ref s) => s.as_str(),
      _ => "linear",
    };

    let alpha = match curve_str {
      "ease_in_out" | "ease-in-out" => alpha_linear * alpha_linear * (3.0 - 2.0 * alpha_linear),
      "cubic" => alpha_linear * alpha_linear * alpha_linear,
      _ => alpha_linear,
    };

    let v0 = parse_kf_val(&kf_arr[idx], channel, initial_val);
    let v1 = parse_kf_val(&kf_arr[idx + 1], channel, initial_val);
    let interpolated = (1.0 - alpha) * v0 + alpha * v1;

    (interpolated, completed)
  } else {
    (initial_val, completed)
  }
}

/// Parses an individual keyframe channel value, resolving `@initial` to `initial_val`.
fn parse_kf_val(kf: &serde_json::Value, channel: &str, initial_val: f64) -> f64 {
  if let Some(val) = kf.get(channel) {
    if let Some(num) = val.as_f64() {
      return num;
    }
    if let Some(s) = val.as_str() {
      if s == "@initial" {
        return initial_val;
      }
      if let Ok(num) = s.parse::<f64>() {
        return num;
      }
    }
  }
  initial_val
}

/// Serializes and dispatches a [`GestureTargetCommand`] to a target module CAN proxy via Zenoh RPC.
///
/// Sends query to `com/reboot-codes/clover/hub/modman/modules/by-id/{module_id}/send`.
pub async fn dispatch_target(
  session: &zenoh::Session,
  module_id: &str,
  component_id: u8,
  target: ComponentTarget,
  transition_ms: u16,
) -> Result<(), anyhow::Error> {
  let key_expr = format!("{MODULE_EVT_ID}/modules/by-id/{module_id}/send");

  let cmd = GestureTargetCommand {
    component_id,
    transition_ms,
    flags: None,
    target,
  };

  let cmd_bytes = postcard::to_allocvec(&cmd)?;
  let raw = RawContentMessage {
    nonce: [0u8; 12],
    data: &cmd_bytes,
    hmac: [0u8; 32],
  };
  let bus_msg = BusMessage::Content(raw);
  let payload_json = serde_json::to_string(&bus_msg)?;

  match session.get(&key_expr).payload(payload_json).await {
    Ok(replies) => {
      let _ = tokio::time::timeout(Duration::from_millis(100), replies.recv_async()).await;
      Ok(())
    }
    Err(e) => {
      warn!("Failed to dispatch target to {module_id}/{component_id}: {e}");
      Ok(())
    }
  }
}

/// Checks if the difference between two [`ComponentTarget`] setpoints exceeds `threshold_mdeg`.
///
/// Returns `true` if any channel of the movement setpoint changes by more than `threshold_mdeg`
/// (typically 50 millidegrees / 0.05°), or if target types differ.
pub fn is_outside_deadband(
  previous: &ComponentTarget,
  current: &ComponentTarget,
  threshold_mdeg: u32,
) -> bool {
  match (previous, current) {
    (
      ComponentTarget::Movement(MovementTarget::OneDof(p1)),
      ComponentTarget::Movement(MovementTarget::OneDof(c1)),
    ) => p1.abs_diff(*c1) > threshold_mdeg,
    (
      ComponentTarget::Movement(MovementTarget::TwoDof(p1, p2)),
      ComponentTarget::Movement(MovementTarget::TwoDof(c1, c2)),
    ) => p1.abs_diff(*c1) > threshold_mdeg || p2.abs_diff(*c2) > threshold_mdeg,
    (
      ComponentTarget::Movement(MovementTarget::ThreeDof(p1, p2, p3)),
      ComponentTarget::Movement(MovementTarget::ThreeDof(c1, c2, c3)),
    ) => {
      p1.abs_diff(*c1) > threshold_mdeg
        || p2.abs_diff(*c2) > threshold_mdeg
        || p3.abs_diff(*c3) > threshold_mdeg
    }
    (
      ComponentTarget::Movement(MovementTarget::FourDof(p1, p2, p3, p4)),
      ComponentTarget::Movement(MovementTarget::FourDof(c1, c2, c3, c4)),
    ) => {
      p1.abs_diff(*c1) > threshold_mdeg
        || p2.abs_diff(*c2) > threshold_mdeg
        || p3.abs_diff(*c3) > threshold_mdeg
        || p4.abs_diff(*c4) > threshold_mdeg
    }
    (
      ComponentTarget::Movement(MovementTarget::FiveDof(p1, p2, p3, p4, p5)),
      ComponentTarget::Movement(MovementTarget::FiveDof(c1, c2, c3, c4, c5)),
    ) => {
      p1.abs_diff(*c1) > threshold_mdeg
        || p2.abs_diff(*c2) > threshold_mdeg
        || p3.abs_diff(*c3) > threshold_mdeg
        || p4.abs_diff(*c4) > threshold_mdeg
        || p5.abs_diff(*c5) > threshold_mdeg
    }
    (
      ComponentTarget::Movement(MovementTarget::SixDof(p1, p2, p3, p4, p5, p6)),
      ComponentTarget::Movement(MovementTarget::SixDof(c1, c2, c3, c4, c5, c6)),
    ) => {
      p1.abs_diff(*c1) > threshold_mdeg
        || p2.abs_diff(*c2) > threshold_mdeg
        || p3.abs_diff(*c3) > threshold_mdeg
        || p4.abs_diff(*c4) > threshold_mdeg
        || p5.abs_diff(*c5) > threshold_mdeg
        || p6.abs_diff(*c6) > threshold_mdeg
    }
    (
      ComponentTarget::IndicatorRGB(prev_red, prev_green, prev_blue),
      ComponentTarget::IndicatorRGB(curr_red, curr_green, curr_blue),
    ) => {
      prev_red.abs_diff(*curr_red) > 0
        || prev_green.abs_diff(*curr_green) > 0
        || prev_blue.abs_diff(*curr_blue) > 0
    }
    (
      ComponentTarget::IndicatorDimmer(prev_brightness),
      ComponentTarget::IndicatorDimmer(curr_brightness),
    ) => prev_brightness.abs_diff(*curr_brightness) > 0,
    (
      ComponentTarget::IndicatorBinary(prev_active),
      ComponentTarget::IndicatorBinary(curr_active),
    ) => prev_active != curr_active,
    (
      ComponentTarget::IndicatorNeoPixels {
        count: prev_count,
        colors: prev_colors,
      },
      ComponentTarget::IndicatorNeoPixels {
        count: curr_count,
        colors: curr_colors,
      },
    ) => prev_count != curr_count || prev_colors != curr_colors,
    (
      ComponentTarget::SpeakerTone {
        frequency_hz: prev_freq,
        volume_percent: prev_vol,
      },
      ComponentTarget::SpeakerTone {
        frequency_hz: curr_freq,
        volume_percent: curr_vol,
      },
    ) => prev_freq != curr_freq || prev_vol.abs_diff(*curr_vol) > 0,
    (
      ComponentTarget::SpeakerSilence,
      ComponentTarget::SpeakerSilence,
    ) => false,
    (
      ComponentTarget::DisplayClear,
      ComponentTarget::DisplayClear,
    ) => false,
    _ => true,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json::json;

  #[test]
  fn test_wildcard_matching() {
    assert!(wildcard_match("*", "head.eyes.right"));
    assert!(wildcard_match("head.eyes.*", "head.eyes.right"));
    assert!(wildcard_match("head.eyes.right", "head.eyes.right"));
    assert!(!wildcard_match("head.ears.*", "head.eyes.right"));
    assert!(!wildcard_match("head.eyes.left", "head.eyes.right"));
  }

  #[test]
  fn test_evaluate_channel_linear_and_once() {
    let anim = AnimationSpec {
      anim_type: OptionalString::Some("once".to_string()),
      curve: OptionalString::Some("linear".to_string()),
      duration: Some(1000.0), // 1.0s
      keyframes: Some(json!([
        { "x": 0.0, "y": 10.0 },
        { "x": 100.0, "y": 50.0 }
      ])),
    };

    // t = 0.0s
    let (x0, c0) = evaluate_channel(&anim, 0.0, "x", 0.0);
    assert!((x0 - 0.0).abs() < 1e-4);
    assert!(!c0);

    // t = 0.5s
    let (x_mid, c_mid) = evaluate_channel(&anim, 0.5, "x", 0.0);
    assert!((x_mid - 50.0).abs() < 1e-4);
    assert!(!c_mid);

    // t = 1.0s (completed)
    let (x_end, c_end) = evaluate_channel(&anim, 1.0, "x", 0.0);
    assert!((x_end - 100.0).abs() < 1e-4);
    assert!(c_end);

    // t = 1.5s (clamps at end)
    let (x_past, c_past) = evaluate_channel(&anim, 1.5, "x", 0.0);
    assert!((x_past - 100.0).abs() < 1e-4);
    assert!(c_past);
  }

  #[test]
  fn test_evaluate_channel_with_initial_keyword() {
    let anim = AnimationSpec {
      anim_type: OptionalString::Some("once".to_string()),
      curve: OptionalString::Some("linear".to_string()),
      duration: Some(1000.0),
      keyframes: Some(json!([
        { "x": "@initial" },
        { "x": 40.0 }
      ])),
    };

    let (x0, _) = evaluate_channel(&anim, 0.0, "x", 20.0);
    assert!((x0 - 20.0).abs() < 1e-4);

    let (x_mid, _) = evaluate_channel(&anim, 0.5, "x", 20.0);
    assert!((x_mid - 30.0).abs() < 1e-4);
  }

  #[test]
  fn test_evaluate_channel_loop_and_bounce() {
    let anim_loop = AnimationSpec {
      anim_type: OptionalString::Some("loop".to_string()),
      curve: OptionalString::Some("linear".to_string()),
      duration: Some(1000.0),
      keyframes: Some(json!([
        { "x": 0.0 },
        { "x": 10.0 }
      ])),
    };

    // At 1.25s in loop, t_eff = 0.25s -> x = 2.5, completed = false
    let (x_loop, c_loop) = evaluate_channel(&anim_loop, 1.25, "x", 0.0);
    assert!((x_loop - 2.5).abs() < 1e-4);
    assert!(!c_loop);

    let anim_bounce = AnimationSpec {
      anim_type: OptionalString::Some("bounce".to_string()),
      curve: OptionalString::Some("linear".to_string()),
      duration: Some(1000.0),
      keyframes: Some(json!([
        { "x": 0.0 },
        { "x": 10.0 }
      ])),
    };

    // Cycle is 2.0s: 0s-1s goes 0->10, 1s-2s goes 10->0
    // At 1.5s, cycle pos is 1.5s -> eff = 2.0 - 1.5 = 0.5s -> x = 5.0
    let (x_bounce, c_bounce) = evaluate_channel(&anim_bounce, 1.5, "x", 0.0);
    assert!((x_bounce - 5.0).abs() < 1e-4);
    assert!(!c_bounce);
  }

  #[test]
  fn test_is_outside_deadband_1_to_6_dof() {
    let t1 = ComponentTarget::Movement(MovementTarget::OneDof(1000));
    let t1_close = ComponentTarget::Movement(MovementTarget::OneDof(1030));
    let t1_far = ComponentTarget::Movement(MovementTarget::OneDof(1060));
    assert!(!is_outside_deadband(&t1, &t1_close, 50));
    assert!(is_outside_deadband(&t1, &t1_far, 50));

    let t6 = ComponentTarget::Movement(MovementTarget::SixDof(100, 200, 300, 400, 500, 600));
    let t6_close = ComponentTarget::Movement(MovementTarget::SixDof(120, 210, 310, 410, 510, 620));
    let t6_far_w = ComponentTarget::Movement(MovementTarget::SixDof(100, 200, 300, 400, 500, 660));
    assert!(!is_outside_deadband(&t6, &t6_close, 50));
    assert!(is_outside_deadband(&t6, &t6_far_w, 50));
  }

  #[test]
  fn test_is_outside_deadband_indicators_and_speakers() {
    let rgb1 = ComponentTarget::IndicatorRGB(255, 128, 64);
    let rgb2 = ComponentTarget::IndicatorRGB(255, 128, 64);
    let rgb3 = ComponentTarget::IndicatorRGB(255, 128, 65);
    assert!(!is_outside_deadband(&rgb1, &rgb2, 0));
    assert!(is_outside_deadband(&rgb1, &rgb3, 0));

    let bin_on = ComponentTarget::IndicatorBinary(true);
    let bin_off = ComponentTarget::IndicatorBinary(false);
    assert!(!is_outside_deadband(&bin_on, &bin_on, 0));
    assert!(is_outside_deadband(&bin_on, &bin_off, 0));

    let tone1 = ComponentTarget::SpeakerTone { frequency_hz: 440, volume_percent: 80 };
    let tone2 = ComponentTarget::SpeakerTone { frequency_hz: 440, volume_percent: 80 };
    let tone3 = ComponentTarget::SpeakerTone { frequency_hz: 880, volume_percent: 80 };
    let silence = ComponentTarget::SpeakerSilence;
    assert!(!is_outside_deadband(&tone1, &tone2, 0));
    assert!(is_outside_deadband(&tone1, &tone3, 0));
    assert!(is_outside_deadband(&tone1, &silence, 0));
    assert!(!is_outside_deadband(&silence, &silence, 0));
  }
}

