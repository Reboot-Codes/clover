use crate::server::renderer::system_ui::{
  AnyDisplayComponent,
  SystemUIIPC,
};
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
use log::warn;
use queues::*;

#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct DisplayWindow {
  pub id: String,
}

#[derive(Component)]
pub struct DisplayCamera {
  pub id: String,
}

#[derive(Component)]
pub struct DisplayComposition {
  pub id: String,
}

#[derive(Component)]
pub struct VirtualDisplayID {
  pub id: String,
}

pub fn display_registrar(
  mut commands: Commands,
  ipc: Res<SystemUIIPC>,
  #[cfg(feature = "compositor")] monitor_entities_query: Query<(Entity, &Monitor)>,
  vdisplay_query: Query<(Entity, &mut VirtualDisplayComponent, &VirtualDisplayID)>,
) {
  let ipc_display_reg_queue_clone = ipc.display_registration_queue.clone();
  let mut display_queue = ipc_display_reg_queue_clone.lock().unwrap();

  match display_queue.remove() {
    Ok(display_config) => {
      let display_id = display_config.0;
      let any_display = display_config.1;

      debug!("Found display to register: {}", display_id.clone());

      match any_display {
        AnyDisplayComponent::Physical(physical_display_component) => {
          match physical_display_component.connection {
            #[cfg(feature = "compositor")]
            crate::server::modman::components::video::displays::models::ConnectionType::Direct(
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
                  None => {
                    // TODO: Is this really the best behaviour?
                    warn!("Requested display not found, defaulting to primary display...");
                    MonitorSelection::Primary
                  }
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

              match physical_display_component.virtual_display {
                Some(vdisplay_id) => {
                  for (
                    queried_vdisplay_entity, 
                    _queried_vdisplay_component, 
                    queried_vdisplay_id
                  ) in &vdisplay_query {
                    if vdisplay_id == queried_vdisplay_id.id {
                      use bevy::window::WindowTheme;
                  
                      let window = commands.get_entity(queried_vdisplay_entity).unwrap().insert((Window {
                        title: format!("Clover SystemUI: Virtual Display: {}", display_id.clone()),
                        mode: windowed,
                        window_theme: Some(WindowTheme::Dark),
                        position,
                        ..Default::default()
                      }, DisplayWindow { id: vdisplay_id.to_string() })).id();
                  
                      commands.spawn((
                        Camera3d::default(),
                        Camera {
                          target: RenderTarget::Window(WindowRef::Entity(window)),
                          ..default()
                        },
                        Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                        DisplayCamera { id: vdisplay_id.to_string() }
                      ));
                      
                      break;
                    }
                  }
                }
                None => {
                  // TODO: Spawn a composition for this display.

                  use bevy::window::WindowTheme;

                  let window = commands
                    .spawn((Window {
                      title: format!("Clover SystemUI: Display: {}", display_id.clone()),
                      mode: windowed,
                      window_theme: Some(WindowTheme::Dark),
                      position,
                      ..Default::default()
                    }, DisplayWindow { id: display_id.clone() }))
                    .id();

                  commands.spawn((
                    Camera3d::default(),
                    Camera {
                      target: RenderTarget::Window(WindowRef::Entity(window)),
                      ..default()
                    },
                    Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                    DisplayCamera { id: display_id.clone() }
                  ));
                }
              }
            }
            crate::server::modman::components::video::displays::models::ConnectionType::ModManProxy(
              proxied_connection,
            ) => {}
            crate::server::modman::components::video::displays::models::ConnectionType::Stream(
              stream_config,
            ) => {}
          }
        }
        AnyDisplayComponent::Virtual(virtual_display_component) => {
          debug!("Spawing VDisplay: {}", display_id.clone());
          // TODO: Spawn a composition for this display.

          commands.spawn((virtual_display_component, VirtualDisplayID { id: display_id.clone() }));

          debug!("Done spawning VDisplay: {}!", display_id.clone());
        }
      }
    }
    Err(_e) => {}
  }
}
