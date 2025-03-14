use crate::server::renderer::system_ui::CustomBevyIPC;
use crate::server::{
  modman::components::video::displays::models::VirtualDisplayComponent,
  renderer::system_ui::systems::view_management::Composition,
};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
#[cfg(feature = "compositor")]
use bevy::window::{
  Monitor,
  WindowMode,
  WindowRef,
};
use queues::*;

pub fn vdisplay_registrar(mut commands: Commands, mut ipc: ResMut<CustomBevyIPC>) {}

pub fn display_registrar(
  mut commands: Commands,
  mut ipc: ResMut<CustomBevyIPC>,
  #[cfg(feature = "compositor")] monitor_entities_query: Query<(Entity, &Monitor)>,
  vdisplay_query: Query<(Entity, &mut VirtualDisplayComponent)>,
) {
  let display_queue = &mut ipc.display_registration_queue;

  let num_displays = display_queue.size();

  if num_displays > 0 {
    let mut displays = vec![];
    for _ in 0..(num_displays - 1) {
      match display_queue.remove() {
        Ok(display) => displays.push(display.clone()),
        Err(_) => {}
      }
    }

    for display in displays {
      match display.connection {
        #[cfg(feature = "compositor")]
        crate::server::modman::components::video::displays::models::ConnectionType::GPUDirect(
          direct_connection,
        ) => {
          let monitor = if direct_connection.display_id.clone() == "@primary" {
            MonitorSelection::Primary
          } else {
            let mut chosen_monitor = None;
            for (monitor_eid, monitor) in &monitor_entities_query {
              if monitor.name == Some(direct_connection.display_id.clone()) {
                chosen_monitor = Some(monitor_eid);
                break;
              }
            }

            match chosen_monitor {
              Some(monitor) => MonitorSelection::Entity(monitor),
              None => MonitorSelection::Primary,
            }
          };

          let windowed = if direct_connection.windowed {
            WindowMode::Windowed
          } else {
            WindowMode::BorderlessFullscreen(monitor.clone())
          };
          let position = if direct_connection.windowed {
            WindowPosition::Centered(monitor.clone())
          } else {
            WindowPosition::Automatic
          };

          match display.virtual_display {
            Some(vdisplay_id) => {
              match vdisplay_query.get(Entity::from_bits(vdisplay_id)) {
                Ok(vdisplay) => {
                  // TODO: Create a camera for this display.

                  commands.get_entity(vdisplay.0).unwrap().insert(Window {
                    title: "Clover SystemUI".into(),
                    mode: windowed,
                    position,
                    ..Default::default()
                  });
                }
                Err(e) => {}
              }
            }
            None => {
              // TODO: Spawn a composition. Create a camera for this display.

              let window = commands
                .spawn(Window {
                  title: format!("Clover SystemUI {}", direct_connection.display_id.clone()),
                  mode: windowed,
                  position,
                  ..Default::default()
                })
                .id();

              commands.spawn((
                Camera3d::default(),
                Camera {
                  target: RenderTarget::Window(WindowRef::Entity(window)),
                  ..default()
                },
                Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
              ));
            }
          }

          // TODO: Show new window!
        }
        crate::server::modman::components::video::displays::models::ConnectionType::ModManProxy(
          proxied_connection,
        ) => {}
      }
    }
  }
}
