use tracing::{
  debug,
  instrument,
};

use crate::server::modman::models::{
  GestureParameters,
  GestureState,
  GestureStates,
};

#[instrument]
pub async fn gesture_command_generator(gesture_id: &String, gesture_state: &GestureStates) {
  debug!(
    "Cacluating gesture: {}, with: {:?}",
    gesture_id, gesture_state
  );

  match gesture_state.current_state {
    GestureState::Begin {
      intensity,
      speed,
      background,
    } => {
      // Get supplemental configuration areas. (E.g. areas from @core.humanoid.furry.canine.domestic-dog.plantigrade in addition to @core, @core.humanoid.pentadactal, @core.humanoid.diopical)

      // Get Union of the gesture's supported areas, the instance's defined areas, and command requested areas.

      // Find all modules under applicable areas.

      // Calculate intensity

      // Calculate speed

      todo!();
    }
    _ => {
      // Not covered by this function, see `super::ipc::gestures`!
    }
  }
}
