use std::sync::Arc;

use log::{
  debug,
  error,
  info,
  warn,
};

use crate::server::modman::models::{
  GestureCommand,
  GestureState,
  GestureStates,
  ModManStore,
};

pub async fn should_be_bg(store: Arc<ModManStore>, bg_from_state: Option<bool>) -> bool {
  match bg_from_state {
    Some(should_be_bg_period) => should_be_bg_period,
    None => store.config.lock().await.modman.gestures_bg_by_default,
  }
}

pub async fn handle_gesture_cmd(
  store: &mut Arc<ModManStore>,
  gesture_id: String,
  command: GestureCommand,
) {
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
        // TODO: Send reply!
        return;
      } else {
        stored_states_from_map
      }
    }
    None => {
      match command.state {
        GestureState::Begin {
          intensity: _,
          speed: _,
          background: _,
        } => &GestureStates {
          prev_state: None,
          current_state: GestureState::End,
          next_state: None,
          paused: false,
        },
        _ => {
          error!("Command: {:#?}, is not applicable since this gesture was not cached in the first place!", command.state);
          // TODO: Send reply!
          return;
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
      // When begining a gesture (or setting new parameters), move it to the top of either back/foreground application list.
      let mut priority_vec;

      if should_be_bg(store.clone(), background).await {
        priority_vec = store.background_gesture_priority.lock().await;
      } else {
        priority_vec = store.foreground_gesture_priority.lock().await;
      }

      match {
        let mut ret = None;
        let mut index: usize = 0;

        for item in priority_vec.iter() {
          if item == &gesture_id.clone() {
            ret = Some((index, item.clone()));
            break;
          }
          index += 1;
        }

        ret
      } {
        Some((index, _item)) => {
          priority_vec.remove(index);
        }
        None => {}
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
      let priority_vecs = vec![
        store.background_gesture_priority.lock().await,
        store.foreground_gesture_priority.lock().await,
      ];

      for mut priority_vec in priority_vecs {
        match {
          let mut ret = None;
          let mut index: usize = 0;

          for item in priority_vec.iter() {
            if item == &gesture_id.clone() {
              ret = Some((index, item.clone()));
              break;
            }
            index += 1;
          }

          ret
        } {
          Some((index, _item)) => {
            priority_vec.remove(index);
          }
          // Wasn't in the vecs at all, so it's a no-op.
          None => {}
        }
      }
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

  // TODO: Send reply!
}
