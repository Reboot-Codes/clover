use std::num::NonZero;
use bevy::{app::AppExit, prelude::{EventWriter, Res}};
use super::{BevyCancelIPC, ExitState};

pub fn stop_system(cancel_ipc: Res<BevyCancelIPC>, mut exit: EventWriter<AppExit>) {
  for state in cancel_ipc.channel.0.iter() {
    match state {
      ExitState::Success => {
        exit.send(AppExit::Success);
      },
      ExitState::Error => {
        exit.send(AppExit::Error(NonZero::new(1).unwrap()));
      },
    }
  }
}
