//! # ModMan Gesture IPC Queryables & Status Publisher
//!
//! Exposes Zenoh queryables to inspect, query, and command gesture playback, and publishes
//! lifecycle events as gestures start, loop, pause, and end.
//!
//! ## Zenoh Endpoints
//!
//! ### Queryables Provided
//! - `com/reboot-codes/clover/hub/modman/gestures/all`: Queries all loaded gestures and keyframe track summaries.
//! - `com/reboot-codes/clover/hub/modman/gestures/active`: Queries active foreground/background stacks and states.
//! - `com/reboot-codes/clover/hub/modman/gestures/update`: Commands gesture state changes (Begin, Pause, End).
//!
//! ### Publishers
//! - `com/reboot-codes/clover/hub/modman/gestures/status`: Emits gesture lifecycle transition events.

use std::sync::Arc;

use anyhow::anyhow;
use serde::{
  Deserialize,
  Serialize,
};
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
  warn,
};

use crate::server::modman::{
  models::{
    gestures::{
      GestureCommand,
      GestureState,
      GestureStates,
    },
    store::ModManStore,
  },
  MODULE_EVT_ID,
};


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(untagged)]
pub enum GestureUpdatePayload {
  Nested {
    gesture_id: String,
    command: GestureCommand,
  },
  Flat {
    gesture_id: String,
    #[serde(flatten)]
    command: GestureCommand,
  },
}

impl GestureUpdatePayload {
  pub fn decompose(self) -> (String, GestureCommand) {
    match self {
      Self::Nested {
        gesture_id,
        command,
      } => (gesture_id, command),
      Self::Flat {
        gesture_id,
        command,
      } => (gesture_id, command),
    }
  }
}

