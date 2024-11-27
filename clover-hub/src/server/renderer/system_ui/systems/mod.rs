use std::num::NonZero;
use bevy::{app::AppExit, prelude::{EventWriter, Res}};
use log::debug;
use super::{BevyCancelIPC, ExitState};

pub fn stop_system(cancel_ipc: Res<BevyCancelIPC>, mut exit: EventWriter<AppExit>) {
  debug!("Stop system registered.");

  for state in cancel_ipc.channel.0.iter() {
    match state {
      ExitState::Success => {
        debug!("Stopping properly!");
        exit.send(AppExit::Success);
      },
      ExitState::Error => {
        debug!("Stopping forcibly!");
        exit.send(AppExit::Error(NonZero::new(1).unwrap()));
      },
    }
  }
}
