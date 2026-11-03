//! # ModMan Gestures Engine
//!
//! Generates real-time actuator command setpoints from active gesture priorities,
//! evaluates keyframe curves, blends overlapping body area targets, and dispatches commands
//! over CAN 2.0 / SocketCAN proxies at a fixed 60 Hz rate.
//!
//! ## Architecture Overview
//! - **60 Hz Generator Loop**: [`gesture_command_generator_manager`] runs on an asynchronous interval
//!   of 16,667 microseconds with [`MissedTickBehavior::Skip`] to guarantee constant-rate servo updates
//!   without burst backlog behavior.
//! - **Playhead Runtime**: [`tracks::GesturePlayhead`] tracks delta time advancement ($\Delta t \times \text{speed}$)
//!   and caches previous setpoints for deadband checks.
//! - **Priority Blending**: Multiple gestures targeting overlapping body areas blend additively
//!   based on playhead intensity weights.
//! - **Deadband Optimization**: Actuator commands differing by less than 50 millidegrees (0.05°) from
//!   the previous frame are filtered out to avoid saturating CAN buffers.
//! - **Automated Transitions**: On track completion, status events are published over Zenoh and queued
//!   `next_state` sequences (such as loops or returns to neutral) automatically begin.

// TODO: Manifests specify bodies. Allow globs.
// TODO: Each area has default applicable gestures
/*
Default area specification
  - com.reboot-codes.clover.CORE.humanoid
    - torso
      - core
        - back
          - spine
            - upper/middle/bottom
        - front
      - chest
        - font
          - right/center/left
        - back
          - right/center/left
      - arms
        - right/left
          - shoulder
          - biceps
          - forearm
          - hand
            - fingers
              - thumb
              - index
              - middle
              - ring
              - pinky
      - head
        - neck
        - face
          - jaw
          - mouth
          - nose
          - eyes
            - right/left
              - eye
              - eyelids
                - upper/lower
              - eyebrow
        - ears
          right/left
        - top
        - back
    - pelivs
      - waist
        - left/right/upper/lower
      - front
        - upper/lower
      - bottom
      - rear
        - right/left
    - legs
      - right/left
        - thigh
        - shin
        - foot
          - toes
            - hallux
            - index
            - middle
            - ring
            - pinky
*/

pub mod command_generator;
pub mod evaluators;
pub mod tracks;

use std::{
  collections::HashMap,
  sync::Arc,
  time::Duration,
};

use carbon_fiber::commands::models::components::ComponentTarget;
use tokio::time::{
  interval,
  MissedTickBehavior,
};
use tokio_util::sync::CancellationToken;
use tracing::{
  info,
  instrument,
  warn,
};

use crate::server::modman::{
  gestures::{
    command_generator::{
      dispatch_target,
      is_outside_deadband,
    },
    evaluators::{
      evaluate_indicator,
      evaluate_movement,
      evaluate_speaker,
    },
    tracks::GesturePlayhead,
  },
  models::{
    components::CloverComponent,
    gestures::GestureState,
    store::ModManStore,
  },
  MODULE_EVT_ID,
};

