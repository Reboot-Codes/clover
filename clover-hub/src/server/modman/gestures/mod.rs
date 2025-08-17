// TODO: Manifests specify bodies? Allow globs.
// TODO: Each area has default applicable gestures
/*
Default area specification
  - com.reboot-codes.clover.CORE.human
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

use std::sync::Arc;

use log::{
  debug,
  error,
};
use tokio_util::sync::CancellationToken;

use crate::server::modman::models::{
  GestureState,
  ModManStore,
};

pub async fn gesture_command_generator_manager(
  store: Arc<ModManStore>,
  cancellation_token: CancellationToken,
) {
  while !cancellation_token.is_cancelled() {
    let gestures_store_guards = vec![
      store.background_gesture_priority.lock().await,
      store.foreground_gesture_priority.lock().await,
    ];

    for mut gestures_store_guard in gestures_store_guards {
      let mut gestures_to_remove = vec![];
      let mut gesture_index_plus_one: usize = 0;

      for gesture_id in gestures_store_guard.iter() {
        gesture_index_plus_one += 1;

        match store.gesture_states.lock().await.get(gesture_id) {
          Some(gesture_state) => match gesture_state.current_state {
            GestureState::Begin {
              intensity,
              speed,
              background,
            } => todo!(),
            GestureState::Pause => todo!(),
            _ => {
              // Not covered by this function, see `super::ipc::gestures`!
            }
          },
          None => {
            error!("Gesture: {}, is not in state map, removing from background gesture array! This state de-sync is not permissable and should be reported as a bug!", gesture_id.clone());
            gestures_to_remove.push((gesture_index_plus_one - 1, gesture_id.clone()));
            continue;
          }
        }
      }

      for (gesture_index, gesture_id) in gestures_to_remove {
        debug!(
          "Removing gesture: {} (@ index: {})...",
          gesture_id, gesture_index
        );
        gestures_store_guard.remove(gesture_index);
      }
    }
  }
}
