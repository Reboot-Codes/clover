mod plugins;
mod systems;

use bevy::{a11y::AccessibilityNode, app::{App, Startup}, core::FrameCount, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}, ecs::system::Query, input::mouse::{MouseScrollUnit, MouseWheel}, log::LogPlugin, prelude::*, window::{PresentMode, Window, WindowPlugin, WindowTheme}, winit::{WakeUp, WinitPlugin}, DefaultPlugins};
use systems::stop_system;
use crate::utils::RecvSync;
use bevy::color::palettes::basic::*;

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
  let mut modded_winit: WinitPlugin::<WakeUp> = Default::default();
  modded_winit.run_on_any_thread = true;

  App::new()
    .insert_resource(bevy_cancel_ipc)
    //.add_systems(Startup, stop_system)
    .add_plugins(bevy_tokio_tasks::TokioTasksPlugin::default())
    .add_plugins((
      DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
          title: "I am a window!".into(),
          name: Some("bevy.app".into()),
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
          // This will spawn an invisible window
          // The window will be made visible in the make_visible() system after 3 frames.
          // This is useful when you want to avoid the white window that shows up before the GPU is ready to render the app.
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
    .add_systems(
      Update, make_visible
    )
    .add_systems(Startup, setup)
    .run();
}

fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
  // info!("Frame: {}", frames.0);
  // The delay may be different for your app or system.
  if frames.0 == 3 {
    // At this point the gpu is ready to show the app so we can make the window visible.
    // Alternatively, you could toggle the visibility in Startup.
    // It will work, but it will have one white frame before it starts rendering
    window.get_single_mut().unwrap().visible = true;
  }
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
  // Camera
  commands.spawn((Camera2dBundle::default(), IsDefaultUiCamera));

  // root node
  commands
      .spawn(NodeBundle {
          style: Style {
              width: Val::Percent(100.0),
              height: Val::Percent(100.0),
              justify_content: JustifyContent::SpaceBetween,
              ..default()
          },
          ..default()
      })
      .with_children(|parent| {
          // left vertical fill (border)
          parent
              .spawn(NodeBundle {
                  style: Style {
                      width: Val::Px(200.),
                      border: UiRect::all(Val::Px(2.)),
                      ..default()
                  },
                  background_color: Color::srgb(0.65, 0.65, 0.65).into(),
                  ..default()
              })
              .with_children(|parent| {
                  // left vertical fill (content)
                  parent
                      .spawn(NodeBundle {
                          style: Style {
                              width: Val::Percent(100.),
                              flex_direction: FlexDirection::Column,
                              padding: UiRect::all(Val::Px(5.)),
                              row_gap: Val::Px(5.),
                              ..default()
                          },
                          background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                          ..default()
                      })
                      .with_children(|parent| {
                          // text
                          parent.spawn((
                              TextBundle::from_section(
                                  "Text Example",
                                  TextStyle {
                                      font_size: 30.0,
                                      ..default()
                                  },
                              ),
                              // Because this is a distinct label widget and
                              // not button/list item text, this is necessary
                              // for accessibility to treat the text accordingly.
                              Label,
                          ));
                      });
              });
          // right vertical fill
          parent
              .spawn(NodeBundle {
                  style: Style {
                      flex_direction: FlexDirection::Column,
                      justify_content: JustifyContent::Center,
                      align_items: AlignItems::Center,
                      width: Val::Px(200.),
                      ..default()
                  },
                  background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                  ..default()
              })
              .with_children(|parent| {
                  // Title
                  parent.spawn((
                      TextBundle::from_section(
                          "Scrolling list",
                          TextStyle {
                              font_size: 25.,
                              ..default()
                          },
                      ),
                      Label,
                  ));
                  // List with hidden overflow
                  parent
                      .spawn(NodeBundle {
                          style: Style {
                              flex_direction: FlexDirection::Column,
                              align_self: AlignSelf::Stretch,
                              height: Val::Percent(50.),
                              overflow: Overflow::clip_y(),
                              ..default()
                          },
                          background_color: Color::srgb(0.10, 0.10, 0.10).into(),
                          ..default()
                      })
                      .with_children(|parent| {
                          // Moving panel
                          parent
                              .spawn((
                                  NodeBundle {
                                      style: Style {
                                          flex_direction: FlexDirection::Column,
                                          align_items: AlignItems::Center,
                                          ..default()
                                      },
                                      ..default()
                                  },
                                  ScrollingList::default(),
                              ))
                              .with_children(|parent| {
                                  // List items
                                  for i in 0..30 {
                                      parent.spawn((
                                          TextBundle::from_section(
                                              format!("Item {i}"),
                                              TextStyle {
                                                  ..default()
                                              },
                                          ),
                                          Label,
                                      ));
                                  }
                              });
                      });
              });
          parent
              .spawn(NodeBundle {
                  style: Style {
                      width: Val::Px(200.0),
                      height: Val::Px(200.0),
                      position_type: PositionType::Absolute,
                      left: Val::Px(210.),
                      bottom: Val::Px(10.),
                      border: UiRect::all(Val::Px(20.)),
                      ..default()
                  },
                  border_color: LIME.into(),
                  background_color: Color::srgb(0.4, 0.4, 1.).into(),
                  ..default()
              })
              .with_children(|parent| {
                  parent.spawn(NodeBundle {
                      style: Style {
                          width: Val::Percent(100.0),
                          height: Val::Percent(100.0),
                          ..default()
                      },
                      background_color: Color::srgb(0.8, 0.8, 1.).into(),
                      ..default()
                  });
              });
          // render order test: reddest in the back, whitest in the front (flex center)
          parent
              .spawn(NodeBundle {
                  style: Style {
                      width: Val::Percent(100.0),
                      height: Val::Percent(100.0),
                      position_type: PositionType::Absolute,
                      align_items: AlignItems::Center,
                      justify_content: JustifyContent::Center,
                      ..default()
                  },
                  ..default()
              })
              .with_children(|parent| {
                  parent
                      .spawn(NodeBundle {
                          style: Style {
                              width: Val::Px(100.0),
                              height: Val::Px(100.0),
                              ..default()
                          },
                          background_color: Color::srgb(1.0, 0.0, 0.).into(),
                          ..default()
                      })
                      .with_children(|parent| {
                          parent.spawn(NodeBundle {
                              style: Style {
                                  // Take the size of the parent node.
                                  width: Val::Percent(100.0),
                                  height: Val::Percent(100.0),
                                  position_type: PositionType::Absolute,
                                  left: Val::Px(20.),
                                  bottom: Val::Px(20.),
                                  ..default()
                              },
                              background_color: Color::srgb(1.0, 0.3, 0.3).into(),
                              ..default()
                          });
                          parent.spawn(NodeBundle {
                              style: Style {
                                  width: Val::Percent(100.0),
                                  height: Val::Percent(100.0),
                                  position_type: PositionType::Absolute,
                                  left: Val::Px(40.),
                                  bottom: Val::Px(40.),
                                  ..default()
                              },
                              background_color: Color::srgb(1.0, 0.5, 0.5).into(),
                              ..default()
                          });
                          parent.spawn(NodeBundle {
                              style: Style {
                                  width: Val::Percent(100.0),
                                  height: Val::Percent(100.0),
                                  position_type: PositionType::Absolute,
                                  left: Val::Px(60.),
                                  bottom: Val::Px(60.),
                                  ..default()
                              },
                              background_color: Color::srgb(1.0, 0.7, 0.7).into(),
                              ..default()
                          });
                          // alpha test
                          parent.spawn(NodeBundle {
                              style: Style {
                                  width: Val::Percent(100.0),
                                  height: Val::Percent(100.0),
                                  position_type: PositionType::Absolute,
                                  left: Val::Px(80.),
                                  bottom: Val::Px(80.),
                                  ..default()
                              },
                              background_color: Color::srgba(1.0, 0.9, 0.9, 0.4).into(),
                              ..default()
                          });
                      });
              });
          // bevy logo (flex center)
          parent
              .spawn(NodeBundle {
                  style: Style {
                      width: Val::Percent(100.0),
                      position_type: PositionType::Absolute,
                      justify_content: JustifyContent::Center,
                      align_items: AlignItems::FlexStart,
                      ..default()
                  },
                  ..default()
              });
      });
}

#[derive(Component, Default)]
struct ScrollingList {
  position: f32,
}

fn mouse_scroll(
  mut mouse_wheel_events: EventReader<MouseWheel>,
  mut query_list: Query<(&mut ScrollingList, &mut Style, &Parent, &Node)>,
  query_node: Query<&Node>,
) {
  for mouse_wheel_event in mouse_wheel_events.read() {
      for (mut scrolling_list, mut style, parent, list_node) in &mut query_list {
          let items_height = list_node.size().y;
          let container_height = query_node.get(parent.get()).unwrap().size().y;

          let max_scroll = (items_height - container_height).max(0.);

          let dy = match mouse_wheel_event.unit {
              MouseScrollUnit::Line => mouse_wheel_event.y * 20.,
              MouseScrollUnit::Pixel => mouse_wheel_event.y,
          };

          scrolling_list.position += dy;
          scrolling_list.position = scrolling_list.position.clamp(-max_scroll, 0.);
          style.top = Val::Px(scrolling_list.position);
      }
  }
}
