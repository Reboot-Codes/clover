pub mod plugins;
pub mod systems;

use std::sync::{
  Arc,
  Mutex,
};

use crate::server::modman::components::video::displays::models::{
  PhysicalDisplayComponent,
  VirtualDisplayComponent,
};
use crate::utils::RecvSync;
use bevy::{
  app::App,
  diagnostic::{
    FrameTimeDiagnosticsPlugin,
    LogDiagnosticsPlugin,
  },
  log::LogPlugin,
  prelude::*,
};
use queues::*;
use serde::{
  Deserialize,
  Serialize,
};
use systems::displays::display_registrar;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyDisplayComponent {
  Physical(PhysicalDisplayComponent),
  Virtual(VirtualDisplayComponent),
}

#[derive(Resource)]
pub struct SystemUIIPC {
  pub exit_channel: RecvSync<bevy::prelude::AppExit>,
  pub display_registration_queue: Arc<Mutex<Queue<(String, AnyDisplayComponent)>>>,
}

pub fn shutdown_system(
  ipc: Res<SystemUIIPC>,
  mut app_exit_events: ResMut<Events<bevy::app::AppExit>>,
) {
  match ipc.exit_channel.0.try_recv() {
    Ok(exit_state) => {
      app_exit_events.send(exit_state);
    }
    Err(_) => {}
  }
}

pub fn system_ui_main(custom_bevy_ipc: SystemUIIPC, disable_winit: Option<bool>) {
  let mut app = App::new();

  app
    .insert_resource(custom_bevy_ipc)
    // .add_systems(Startup, setup)
    .add_systems(Update, display_registrar)
    .add_systems(Update, shutdown_system);

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

        use crate::server::renderer::system_ui::systems::simulated_controls::sim_setup;

        let mut modded_winit: WinitPlugin<WakeUp> = Default::default();
        modded_winit.run_on_any_thread = true;

        app
          .add_plugins((
            DefaultPlugins
              .set(WindowPlugin {
                primary_window: Some(Window {
                  title: "Clover".into(),
                  resolution: (100.0, 100.0).into(),
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
          .add_systems(Startup, sim_setup)
          .add_systems(Update, (make_visible, update_scroll_position));
      }
    }
    None => {}
  }

  app.run();
}
