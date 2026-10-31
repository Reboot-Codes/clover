//! # Indicator Track Evaluator
//!
//! Evaluates keyframe tracks for [`IndicatorComponent`] supporting RGB (`r`, `g`, `b`),
//! Dimmer (`brightness`), and Binary (`state`) setpoint generation.

use std::collections::HashMap;

use carbon_fiber::commands::models::components::ComponentTarget;

use crate::server::{
  modman::{
    components::indicators::models::IndicatorComponent,
    gestures::{
      command_generator::{evaluate_channel, find_gesture_in_manifests, wildcard_match},
      tracks::GesturePlayhead,
    },
    models::components::CloverComponentMeta,
  },
  warehouse::repos::models::{Manifest, Optional, OptionalStrTHashMap},
};

/// Evaluates active gesture tracks targeting an [`IndicatorComponent`].
pub fn evaluate_indicator(
  _indicator_comp: &IndicatorComponent,
  meta: &CloverComponentMeta,
  playheads: &HashMap<String, GesturePlayhead>,
  active_gesture_ids: &[(String, bool)],
  repos: &HashMap<String, Manifest>,
  default_pack: &str,
  completed_gestures: &mut Vec<String>,
) -> Option<(ComponentTarget, u16)> {
  let mut resolved_r = 0.0;
  let mut resolved_g = 0.0;
  let mut resolved_b = 0.0;
  let mut has_rgb = false;

  let mut resolved_brightness = 0.0;
  let mut has_brightness = false;

  let mut resolved_state = false;
  let mut has_binary = false;

  let mut transition_window_ms = 16u16;

  for (gesture_id, _) in active_gesture_ids {
    let playhead = match playheads.get(gesture_id) {
      Some(p) => p,
      None => continue,
    };

    let dyn_spec = match find_gesture_in_manifests(repos, default_pack, gesture_id) {
      Some(s) => s,
      None => continue,
    };

    if let OptionalStrTHashMap::Some(ref archetypes) = dyn_spec.configs {
      for (_archetype_pattern, archetype_target) in archetypes {
        if let OptionalStrTHashMap::Some(ref targets) = archetype_target.targets {
          for (area_pattern, area_target) in targets {
            if wildcard_match(area_pattern, &meta.location) {
              if let Optional::Some(ref anim) = area_target.animation {
                let elapsed = playhead.elapsed.as_secs_f64();
                let blend_intensity = playhead.intensity.clamp(0.0, 1.0);

                if let Some(serde_json::Value::Array(ref kf_arr)) = anim.keyframes {
                  if let Some(first_kf) = kf_arr.first() {
                    if first_kf.get("r").is_some()
                      || first_kf.get("g").is_some()
                      || first_kf.get("b").is_some()
                    {
                      let (r, c_r) = evaluate_channel(anim, elapsed, "r", 0.0);
                      let (g, c_g) = evaluate_channel(anim, elapsed, "g", 0.0);
                      let (b, c_b) = evaluate_channel(anim, elapsed, "b", 0.0);
                      if c_r || c_g || c_b {
                        completed_gestures.push(gesture_id.clone());
                      }
                      if !has_rgb {
                        resolved_r = r * blend_intensity;
                        resolved_g = g * blend_intensity;
                        resolved_b = b * blend_intensity;
                        has_rgb = true;
                      } else {
                        resolved_r += r * blend_intensity;
                        resolved_g += g * blend_intensity;
                        resolved_b += b * blend_intensity;
                      }
                    } else if first_kf.get("brightness").is_some() {
                      let (br, c_br) = evaluate_channel(anim, elapsed, "brightness", 0.0);
                      if c_br {
                        completed_gestures.push(gesture_id.clone());
                      }
                      if !has_brightness {
                        resolved_brightness = br * blend_intensity;
                        has_brightness = true;
                      } else {
                        resolved_brightness += br * blend_intensity;
                      }
                    } else if first_kf.get("state").is_some() {
                      let (st, c_st) = evaluate_channel(anim, elapsed, "state", 0.0);
                      if c_st {
                        completed_gestures.push(gesture_id.clone());
                      }
                      resolved_state = st > 0.5;
                      has_binary = true;
                    }
                  }
                }

                let duration = anim.duration.unwrap_or(1000.0);
                transition_window_ms = (duration.min(100.0) as u16).max(16);
              }
            }
          }
        }
      }
    }
  }

  if has_rgb {
    Some((
      ComponentTarget::IndicatorRGB(
        resolved_r.clamp(0.0, 255.0) as u8,
        resolved_g.clamp(0.0, 255.0) as u8,
        resolved_b.clamp(0.0, 255.0) as u8,
      ),
      transition_window_ms,
    ))
  } else if has_brightness {
    Some((
      ComponentTarget::IndicatorDimmer(resolved_brightness.clamp(0.0, 255.0) as u8),
      transition_window_ms,
    ))
  } else if has_binary {
    Some((
      ComponentTarget::IndicatorBinary(resolved_state),
      transition_window_ms,
    ))
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;
  use super::*;
  use crate::server::modman::components::indicators::models::ConnectionType;

  fn dummy_manifest(gesture_id: &str, target_location: &str, kf: serde_json::Value) -> Manifest {
    let raw_json = serde_json::json!({
      "version": "1.0.0",
      "directory": {
        "gesture_packs": {
          "default": {
            "gestures": {
              gesture_id: {
                "configs": {
                  "robot": {
                    "targets": {
                      target_location: {
                        "@animation": {
                          "duration": 1000.0,
                          "keyframes": kf
                        }
                      }
                    }
                  }
                }
              }
            }
          }
        }
      }
    });
    serde_json::from_value(raw_json).expect("valid manifest json")
  }

  #[test]
  fn test_evaluate_indicator_rgb() {
    let comp = IndicatorComponent {
      gesture_config: None,
      connection: ConnectionType::ModManProxy,
    };
    let meta = CloverComponentMeta {
      name: "rgb_led".to_string(),
      critical: false,
      location: "head.eyes.right".to_string(),
      base_gesture_parameters: HashMap::new(),
      internal: false,
    };

    let manifest = dummy_manifest(
      "blink_rgb",
      "head.eyes.*",
      serde_json::json!([
        { "r": 0.0, "g": 100.0, "b": 200.0 },
        { "r": 255.0, "g": 0.0, "b": 100.0 }
      ]),
    );
    let mut repos = HashMap::new();
    repos.insert("core".to_string(), manifest);

    let mut playheads = HashMap::new();
    playheads.insert(
      "blink_rgb".to_string(),
      GesturePlayhead {
        elapsed: Duration::from_millis(500),
        intensity: 1.0,
        speed: 1.0,
        background: false,
        last_dispatched_targets: HashMap::new(),
      },
    );

    let active_gestures = vec![("blink_rgb".to_string(), true)];
    let mut completed = Vec::new();

    let result = evaluate_indicator(
      &comp,
      &meta,
      &playheads,
      &active_gestures,
      &repos,
      "default",
      &mut completed,
    );

    assert!(result.is_some());
    let (target, _window) = result.unwrap();
    match target {
      ComponentTarget::IndicatorRGB(r, g, b) => {
        // Linear interpolation at 50%: r = 127/128, g = 50, b = 150
        assert!((r as i32 - 127).abs() <= 1);
        assert_eq!(g, 50);
        assert_eq!(b, 150);
      }
      other => panic!("Expected IndicatorRGB, got {other:?}"),
    }
  }

  #[test]
  fn test_evaluate_indicator_dimmer_and_binary() {
    let comp = IndicatorComponent {
      gesture_config: None,
      connection: ConnectionType::ModManProxy,
    };
    let meta = CloverComponentMeta {
      name: "dimmer_led".to_string(),
      critical: false,
      location: "head.antenna".to_string(),
      base_gesture_parameters: HashMap::new(),
      internal: false,
    };

    let manifest_dimmer = dummy_manifest(
      "fade",
      "head.antenna",
      serde_json::json!([
        { "brightness": 0.0 },
        { "brightness": 200.0 }
      ]),
    );
    let mut repos = HashMap::new();
    repos.insert("core".to_string(), manifest_dimmer);

    let mut playheads = HashMap::new();
    playheads.insert(
      "fade".to_string(),
      GesturePlayhead {
        elapsed: Duration::from_millis(500),
        intensity: 1.0,
        speed: 1.0,
        background: false,
        last_dispatched_targets: HashMap::new(),
      },
    );

    let active_gestures = vec![("fade".to_string(), true)];
    let mut completed = Vec::new();

    let result = evaluate_indicator(
      &comp,
      &meta,
      &playheads,
      &active_gestures,
      &repos,
      "default",
      &mut completed,
    );

    assert!(result.is_some());
    let (target, _window) = result.unwrap();
    match target {
      ComponentTarget::IndicatorDimmer(br) => {
        assert_eq!(br, 100);
      }
      other => panic!("Expected IndicatorDimmer, got {other:?}"),
    }

    // Binary test
    let manifest_bin = dummy_manifest(
      "toggle",
      "head.antenna",
      serde_json::json!([
        { "state": 0.0 },
        { "state": 1.0 }
      ]),
    );
    repos.insert("core".to_string(), manifest_bin);
    let active_bin = vec![("toggle".to_string(), true)];
    playheads.insert(
      "toggle".to_string(),
      GesturePlayhead {
        elapsed: Duration::from_millis(600), // > 0.5
        intensity: 1.0,
        speed: 1.0,
        background: false,
        last_dispatched_targets: HashMap::new(),
      },
    );

    let result_bin = evaluate_indicator(
      &comp,
      &meta,
      &playheads,
      &active_bin,
      &repos,
      "default",
      &mut completed,
    );
    assert_eq!(
      result_bin.unwrap().0,
      ComponentTarget::IndicatorBinary(true)
    );
  }
}