pub async fn should_be_bg(store: Arc<ModManStore>, bg_from_state: Option<bool>) -> bool {
  match bg_from_state {
    Some(should_be_bg_period) => should_be_bg_period,
    None => store.config.lock().await.modman.gestures_bg_by_default,
  }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GestureCatalogItem {
  pub id: String,
  pub name: String,
  pub duration_ms: f64,
  pub curve: String,
  pub anim_type: String,
  pub keyframe_count: usize,
  pub keyframe_fractions: Vec<f64>,
}



/// Zenoh queryable responder and status publisher for gesture management.
///
/// ### Zenoh Queryables
///
/// #### 1. Gesture Command & Update
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/gestures/update`
/// - **Query Payload**: Serialized JSON [`GestureUpdatePayload`]
/// - **Reply Format**: Status string confirming outcome (e.g. `"ok"`, `"no-op: ..."`).
///
/// ##### Example Query Payload
/// ```json
/// {
///   "gesture_id": "wag_tail",
///   "state": {
///     "Begin": {
///       "speed": 1.0,
///       "intensity": 1.0,
///       "background": false
///     }
///   },
///   "auto_switch": null,
///   "is_from_system": false
/// }
/// ```
///
/// #### 2. Active Gestures
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/gestures/active`
/// - **Query Payload**: Empty
/// - **Reply Format**: Serialized JSON with priority lists and current playback states
///
/// ##### Example Reply
/// ```json
/// {
///   "foreground": ["wag_tail"],
///   "background": ["idle_breathing"],
///   "states": {
///     "wag_tail": {
///       "current_state": "Begin",
///       "paused": false,
///       "start_epoch": 1
///     }
///   }
/// }
/// ```
///
/// #### 3. Gestures Catalog
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/gestures/all`
/// - **Query Payload**: Empty
/// - **Reply Format**: Serialized JSON array of [`GestureSummary`]
///
/// ### Zenoh Publishers
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/gestures/status`
/// - **Payload Format**: JSON event object notifying lifecycle status
///
/// ##### Example Event
/// ```json
/// {
///   "gesture_id": "wag_tail",
///   "status": "started",
///   "foreground": ["wag_tail"],
///   "background": []
/// }
/// ```
#[instrument(skip(store, cancellation_token, session))]
pub async fn gesture_queryable(

  store: ModManStore,
  cancellation_token: CancellationToken,
  session: Arc<zenoh::Session>,
) {
  let key_expr = format!("{MODULE_EVT_ID}/gestures/update");
  let status_key_expr = format!("{MODULE_EVT_ID}/gestures/status");

  let queryable = match session.declare_queryable(&key_expr).await {
    Ok(q) => q,
    Err(e) => {
      error!("Failed to declare gesture queryable at {key_expr}: {e}");
      return;
    }
  };

  let status_publisher = match session.declare_publisher(&status_key_expr).await {
    Ok(p) => Some(p),
    Err(e) => {
      warn!("Failed to declare status publisher at {status_key_expr}: {e}");
      None
    }
  };

  let active_key_expr = format!("{MODULE_EVT_ID}/gestures/active");
  let active_queryable = match session.declare_queryable(&active_key_expr).await {
    Ok(q) => Some(q),
    Err(e) => {
      warn!("Failed to declare active gestures queryable at {active_key_expr}: {e}");
      None
    }
  };

  let all_key_expr = format!("{MODULE_EVT_ID}/gestures/all");
  let all_queryable = match session.declare_queryable(&all_key_expr).await {
    Ok(q) => Some(q),
    Err(e) => {
      warn!("Failed to declare all gestures queryable at {all_key_expr}: {e}");
      None
    }
  };

  let store_arc = Arc::new(store);

  debug!("Listening for gesture updates on {key_expr}, {active_key_expr}, and {all_key_expr}!");
  while !cancellation_token.is_cancelled() {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        debug!("Shutting down gesture queryable...");
        break;
      }
      recv_all = async {
        if let Some(ref q) = all_queryable {
          q.recv_async().await
        } else {
          futures::future::pending().await
        }
      } => {
        match recv_all {
          Ok(query) => {
            let mut items = Vec::new();
            {
              let repos = store_arc.repos.lock().await;
              for (_repo_id, manifest) in repos.iter() {
                if let crate::server::warehouse::repos::models::Optional::Some(ref dir) = manifest.directory {
                  #[cfg(feature = "core")]
                  if let crate::server::warehouse::repos::models::OptionalStrTHashMap::Some(ref packs) = dir.gesture_packs {
                    for (_pack_id, pack) in packs {
                      if let crate::server::warehouse::repos::models::OptionalStrTHashMap::Some(ref gestures) = pack.gestures {
                        for (g_id, g_spec) in gestures {
                          if let crate::server::warehouse::repos::models::GestureSpec::DynamicGestureSpec(ref dyn_spec) = g_spec {
                            let name = match &dyn_spec.name {
                              crate::server::warehouse::repos::models::OptionalString::Some(s) => s.clone(),
                              _ => g_id.clone(),
                            };

                            let mut duration_ms = 1000.0;
                            let mut curve = "ease_in_out".to_string();
                            let mut anim_type = "once".to_string();
                            let mut kf_count = 3;
                            let mut kf_fractions = vec![0.0, 0.5, 1.0];

                            if let crate::server::warehouse::repos::models::OptionalStrTHashMap::Some(ref configs) = dyn_spec.configs {
                              for (_arch_id, arch) in configs {
                                if let crate::server::warehouse::repos::models::OptionalStrTHashMap::Some(ref targets) = arch.targets {
                                  for (_area_id, area) in targets {
                                    if let crate::server::warehouse::repos::models::Optional::Some(ref anim) = area.animation {
                                      if let Some(dur) = anim.duration {
                                        duration_ms = dur;
                                      }
                                      if let crate::server::warehouse::repos::models::OptionalString::Some(ref c) = anim.curve {
                                        curve = c.clone();
                                      }
                                      if let crate::server::warehouse::repos::models::OptionalString::Some(ref t) = anim.anim_type {
                                        anim_type = t.clone();
                                      }
                                      if let Some(ref kfs_val) = anim.keyframes {
                                        if let Some(arr) = kfs_val.as_array() {
                                          kf_count = arr.len().max(2);
                                          kf_fractions = (0..kf_count).map(|i| i as f64 / (kf_count - 1) as f64).collect();
                                        }
                                      }
                                      break;
                                    }
                                  }
                                }
                              }
                            }

                            items.push(GestureCatalogItem {
                              id: g_id.clone(),
                              name,
                              duration_ms,
                              curve,
                              anim_type,
                              keyframe_count: kf_count,
                              keyframe_fractions: kf_fractions,
                            });
                          }
                        }
                      }
                    }
                  }
                }
              }
            }
            if let Ok(json_str) = serde_json::to_string(&items) {
              let _ = query.reply(&all_key_expr, json_str).await;
            }
          }
          Err(err) => {
            warn!("Error receiving from all gestures queryable: {err}");
          }
        }
      }
      recv_active = async {
        if let Some(ref q) = active_queryable {
          q.recv_async().await
        } else {
          futures::future::pending().await
        }
      } => {
        match recv_active {
          Ok(query) => {
            let fg = store_arc.foreground_gesture_priority.lock().await.clone();
            let bg = store_arc.background_gesture_priority.lock().await.clone();
            let states = store_arc.gesture_states.lock().await.clone();
            let status_json = serde_json::json!({
              "foreground": fg,
              "background": bg,
              "states": states,
            });
            if let Ok(json_str) = serde_json::to_string(&status_json) {
              let _ = query.reply(&active_key_expr, json_str).await;
            }
          }
          Err(err) => {
            warn!("Error receiving from active gestures queryable: {err}");
          }
        }
      }
      recv_result = queryable.recv_async() => {
        match recv_result {
          Ok(query) => {
            let payload_str = match query.payload() {
              Some(p) => match p.try_to_string() {
                Ok(s) => s.to_string(),
                Err(_) => {
                  let reply_payload = serde_json::json!({
                    "status": "error",
                    "message": "Payload must be a valid UTF-8 string"
                  });
                  let _ = query.reply(&key_expr, &serde_json::to_string(&reply_payload).unwrap()).await;
                  continue;
                }
              },
              None => {
                let reply_payload = serde_json::json!({
                  "status": "error",
                  "message": "Empty query payload"
                });
                let _ = query.reply(&key_expr, &serde_json::to_string(&reply_payload).unwrap()).await;
                continue;
              }
            };

            match serde_json_lenient::from_str::<GestureUpdatePayload>(&payload_str) {
              Ok(payload) => {
                let (gesture_id, command) = payload.decompose();
                match handle_gesture_cmd(store_arc.clone(), gesture_id.clone(), command).await {
                  Ok(msg) => {
                    let reply_payload = serde_json::json!({
                      "status": "ok",
                      "gesture_id": gesture_id,
                      "message": msg
                    });
                    let _ = query.reply(&key_expr, &serde_json::to_string(&reply_payload).unwrap()).await;

                    if let Some(ref publ) = status_publisher {
                      let fg = store_arc.foreground_gesture_priority.lock().await.clone();
                      let bg = store_arc.background_gesture_priority.lock().await.clone();
                      let status_json = serde_json::json!({
                        "foreground": fg,
                        "background": bg,
                      });
                      let _ = publ.put(serde_json::to_string(&status_json).unwrap()).await;
                    }
                  }
                  Err(err) => {
                    let reply_payload = serde_json::json!({
                      "status": "error",
                      "gesture_id": gesture_id,
                      "message": err.to_string()
                    });
                    let _ = query.reply(&key_expr, &serde_json::to_string(&reply_payload).unwrap()).await;
                  }
                }
              }
              Err(err) => {
                let reply_payload = serde_json::json!({
                  "status": "error",
                  "message": format!("Failed to parse gesture update payload: {err}")
                });
                let _ = query.reply(&key_expr, &serde_json::to_string(&reply_payload).unwrap()).await;
              }
            }
          }
          Err(err) => {
            warn!("Error receiving from gesture queryable: {err}");
          }
        }
      }
    }
  }

  debug!("Stopped gesture queryable.");
}

