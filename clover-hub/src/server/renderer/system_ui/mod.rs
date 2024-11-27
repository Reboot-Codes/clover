mod systems;

use bevy::{app::{App, Startup}, prelude::Resource};
use systems::stop_system;
use crate::utils::RecvSync;

pub enum ExitState {
  Success,
  Error,
}

unsafe impl Sync for ExitState {}

#[derive(Resource)]
pub struct BevyCancelIPC {
  pub channel: RecvSync<ExitState>
}

pub fn system_ui_main(bevy_cancel_ipc: BevyCancelIPC) {
  App::new()
    .insert_resource(bevy_cancel_ipc)
    .add_systems(Startup, stop_system)
    .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
    .run();
}
