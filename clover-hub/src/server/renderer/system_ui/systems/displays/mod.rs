use crate::server::renderer::system_ui::CustomBevyIPC;
use bevy::prelude::*;
#[cfg(feature = "compositor")]
use bevy::window::{Monitor, WindowMode};
use queues::*;

pub fn display_registrar(
    mut commands: Commands,
    mut ipc: ResMut<CustomBevyIPC>,
    #[cfg(feature = "compositor")] monitor_entities_query: Query<(Entity, &Monitor)>,
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
              crate::server::modman::components::video::displays::models::ConnectionType::GPUDirect(direct_connection) => {
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
                        Some(monitor) => {
                          MonitorSelection::Entity(monitor)
                        },
                        None => {
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

                    let window = commands.spawn(Window {
                        title: "Clover SystemUI".into(),
                        mode: windowed,
                        position,
                        ..Default::default()
                    });

                    // TODO: Show new window!
                },
                crate::server::modman::components::video::displays::models::ConnectionType::ModManProxy(proxied_connection) => {

                },
            }
        }
    }
}