#[instrument(skip(store))]
pub async fn handle_gesture_cmd(
  store: Arc<ModManStore>,
  gesture_id: String,
  command: GestureCommand,
) -> Result<String, anyhow::Error> {
  info!(
    "Handling gesture \"{}\" and state command: {:#?}",
    gesture_id.clone(),
    command.clone()
  );

  let mut gesture_state_map = store.gesture_states.lock().await;
  let mut modded_gesture_state = None;

  let stored_states = match gesture_state_map.get(&gesture_id.clone()) {
    Some(stored_states_from_map) => {
      if (stored_states_from_map.current_state == command.state)
        || (command.state == GestureState::Pause && stored_states_from_map.paused)
        || (command.state == GestureState::UnPause && !stored_states_from_map.paused)
      {
        warn!("Sent gesture state command which matches current state, this is a no-op!");
        return Ok("no-op: state already set to requested value".to_string());
      } else {
        stored_states_from_map.clone()
      }
    }
    None => {
      match command.state {
        GestureState::Begin {
          intensity: _,
          speed: _,
          background: _,
        } => GestureStates {
          prev_state: None,
          current_state: GestureState::End,
          next_state: None,
          paused: false,
          start_epoch: 0,
        },
        _ => {
          let msg = format!(
            "Command: {:?}, is not applicable since this gesture was not cached in the first place!",
            command.state
          );
          error!("{msg}");
          return Err(anyhow!("{msg}"));
        }
      }
    }
  };

  match command.state {
    GestureState::Begin {
      intensity: _,
      speed: _,
      background,
    } => {
      let is_bg = should_be_bg(store.clone(), background).await;
      if is_bg {
        let mut fg = store.foreground_gesture_priority.lock().await;
        fg.retain(|item| item != &gesture_id);
        drop(fg);

        let mut bg = store.background_gesture_priority.lock().await;
        bg.retain(|item| item != &gesture_id);
        bg.push(gesture_id.clone());
      } else {
        let mut bg = store.background_gesture_priority.lock().await;
        bg.retain(|item| item != &gesture_id);
        drop(bg);

        let mut fg = store.foreground_gesture_priority.lock().await;
        fg.retain(|item| item != &gesture_id);
        fg.push(gesture_id.clone());
      }

      modded_gesture_state = Some(GestureStates {
        prev_state: stored_states.prev_state.clone(),
        current_state: command.state,
        next_state: stored_states.next_state.clone(),
        paused: false,
        start_epoch: stored_states.start_epoch.wrapping_add(1),
      });
    }
    GestureState::Pause => {
      modded_gesture_state = Some(GestureStates {
        prev_state: stored_states.prev_state.clone(),
        current_state: stored_states.current_state.clone(),
        next_state: stored_states.next_state.clone(),
        paused: true,
        start_epoch: stored_states.start_epoch,
      });
    }
    GestureState::End => {
      // Remove the gesture from both gesture priority vecs.
      let mut bg_priority = store.background_gesture_priority.lock().await;
      bg_priority.retain(|item| item != &gesture_id);
      drop(bg_priority);

      let mut fg_priority = store.foreground_gesture_priority.lock().await;
      fg_priority.retain(|item| item != &gesture_id);
      drop(fg_priority);
    }
    GestureState::UnPause => {
      modded_gesture_state = Some(GestureStates {
        prev_state: stored_states.prev_state.clone(),
        current_state: stored_states.current_state.clone(),
        next_state: stored_states.next_state.clone(),
        paused: false,
        start_epoch: stored_states.start_epoch,
      });
    }
  }

  // Since we already checked that the state command is different, either apply the changes or remove the gesture from the state map
  match modded_gesture_state {
    Some(state_to_insert) => {
      gesture_state_map.insert(gesture_id.clone(), state_to_insert);
    }
    None => {
      gesture_state_map.remove(&gesture_id.clone());
    }
  }

  debug!("Gesture ID: {}, state set!", gesture_id.clone());
  Ok(format!("Gesture ID: {gesture_id}, state successfully updated"))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[tokio::test]
  async fn test_gesture_lifecycle_transitions() {
    let store = Arc::new(ModManStore::new(None, None));
    let gesture_id = "ear_twitch".to_string();

    // 1. Begin foreground gesture
    let begin_cmd = GestureCommand {
      state: GestureState::Begin {
        intensity: 0.8,
        speed: 1.2,
        background: Some(false),
      },
      auto_switch: None,
      is_from_system: false,
      areas: None,
    };

    let res = handle_gesture_cmd(store.clone(), gesture_id.clone(), begin_cmd.clone()).await;
    assert!(res.is_ok());

    {
      let fg = store.foreground_gesture_priority.lock().await;
      assert_eq!(fg.as_slice(), &[gesture_id.clone()]);
      let states = store.gesture_states.lock().await;
      assert!(states.contains_key(&gesture_id));
      assert!(!states.get(&gesture_id).unwrap().paused);
    }

    // 2. Duplicate Begin is a no-op
    let res = handle_gesture_cmd(store.clone(), gesture_id.clone(), begin_cmd).await;
    assert!(res.is_ok());
    assert!(res.unwrap().contains("no-op"));

    // 3. Pause gesture
    let pause_cmd = GestureCommand {
      state: GestureState::Pause,
      auto_switch: None,
      is_from_system: false,
      areas: None,
    };
    let res = handle_gesture_cmd(store.clone(), gesture_id.clone(), pause_cmd).await;
    assert!(res.is_ok());
    {
      let states = store.gesture_states.lock().await;
      assert!(states.get(&gesture_id).unwrap().paused);
    }

    // 4. UnPause gesture
    let unpause_cmd = GestureCommand {
      state: GestureState::UnPause,
      auto_switch: None,
      is_from_system: false,
      areas: None,
    };
    let res = handle_gesture_cmd(store.clone(), gesture_id.clone(), unpause_cmd).await;
    assert!(res.is_ok());
    {
      let states = store.gesture_states.lock().await;
      assert!(!states.get(&gesture_id).unwrap().paused);
    }

    // 5. End gesture
    let end_cmd = GestureCommand {
      state: GestureState::End,
      auto_switch: None,
      is_from_system: false,
      areas: None,
    };
    let res = handle_gesture_cmd(store.clone(), gesture_id.clone(), end_cmd).await;
    assert!(res.is_ok());
    {
      let fg = store.foreground_gesture_priority.lock().await;
      assert!(fg.is_empty());
      let states = store.gesture_states.lock().await;
      assert!(!states.contains_key(&gesture_id));
    }
  }

  #[tokio::test]
  async fn test_unregistered_gesture_cannot_pause() {
    let store = Arc::new(ModManStore::new(None, None));
    let pause_cmd = GestureCommand {
      state: GestureState::Pause,
      auto_switch: None,
      is_from_system: false,
      areas: None,
    };
    let res = handle_gesture_cmd(store, "unknown_gesture".to_string(), pause_cmd).await;
    assert!(res.is_err());
  }

  #[test]
  fn test_gesture_update_payload_deserialization() {
    // Nested format
    let nested_json = r#"{
      "gesture_id": "tail_wag",
      "command": {
        "state": {
          "command": "begin",
          "intensity": 0.5,
          "speed": 1.0,
          "background": true
        },
        "auto_switch": null,
        "is_from_system": true,
        "areas": ["pelvis.waist"]
      }
    }"#;

    let payload: GestureUpdatePayload = serde_json_lenient::from_str(nested_json).unwrap();
    let (id, cmd) = payload.decompose();
    assert_eq!(id, "tail_wag");
    assert!(cmd.is_from_system);
    assert_eq!(
      cmd.state,
      GestureState::Begin {
        intensity: 0.5,
        speed: 1.0,
        background: Some(true),
      }
    );

    // Flat format
    let flat_json = r#"{
      "gesture_id": "ear_twitch",
      "state": {
        "command": "pause"
      },
      "auto_switch": null,
      "is_from_system": false,
      "areas": null
    }"#;

    let payload: GestureUpdatePayload = serde_json_lenient::from_str(flat_json).unwrap();
    let (id, cmd) = payload.decompose();
    assert_eq!(id, "ear_twitch");
    assert_eq!(cmd.state, GestureState::Pause);
  }
}
