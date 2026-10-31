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

  let store_arc = Arc::new(store);

  debug!("Listening for gesture updates on {key_expr}!");
  while !cancellation_token.is_cancelled() {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        debug!("Shutting down gesture queryable...");
        break;
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
      if stored_states_from_map.current_state == command.state {
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
      // When beginning a gesture (or setting new parameters), move it to the top of either back/foreground application list.
      let mut priority_vec;

      if should_be_bg(store.clone(), background).await {
        priority_vec = store.background_gesture_priority.lock().await;
      } else {
        priority_vec = store.foreground_gesture_priority.lock().await;
      }

      if let Some(index) = priority_vec.iter().position(|item| item == &gesture_id) {
        priority_vec.remove(index);
      }

      priority_vec.push(gesture_id.clone());

      modded_gesture_state = Some(GestureStates {
        prev_state: stored_states.prev_state.clone(),
        current_state: command.state,
        next_state: stored_states.next_state.clone(),
        paused: false,
      });
    }
    GestureState::Pause => {
      modded_gesture_state = Some(GestureStates {
        prev_state: stored_states.prev_state.clone(),
        current_state: stored_states.current_state.clone(),
        next_state: stored_states.next_state.clone(),
        paused: true,
      });
    }
    GestureState::End => {
      // Remove the gesture from either gesture priority vec.
      let mut bg_priority = store.background_gesture_priority.lock().await;
      if let Some(index) = bg_priority.iter().position(|item| item == &gesture_id) {
        bg_priority.remove(index);
      }
      drop(bg_priority);

      let mut fg_priority = store.foreground_gesture_priority.lock().await;
      if let Some(index) = fg_priority.iter().position(|item| item == &gesture_id) {
        fg_priority.remove(index);
      }
      drop(fg_priority);
    }
    GestureState::UnPause => {
      modded_gesture_state = Some(GestureStates {
        prev_state: stored_states.prev_state.clone(),
        current_state: stored_states.current_state.clone(),
        next_state: stored_states.next_state.clone(),
        paused: false,
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