/// Fixed 60 Hz gesture command generator loop.
///
/// Drives all active gestures in foreground and background priority queues, advancing
/// playheads by $\Delta t \times \text{speed}$, evaluating keyframe tracks across matching body
/// locations, and dispatching composite setpoints to modules over Zenoh.
///
/// # Arguments
/// * `store` - Shared [`ModManStore`] containing active modules, components, and priority stacks.
/// * `session` - Active Zenoh session for query dispatch and status broadcasts.
/// * `cancellation_token` - Token signaling graceful shutdown of the generator task.
///
/// # Pipeline Steps
/// 1. **Tick Synchronization**: Executes every ~16.67 ms. Skips dropped ticks if execution falls behind.
/// 2. **Playhead Advancement**: Synchronizes playheads with priority stacks, advancing non-paused gestures.
/// 3. **Track Evaluation & Blending**: Queries compiled manifests in Warehouse, evaluates channels,
///    and blends setpoints additively.
/// 4. **Deadband Suppression**: Compares computed setpoints against previously sent targets; setpoint
///    changes under 50 millidegrees are omitted from transmission.
/// 5. **Lifecycle & Transitions**: On track completion, emits `"ended"` status events, then automatically
///    advances to `next_state` if queued (e.g., transitioning to a loop or returning to neutral).
#[instrument(skip(store, session, cancellation_token))]
pub async fn gesture_command_generator_manager(
  store: Arc<ModManStore>,
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
) {
  info!("Starting gesture command generator...");

  // TODO: Make this configurable!
  let mut ticker = interval(Duration::from_micros(16667));
  ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

  let status_key_expr = format!("{MODULE_EVT_ID}/gestures/status");
  let status_publisher = match session.declare_publisher(&status_key_expr).await {
    Ok(p) => Some(p),
    Err(e) => {
      warn!("Failed to declare gesture status publisher: {e}");
      None
    }
  };

  let mut playheads: HashMap<String, GesturePlayhead> = HashMap::new();
  let dt = Duration::from_micros(16667);

  loop {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        info!("Gesture generator received cancellation signal, stopping.");
        break;
      }
      _ = ticker.tick() => {
        let (fg_ids, bg_ids) = {
          let fg = store.foreground_gesture_priority.lock().await.clone();
          let bg = store.background_gesture_priority.lock().await.clone();
          (fg, bg)
        };

        if fg_ids.is_empty() && bg_ids.is_empty() {
          continue;
        }

        let default_pack = {
          store.config.lock().await.default_gesture_pack.clone()
        };

        let mut completed_gestures: Vec<String> = Vec::new();
        let mut dispatches = Vec::new();

        // 1. Synchronize and advance playheads
        let all_active_ids: Vec<(String, bool)> = bg_ids.iter().map(|id| (id.clone(), true))
          .chain(fg_ids.iter().map(|id| (id.clone(), false)))
          .collect();

        // Remove stale playheads
        playheads.retain(|k, _| bg_ids.contains(k) || fg_ids.contains(k));

        for (gesture_id, is_bg) in &all_active_ids {
          let states_lock = store.gesture_states.lock().await;
          let gesture_state = match states_lock.get(gesture_id) {
            Some(s) => s.clone(),
            None => continue,
          };
          drop(states_lock);

          let (speed, intensity) = match gesture_state.current_state {
            GestureState::Begin { speed, intensity, .. } => (speed, intensity),
            _ => (1.0, 1.0),
          };

          let playhead = playheads.entry(gesture_id.clone()).or_insert_with(|| {
            let mut p = GesturePlayhead::new(speed, intensity, *is_bg);
            p.epoch = gesture_state.start_epoch;
            p
          });

          // Detect restart via start_epoch change
          if playhead.epoch != gesture_state.start_epoch {
            playhead.epoch = gesture_state.start_epoch;
            playhead.elapsed = Duration::ZERO;
          }

          playhead.speed = speed;
          playhead.intensity = intensity;
          playhead.background = *is_bg;

          if !gesture_state.paused {
            playhead.advance(dt);
          }
        }

        // 2. Direct timeline completion detection across all active gestures
        {
          let repos = store.repos.lock().await;
          for (gesture_id, _) in &all_active_ids {
            let (duration_ms, anim_type) =
              command_generator::get_gesture_duration_and_type(&repos, &default_pack, gesture_id);
            let duration_secs = (duration_ms / 1000.0).max(0.01);
            if anim_type != "loop" && anim_type != "bounce" {
              if let Some(p) = playheads.get(gesture_id) {
                if p.elapsed.as_secs_f64() >= duration_secs {
                  completed_gestures.push(gesture_id.clone());
                }
              }
            }
          }
        }

        // 3. Evaluate gesture tracks and blend setpoints for each component using borrowed locks
        {
          let repos = store.repos.lock().await;
          let components = store.components.lock().await;
          let modules = store.modules.lock().await;

          for (component_id, arc_comp) in components.iter() {
            let (meta, component) = &**arc_comp;
            // Locate owning module for this component
            let mut owning_module_id = None;
            let mut component_idx = 0u8;

            for (m_id, m) in modules.iter() {
              for (idx, (c_id, _)) in m.components.iter().enumerate() {
                if c_id == component_id {
                  owning_module_id = Some(m_id.clone());
                  component_idx = (idx & 0xFF) as u8;
                  break;
                }
              }
              if owning_module_id.is_some() {
                break;
              }
            }

            let mod_id = match owning_module_id {
              Some(id) => id,
              None => continue,
            };

            let target_and_window = match component {
              CloverComponent::MovementComponent(ref movement_comp) => evaluate_movement(
                movement_comp,
                meta,
                &playheads,
                &all_active_ids,
                &repos,
                &default_pack,
                &mut completed_gestures,
              ),
              CloverComponent::IndicatorComponent(ref indicator_comp) => evaluate_indicator(
                indicator_comp,
                meta,
                &playheads,
                &all_active_ids,
                &repos,
                &default_pack,
                &mut completed_gestures,
              ),
              CloverComponent::AudioOutputComponent(ref audio_comp) => evaluate_speaker(
                audio_comp,
                meta,
                &playheads,
                &all_active_ids,
                &repos,
                &default_pack,
                &mut completed_gestures,
              ),
              _ => None,
            };

            if let Some((target, transition_window_ms)) = target_and_window {
              let threshold = match target {
                ComponentTarget::Movement(_) => 50,
                _ => 0,
              };

              let should_dispatch = if let Some(first_playhead) = playheads.values_mut().next() {
                match first_playhead.last_dispatched_targets.get(component_id) {
                  Some(last_target) => is_outside_deadband(last_target, &target, threshold),
                  None => true,
                }
              } else {
                true
              };

              if should_dispatch {
                dispatches.push((mod_id, component_idx, target, transition_window_ms));
                for p in playheads.values_mut() {
                  p.last_dispatched_targets.insert(component_id.clone(), target);
                }
              }
            }
          }
        }

        // Dispatch outside locks to avoid holding Mutexes across async network calls
        for (mod_id, comp_idx, target, window_ms) in dispatches {
          let _ = dispatch_target(&session, &mod_id, comp_idx, target, window_ms).await;
        }

        // Periodic memory trim every 300 ticks (~5s) to release unused heap pages to OS
        #[cfg(target_os = "linux")]
        {
          static TICK_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
          if TICK_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % 300 == 0 {
            unsafe {
              extern "C" {
                fn malloc_trim(pad: usize) -> i32;
              }
              malloc_trim(0);
            }
          }
        }

        // 4. Process completed gestures and transition to next state
        if !completed_gestures.is_empty() {
          completed_gestures.dedup();
          for g_id in completed_gestures {
            info!("Gesture {g_id} completed playback.");

            let mut states_lock = store.gesture_states.lock().await;
            if let Some(state) = states_lock.get_mut(&g_id) {
              if let Some((_delay, next)) = state.next_state.take() {
                match next {
                  GestureState::Begin { speed, intensity, background } => {
                    info!("Auto-transitioning gesture {g_id} to next state: Begin");
                    state.current_state = GestureState::Begin { speed, intensity, background };
                    state.start_epoch = state.start_epoch.wrapping_add(1);
                    if let Some(p) = playheads.get_mut(&g_id) {
                      p.elapsed = Duration::ZERO;
                      p.epoch = state.start_epoch;
                      p.speed = speed;
                      p.intensity = intensity;
                    }
                  }
                  GestureState::End => {
                    info!("Gesture {g_id} next state is End, removing from priorities.");
                    states_lock.remove(&g_id);
                    playheads.remove(&g_id);
                    drop(states_lock);

                    let mut fg = store.foreground_gesture_priority.lock().await;
                    fg.retain(|item| item != &g_id);
                    let mut bg = store.background_gesture_priority.lock().await;
                    bg.retain(|item| item != &g_id);
                  }
                  _ => {}
                }
              } else {
                // No queued next state, remove completed gesture
                states_lock.remove(&g_id);
                playheads.remove(&g_id);
                drop(states_lock);

                let mut fg = store.foreground_gesture_priority.lock().await;
                fg.retain(|item| item != &g_id);
                let mut bg = store.background_gesture_priority.lock().await;
                bg.retain(|item| item != &g_id);
              }
            } else {
              playheads.remove(&g_id);
              drop(states_lock);

              let mut fg = store.foreground_gesture_priority.lock().await;
              fg.retain(|item| item != &g_id);
              let mut bg = store.background_gesture_priority.lock().await;
              bg.retain(|item| item != &g_id);
            }

            if let Some(ref publ) = status_publisher {
              let fg = store.foreground_gesture_priority.lock().await.clone();
              let bg = store.background_gesture_priority.lock().await.clone();
              let evt = serde_json::json!({
                "gesture_id": g_id,
                "status": "ended",
                "foreground": fg,
                "background": bg,
              });
              let _ = publ.put(serde_json::to_string(&evt).unwrap()).await;
            }
          }
        }
      }
    }
  }

  info!("Stopped generating gesture command generator.");
}
