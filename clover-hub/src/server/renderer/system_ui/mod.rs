pub mod plugins;
pub mod systems;

use crate::server::modman::components::video::displays::models::PhysicalDisplayComponent;
use crate::utils::RecvSync;
use bevy::{
  app::{
    App,
    Startup,
  },
  diagnostic::{
    FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  log::LogPlugin,
  prelude::*,
};
use queues::*;
use systems::{
  displays::{
    display_registrar,
    vdisplay_registrar,
  },
  setup::setup,
};

pub enum ExitState {
  Success,
}

unsafe impl Sync for ExitState {}

#[derive(Resource)]
pub struct CustomBevyIPC {
  pub exit_channel: RecvSync<ExitState>,
  pub display_registration_queue: Queue<PhysicalDisplayComponent>,
}

pub fn system_ui_main(custom_bevy_ipc: CustomBevyIPC, disable_winit: Option<bool>) {
  let mut app = App::new();

  app
    .insert_resource(custom_bevy_ipc)
    .add_systems(Startup, setup)
    .add_systems(Update, (vdisplay_registrar, display_registrar).chain());

  #[cfg(feature = "compositor")]
  match disable_winit {
    Some(disable_winit_bool) => {
      if !disable_winit_bool {
        use bevy::{
          window::{
            PresentMode,
            Window,
            WindowPlugin,
            WindowTheme,
          },
          winit::{
            WakeUp,
            WinitPlugin,
          },
          DefaultPlugins,
        };
        use systems::simulated_controls::{
          make_visible,
          update_scroll_position,
        };

        let mut modded_winit: WinitPlugin<WakeUp> = Default::default();
        modded_winit.run_on_any_thread = true;

        app
          .add_plugins((
            DefaultPlugins
              .set(WindowPlugin {
                primary_window: Some(Window {
                  title: "Clover Simulated Controls".into(),
                  resolution: (500., 200.).into(),
                  present_mode: PresentMode::AutoVsync,
                  // Tells Wasm to resize the window according to the available canvas
                  fit_canvas_to_parent: true,
                  // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                  prevent_default_event_handling: false,
                  window_theme: Some(WindowTheme::Dark),
                  enabled_buttons: bevy::window::EnabledButtons {
                    maximize: false,
                    ..Default::default()
                  },
                  visible: false,
                  ..Default::default()
                }),
                ..Default::default()
              })
              .set(modded_winit)
              .disable::<LogPlugin>(),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
          ))
          .add_systems(Update, (make_visible, update_scroll_position));
      }
    }
    None => {}
  }

  app.run();
}
