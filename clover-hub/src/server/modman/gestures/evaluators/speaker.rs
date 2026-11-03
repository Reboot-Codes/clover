//! # Speaker Track Evaluator
//!
//! Evaluates keyframe tracks for [`AudioOutputComponent`] connected via `ModManProxy`,
//! supporting tone frequency and volume percent modulation, and automatic silence transitions.

use std::collections::HashMap;

use carbon_fiber::commands::models::components::ComponentTarget;

use crate::server::{
  modman::{
    components::audio::models::{AudioOutputComponent, ConnectionType},
    gestures::{
      command_generator::{evaluate_channel, find_gesture_in_manifests, wildcard_match},
      tracks::GesturePlayhead,
    },
    models::components::CloverComponentMeta,
  },
  warehouse::repos::models::{Manifest, Optional, OptionalStrTHashMap},
};

/// Evaluates active gesture tracks targeting an [`AudioOutputComponent`].
pub fn evaluate_speaker(
  audio_comp: &AudioOutputComponent,
  meta: &CloverComponentMeta,
  playheads: &HashMap<String, GesturePlayhead>,
  active_gesture_ids: &[(String, bool)],
  repos: &HashMap<String, Manifest>,
  default_pack: &str,
  completed_gestures: &mut Vec<String>,
) -> Option<(ComponentTarget, u16)> {
  if !matches!(audio_comp.connection, ConnectionType::ModManProxy) {
    return None;
  }

  let mut resolved_freq = 0.0;
  let mut resolved_vol = 0.0;
  let mut has_audio = false;
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
                let (freq, c_f) = evaluate_channel(anim, elapsed, "frequency", 0.0);
                let (vol, c_v) = evaluate_channel(anim, elapsed, "volume", 100.0);

                if c_f || c_v {
                  completed_gestures.push(gesture_id.clone());
                }

                let blend_intensity = playhead.intensity.clamp(0.0, 1.0);
                if !has_audio {
                  resolved_freq = freq;
                  resolved_vol = vol * blend_intensity;
                  has_audio = true;
                } else {
                  resolved_freq = freq;
                  resolved_vol = (resolved_vol + vol * blend_intensity).min(100.0);
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

  if has_audio {
    let target = if resolved_vol <= 0.0 || resolved_freq <= 0.0 {
      ComponentTarget::SpeakerSilence
    } else {
      ComponentTarget::SpeakerTone {
        frequency_hz: resolved_freq.clamp(0.0, 65535.0) as u16,
        volume_percent: resolved_vol.clamp(0.0, 100.0) as u8,
      }
    };
    Some((target, transition_window_ms))
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use std::time::Duration;
  use super::*;
  use crate::server::modman::components::audio::models::DirectConnection;

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
  fn test_evaluate_speaker_non_modman_proxy_returns_none() {
    let comp = AudioOutputComponent {
      connection: ConnectionType::Direct(DirectConnection {
        device_id: "default".to_string(),
        connection: None,
      }),
      gesture_config: None,
    };
    let meta = CloverComponentMeta {
      name: "speaker".to_string(),
      critical: false,
      location: "head.speaker".to_string(),
      base_gesture_parameters: HashMap::new(),
      internal: false,
    };
    let playheads = HashMap::new();
    let repos = HashMap::new();
    let active = vec![("beep".to_string(), true)];
    let mut completed = Vec::new();

    let result = evaluate_speaker(
      &comp,
      &meta,
      &playheads,
      &active,
      &repos,
      "default",
      &mut completed,
    );
    assert!(result.is_none());
  }

  #[test]
  fn test_evaluate_speaker_tone_and_silence() {
    let comp = AudioOutputComponent {
      connection: ConnectionType::ModManProxy,
      gesture_config: None,
    };
    let meta = CloverComponentMeta {
      name: "speaker".to_string(),
      critical: false,
      location: "head.speaker".to_string(),
      base_gesture_parameters: HashMap::new(),
      internal: false,
    };

    let manifest = dummy_manifest(
      "beep",
      "head.speaker",
      serde_json::json!([
        { "frequency": 440.0, "volume": 80.0 },
        { "frequency": 880.0, "volume": 40.0 }
      ]),
    );
    let mut repos = HashMap::new();
    repos.insert("core".to_string(), manifest);

    let mut playheads = HashMap::new();
    playheads.insert(
      "beep".to_string(),
      GesturePlayhead {
        elapsed: Duration::from_millis(500),
        intensity: 1.0,
        speed: 1.0,
        background: false,
        epoch: 0,
        last_dispatched_targets: HashMap::new(),
      },
    );

    let active = vec![("beep".to_string(), true)];
    let mut completed = Vec::new();

    let result = evaluate_speaker(
      &comp,
      &meta,
      &playheads,
      &active,
      &repos,
      "default",
      &mut completed,
    );
    assert!(result.is_some());
    let (target, _window) = result.unwrap();
    match target {
      ComponentTarget::SpeakerTone {
        frequency_hz,
        volume_percent,
      } => {
        assert_eq!(frequency_hz, 660);
        assert_eq!(volume_percent, 60);
      }
      other => panic!("Expected SpeakerTone, got {other:?}"),
    }

    // Zero volume should produce silence
    let manifest_silent = dummy_manifest(
      "quiet",
      "head.speaker",
      serde_json::json!([
        { "frequency": 440.0, "volume": 0.0 }
      ]),
    );
    repos.insert("core".to_string(), manifest_silent);
    let active_quiet = vec![("quiet".to_string(), true)];
    playheads.insert(
      "quiet".to_string(),
      GesturePlayhead {
        elapsed: Duration::from_millis(0),
        intensity: 1.0,
        speed: 1.0,
        background: false,
        epoch: 0,
        last_dispatched_targets: HashMap::new(),
      },
    );

    let result_silent = evaluate_speaker(
      &comp,
      &meta,
      &playheads,
      &active_quiet,
      &repos,
      "default",
      &mut completed,
    );
    assert_eq!(
      result_silent.unwrap().0,
      ComponentTarget::SpeakerSilence
    );
  }
}
