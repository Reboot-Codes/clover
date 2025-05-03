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

use log::info;

use crate::server::modman::models::GestureCommand;

pub fn handle_gesture_cmd(gesture_id: String, command: GestureCommand) {
  info!(
    "Handling gesture \"{}\" and state: {:#?}",
    gesture_id.clone(),
    command.clone()
  );
}
