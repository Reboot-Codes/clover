//! # Movement Track Evaluator
//!
//! Evaluates multi-channel keyframe tracks for [`MovementComponent`] supporting 1 through 6
//! degrees of freedom (`x`, `y`, `z`, `u`, `v`, `w`), intensity blending, and millidegree setpoints.

use std::collections::HashMap;

use carbon_fiber::commands::models::components::{
  movement::MovementTarget,
  ComponentTarget,
};

use crate::server::{
  modman::{
    components::movement::models::{DegreesOfFreedom, MovementComponent},
    gestures::{
      command_generator::{evaluate_channel, find_gesture_in_manifests, wildcard_match},
      tracks::GesturePlayhead,
    },
    models::components::CloverComponentMeta,
  },
  warehouse::repos::models::{Manifest, Optional, OptionalStrTHashMap},
};

const CHANNELS: [&str; 6] = ["x", "y", "z", "u", "v", "w"];

/// Evaluates active gesture tracks targeting a [`MovementComponent`].
pub fn evaluate_movement(
  movement_comp: &MovementComponent,
  meta: &CloverComponentMeta,
  playheads: &HashMap<String, GesturePlayhead>,
  active_gesture_ids: &[(String, bool)],
  repos: &HashMap<String, Manifest>,
  default_pack: &str,
  completed_gestures: &mut Vec<String>,
) -> Option<(ComponentTarget, u16)> {
  let (dof_count, initial_positions): (usize, [f64; 6]) = match movement_comp.initial_position {
    DegreesOfFreedom::OneDegree(x) => (1, [x, 0.0, 0.0, 0.0, 0.0, 0.0]),
    DegreesOfFreedom::TwoDegrees(ref p) => (2, [p.x, p.y, 0.0, 0.0, 0.0, 0.0]),
    DegreesOfFreedom::ThreeDegrees(ref p) => (3, [p.x, p.y, p.z, 0.0, 0.0, 0.0]),
    DegreesOfFreedom::FourDegrees(ref p) => (4, [p.x, p.y, p.z, p.u, 0.0, 0.0]),
    DegreesOfFreedom::FiveDegrees(ref p) => (5, [p.x, p.y, p.z, p.u, p.v, 0.0]),
    DegreesOfFreedom::SixDegrees(ref p) => (6, [p.x, p.y, p.z, p.u, p.v, p.w]),
  };

  let mut resolved = initial_positions;
  let mut has_target = false;
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

                for i in 0..dof_count {
                  let (val, completed) =
                    evaluate_channel(anim, elapsed, CHANNELS[i], initial_positions[i]);
                  if completed {
                    completed_gestures.push(gesture_id.clone());
                  }
                  let delta = (val - initial_positions[i]) * blend_intensity;
                  if !has_target {
                    resolved[i] = initial_positions[i] + delta;
                  } else {
                    resolved[i] += delta;
                  }
                }
                has_target = true;

                let duration = anim.duration.unwrap_or(1000.0);
                transition_window_ms = (duration.min(100.0) as u16).max(16);
              }
            }
          }
        }
      }
    }
  }

  if has_target {
    let mdeg: [u32; 6] = std::array::from_fn(|i| (resolved[i].max(0.0) * 1000.0) as u32);
    let movement_target = match dof_count {
      1 => MovementTarget::OneDof(mdeg[0]),
      2 => MovementTarget::TwoDof(mdeg[0], mdeg[1]),
      3 => MovementTarget::ThreeDof(mdeg[0], mdeg[1], mdeg[2]),
      4 => MovementTarget::FourDof(mdeg[0], mdeg[1], mdeg[2], mdeg[3]),
      5 => MovementTarget::FiveDof(mdeg[0], mdeg[1], mdeg[2], mdeg[3], mdeg[4]),
      _ => MovementTarget::SixDof(mdeg[0], mdeg[1], mdeg[2], mdeg[3], mdeg[4], mdeg[5]),
    };
    Some((
      ComponentTarget::Movement(movement_target),
      transition_window_ms,
    ))
  } else {
    None
  }
}
