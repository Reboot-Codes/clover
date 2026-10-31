//! # Clover SystemUI
//!
//! The job of SystemUI (and its various APIs) is to create a structured higherachy of attention to ensure that the instance's user is not overwhelmed, and has complete control of Clover.
//!
//! SystemUI has complete control over the content sent to displays through a direct connection to the GPU, or via a bus controlled by Modman. SystemUI will then compose a Composition of Views following X/Y/Z position resolved to relative to the origin of the display, blending modes when handling transparency, and more. SystemUI is heavily based on Bevy's ECS and learning the basics to how that works is suggested before working with CloverHub's default SystemUI implementation.
//!
//! ## Compositions
//!
//! SystemUI as a compositor must analyze the positions of Views within the context of one or more displays. (The latter scenario of multiple displays being handled by one composition is known as a virtual display, similar to ones in desktop operating systems.) A composition has a context that holds the theme, view positions, and global user input. Each client on EvtBuzz that is authorized to do so may own an arbitrary number of top level Composition Views in which it holds an absolute control of context within said view. Drawing over a view is possible if authorized, and Users may choose to permit the view covered to be informed of this in its context.
//!
//! ## Views
//!
//! Views are containers for graphics and context for a specific EvtBuzz Client and/or its inheritants. To save on system resources, views are split into the following types, ordered in least to most overhead.
//!
//! ### Composition Views
//!
//! Composition views use the Component API to render canned graphics commands in commonly used manners (like Buttons, Prompts, etc) and should automatically respect the User's theme unless authorized. (Verified using 3rd party ratings on app listings.) Composition views use the least resources as they are using known values (even when using custom components) which makes rendering easier on SystemUI as it already uses this system internally when composing all top-level views and their subviews when possible. They may also compose other views within themselves as previously mentioned with no extra overhead other than the new content to be rendered with or without passed in context when authorized by the framed view (minus when used as a top level view in a SystemUI implementation, of course).
//!
//! ### Canvas Views
//!
//! Canvas views act as a frame for direct graphics commands to the underlying Bevy library. For security, they are provided with their own ECS world which is composed into the one for the composition as a whole. They have more overhead than composition views as they must stream all operations to SystemUI through EvtBuzz (and Modman and/or the App Daemon if needed), but are less bandwidth-heavy than Stream Views as they rely on the GPU to expand the commands into actual graphics. It's possible to compose views inside of canvases using Tesseract to use a similar security flow to Compositon Views' Frames.
//!
//! ### Stream Views
//!
//! Stream views are used when rendering on the GPU is not possible (e.g. video stream) and all frames must be manually sent to SystemUI. Obviously, this is the heaviest View and is discouraged in favor of Canvas Views when possible. Stream views may compose other views within themselves as authorized via a frame-by-frame stream, however, this should not be used to send View data externally. For that, use CarbonSteel's [Mirroring API].
//!

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

/// Unified wrapper enum representing either a physical hardware display or a composited virtual display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnyDisplayComponent {
  /// Physical display hardware backed by a driver or bus proxy.
  Physical(PhysicalDisplayComponent),
  /// Virtual display composition aggregating multiple physical or logical views.
  Virtual(VirtualDisplayComponent),
}

/// IPC bridge and resource sharing state between the Tokio async runtime and the Bevy ECS world.
#[derive(Resource)]
pub struct SystemUIIPC {
  /// Receiver channel delivering graceful shutdown requests to Bevy.
  pub exit_channel: RecvSync<bevy::prelude::AppExit>,
  /// Queue of newly discovered or registered display components awaiting ECS initialization.
  pub display_registration_queue: Arc<Mutex<Queue<(String, AnyDisplayComponent)>>>,
  /// Shared Zenoh session handle for publishing alerts and frames.
  pub zenoh_session: Arc<zenoh::Session>,
  /// Tokio runtime handle allowing Bevy systems to spawn background async tasks.
  pub tokio_handle: tokio::runtime::Handle,
}

/// Bevy ECS system that monitors [`SystemUIIPC::exit_channel`] and initiates app exit when triggered.
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

/// Main entrypoint that builds and runs the Bevy [`App`] powering Clover's System UI.
///
/// Configures rendering plugins, displays, offscreen frame capturing, and the simulated controls testing interface.
pub fn system_ui_main(custom_bevy_ipc: SystemUIIPC, disable_winit: Option<bool>) {
  let mut app = App::new();

  app
    .insert_resource(custom_bevy_ipc)
    // .add_systems(Startup, setup)
    .add_systems(
      Update,
      (
        systems::displays::display_registrar,
        systems::displays::capture_proxied_displays,
        shutdown_system,
      ),
    );

  #[cfg(feature = "compositor")]
  match disable_winit {
    Some(disable_winit_bool) => {
      if !disable_winit_bool {
        use bevy::{
          render::{
            settings::{Backends, RenderCreation, WgpuSettings},
            RenderPlugin,
          },
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
          handle_sim_keyboard_controls,
          sync_testing_window_displays,
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
                  title: "Clover - Controls & Displays".into(),
                  resolution: (640.0, 520.0).into(),
                  present_mode: PresentMode::AutoVsync,
                  // Tells Wasm to resize the window according to the available canvas
                  fit_canvas_to_parent: true,
                  // Tells Wasm not to override default event handling, like F5, Ctrl+R etc.
                  prevent_default_event_handling: false,
                  window_theme: Some(WindowTheme::Dark),
                  enabled_buttons: bevy::window::EnabledButtons {
                    maximize: true,
                    ..Default::default()
                  },
                  visible: true,
                  ..Default::default()
                }),
                ..Default::default()
              })
              .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                  backends: Some(Backends::from_env().unwrap_or(Backends::PRIMARY)),
                  ..Default::default()
                }),
                ..Default::default()
              })
              .set(modded_winit)
              .disable::<LogPlugin>(),
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin::default(),
          ))
          .add_systems(Startup, sim_setup)
          .add_systems(
            Update,
            (
              update_scroll_position,
              sync_testing_window_displays,
              handle_sim_keyboard_controls,
            ),
          );
      }
    }
    None => {}
  }

  app.run();
}
