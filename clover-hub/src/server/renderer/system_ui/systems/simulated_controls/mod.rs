//! # Simulated Controls & Display Previews
//!
//! Provides a unified desktop testing interface hosted directly on Bevy's [`bevy::window::PrimaryWindow`].
//!
//! Features include:
//! - **Side-by-Side Display Previews**: Live offscreen-rendered texture previews of the
//!   HUD OLED interior display (128x64 previewed at 256x128) and the Eye LCD exterior display (240x240).
//! - **Cross D-Pad**: Directional control pad (`UP`, `DOWN`, `LEFT`, `RIGHT`) bound to both mouse clicks
//!   and keyboard shortcuts (`W`/`S`/`A`/`D` and Arrow keys).
//! - **Action Buttons**: Function buttons `1` (Left-Click/Select: `Enter`/`E`), `2` (Right-Click/Context: `Tab`/`Q`),
//!   `BACK` (`Escape`), and `HOME` (`H`).
//! - **Zenoh Bus Integration**: Emits momentary contact [`carbon_fiber::alerts::models::components::AlertMessage`]
//!   pulses (`active: true`/`false`) to `.../components/by-id/{id}/alerts` and replies to state queries
//!   at `.../components/by-id/{id}/state`.

#[cfg(feature = "compositor")]
use bevy::window::Monitor;
use bevy::{
  diagnostic::FrameCount,
  ecs::hierarchy::ChildSpawnerCommands,
  ecs::system::Query,
  input::mouse::{
    MouseScrollUnit,
    MouseWheel,
  },
  picking::hover::HoverMap,
  picking::Pickable,
  prelude::*,
  ui::UiTargetCamera,
  window::Window,
};
use bevy::{
  render::camera::RenderTarget,
  window::WindowRef,
};
use std::collections::HashMap;
use std::sync::{
  Arc,
  Mutex,
};

use crate::server::renderer::system_ui::{
  systems::displays::ProxiedDisplayTarget,
  SystemUIIPC,
};

/// Marker component attached to the primary window hosting the simulated controls and previews.
#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct SimulatedControlsWindow {}

/// Marker component for the 2D UI camera rendering the simulated controls interface.
#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct SimulatedControlsCamera {}

/// Root UI layout container for the simulated controls and display previews.
#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct SimulatedControlsRootNode {}

/// Marker identifying the UI node displaying the live interior display texture.
#[derive(Component)]
pub struct InternalDisplayPreviewMarker;

/// Marker identifying the UI node displaying the live exterior display texture.
#[derive(Component)]
pub struct ExternalDisplayPreviewMarker;

/// Metadata component attached to interactive simulated buttons for visual state highlighting.
#[derive(Component)]
pub struct SimButtonMarker {
  /// Unique component identifier on the Zenoh bus (e.g. `"sim_btn_up"`).
  pub id: String,
  /// Idle background color.
  pub default_color: Color,
  /// Highlighted background color when pressed via mouse or keyboard.
  pub active_color: Color,
}

/// Shared thread-safe map of momentary button states (`component_id -> bool`).
#[derive(Resource, Clone)]
pub struct SimButtonStates(pub Arc<Mutex<HashMap<String, bool>>>);

/// Publishes a momentary alert and state update to Zenoh for a simulated sensor component.
pub async fn publish_sim_sensor_alert(
  session: &zenoh::Session,
  comp_id: &str,
  active: bool,
) {
  use carbon_fiber::alerts::models::components::{
    AlertMessage,
    AlertPayload,
    AlertSeverity,
  };
  use crate::server::modman::MODULE_EVT_ID;

  let timestamp_ms = (std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .unwrap_or_default()
    .as_millis() % 65536) as u16;

  let alert = AlertMessage {
    component_id: 0,
    severity: AlertSeverity::Info,
    timestamp_ms,
    payload: AlertPayload::DigitalContact {
      pin_index: 0,
      active,
    },
  };

  let alert_topic = format!("{MODULE_EVT_ID}/components/by-id/{comp_id}/alerts");
  if let Ok(alert_json) = serde_json::to_string(&alert) {
    if let Err(e) = session.put(&alert_topic, alert_json).await {
      log::warn!("Failed to publish sim sensor alert to {alert_topic}: {e}");
    } else {
      log::debug!("Published sim sensor alert (active={active}) to {alert_topic}");
    }
  }

  let state_topic = format!("{MODULE_EVT_ID}/components/by-id/{comp_id}/state");
  let state_json = serde_json::json!({ "active": active }).to_string();
  let _ = session.put(&state_topic, state_json).await;
}

/// Spawns an interactive simulated button with mouse pointer trigger handlers and text labels.
pub fn sim_button(
  parent: &mut ChildSpawnerCommands<'_>,
  color: Color,
  label: &str,
  component_id: &str,
  session: Arc<zenoh::Session>,
  tokio_handle: tokio::runtime::Handle,
  states: Arc<Mutex<HashMap<String, bool>>>,
  width_px: f32,
  height_px: f32,
) {
  let comp_id_down = component_id.to_string();
  let session_down = session.clone();
  let handle_down = tokio_handle.clone();
  let states_down = states.clone();

  let comp_id_up = component_id.to_string();
  let session_up = session.clone();
  let handle_up = tokio_handle.clone();
  let states_up = states.clone();

  let active_color = Color::srgb(1.0, 1.0, 1.0);

  parent
    .spawn((
      Node {
        width: Val::Px(width_px),
        height: Val::Px(height_px),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border: UiRect::all(Val::Px(1.5)),
        margin: UiRect::all(Val::Px(2.5)),
        ..default()
      },
      BorderColor(Color::srgb(0.35, 0.35, 0.35)),
      BackgroundColor(color),
      SimButtonMarker {
        id: component_id.to_string(),
        default_color: color,
        active_color,
      },
    ))
    .insert(Pickable {
      should_block_lower: false,
      is_hoverable: true,
    })
    .with_children(|button| {
      button.spawn((
        Text::new(label),
        TextFont {
          font_size: 10.5,
          ..Default::default()
        },
        TextLayout::new_with_justify(JustifyText::Center),
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
      ));
    })
    .observe(move |_: Trigger<Pointer<Pressed>>| {
      let comp_id = comp_id_down.clone();
      let session = session_down.clone();
      {
        let mut map = states_down.lock().unwrap();
        map.insert(comp_id.clone(), true);
      }
      handle_down.spawn(async move {
        publish_sim_sensor_alert(&session, &comp_id, true).await;
      });
    })
    .observe(move |_: Trigger<Pointer<Released>>| {
      let comp_id = comp_id_up.clone();
      let session = session_up.clone();
      {
        let mut map = states_up.lock().unwrap();
        map.insert(comp_id.clone(), false);
      }
      handle_up.spawn(async move {
        publish_sim_sensor_alert(&session, &comp_id, false).await;
      });
    });
}

/// Bevy startup system that attaches the simulated controls layout and display preview viewports
/// directly to the primary window.
pub fn sim_setup(
  mut commands: Commands,
  ipc: Res<SystemUIIPC>,
  #[cfg(feature = "compositor")] primary_window_query: Query<Entity, With<bevy::window::PrimaryWindow>>,
  #[cfg(feature = "compositor")] monitor_entities_query: Query<(Entity, &Monitor)>,
) {
  #[cfg(feature = "compositor")]
  {
    debug!("Available Monitors:");
    for (monitor_eid, monitor) in &monitor_entities_query {
      debug!(
        "Monitor {} with EntityID of {}.",
        monitor.name.clone().unwrap_or("unknown".into()),
        monitor_eid.to_bits()
      );
    }

    let Ok(primary_window_entity) = primary_window_query.single() else {
      warn!("PrimaryWindow not found in sim_setup!");
      return;
    };

    commands.entity(primary_window_entity).insert(SimulatedControlsWindow {});

    let ui_camera = commands
      .spawn((
        Camera2d::default(),
        IsDefaultUiCamera,
        SimulatedControlsCamera {},
        Camera {
          target: RenderTarget::Window(WindowRef::Entity(primary_window_entity)),
          ..default()
        },
      ))
      .id();

    let button_states: Arc<Mutex<HashMap<String, bool>>> = Arc::new(Mutex::new(HashMap::new()));
    commands.insert_resource(SimButtonStates(button_states.clone()));

    // Expose Zenoh queryables for each simulated button
    let sim_buttons = [
      ("sim_btn_up", "UP\n(W/↑)", Color::srgb(0.75, 0.75, 0.75)),
      ("sim_btn_down", "DOWN\n(S/↓)", Color::srgb(0.75, 0.75, 0.75)),
      ("sim_btn_left", "LEFT\n(A/←)", Color::srgb(0.75, 0.75, 0.75)),
      ("sim_btn_right", "RIGHT\n(D/→)", Color::srgb(0.75, 0.75, 0.75)),
      ("sim_btn_1", "1 [L-Click]\n(E/Enter)", Color::srgb(0.3, 0.8, 0.3)),
      ("sim_btn_2", "2 [R-Click]\n(Q/Tab)", Color::srgb(0.3, 0.5, 0.9)),
      ("sim_btn_back", "BACK\n(Esc)", Color::srgb(0.85, 0.3, 0.3)),
      ("sim_btn_home", "HOME\n(H)", Color::srgb(0.9, 0.8, 0.2)),
    ];

    for (btn_id, _, _) in sim_buttons {
      let session_q = ipc.zenoh_session.clone();
      let states_q = button_states.clone();
      let btn_q = btn_id.to_string();

      ipc.tokio_handle.spawn(async move {
        let key_expr = format!("com/reboot-codes/clover/hub/modman/components/by-id/{btn_q}/state");
        if let Ok(queryable) = session_q.declare_queryable(&key_expr).await {
          while let Ok(query) = queryable.recv_async().await {
            let is_active = {
              let map = states_q.lock().unwrap();
              map.get(&btn_q).copied().unwrap_or(false)
            };
            let reply_json = serde_json::json!({ "active": is_active }).to_string();
            let _ = query.reply(&key_expr, reply_json).await;
          }
        }
      });
    }

    // Spawn unified vertical stack inside PrimaryWindow
    commands
      .spawn((
        Node {
          width: Val::Percent(100.0),
          height: Val::Percent(100.0),
          flex_direction: FlexDirection::Column,
          justify_content: JustifyContent::SpaceBetween,
          align_items: AlignItems::Center,
          padding: UiRect::all(Val::Px(10.0)),
          ..default()
        },
        UiTargetCamera(ui_camera),
        SimulatedControlsRootNode {},
      ))
      .insert(Pickable {
        should_block_lower: false,
        is_hoverable: false,
      })
      .with_children(|root| {
        // --- TOP SECTION: Displays Preview Side-by-Side ---
        root
          .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            column_gap: Val::Px(16.0),
            padding: UiRect::vertical(Val::Px(4.0)),
            ..default()
          })
          .with_children(|displays_row| {
            // Interior Display Container (128x64 HUD OLED, previewed at 256x128)
            displays_row
              .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
              })
              .with_children(|col| {
                col.spawn((
                  Text::new("Interior Display (128x64 HUD)"),
                  TextFont {
                    font_size: 11.0,
                    ..default()
                  },
                  TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));

                col.spawn((
                  Node {
                    width: Val::Px(256.0),
                    height: Val::Px(128.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                  },
                  BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                  BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                  InternalDisplayPreviewMarker,
                ));
              });

            // Exterior Display Container (240x240 Eye LCD, previewed at 240x240)
            displays_row
              .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(4.0),
                ..default()
              })
              .with_children(|col| {
                col.spawn((
                  Text::new("Exterior Display (240x240 Eye)"),
                  TextFont {
                    font_size: 11.0,
                    ..default()
                  },
                  TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));

                col.spawn((
                  Node {
                    width: Val::Px(240.0),
                    height: Val::Px(240.0),
                    border: UiRect::all(Val::Px(2.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                  },
                  BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                  BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
                  ExternalDisplayPreviewMarker,
                ));
              });
          });

        // --- BOTTOM SECTION: Controls Row (D-Pad on bottom-left, Action buttons on bottom-right) ---
        root
          .spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::SpaceAround,
            align_items: AlignItems::Center,
            margin: UiRect::top(Val::Px(4.0)),
            ..default()
          })
          .with_children(|bottom_row| {
            // --- Bottom-Left: Cross D-Pad ---
            bottom_row
              .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(2.0),
                ..default()
              })
              .with_children(|dpad| {
                // Top row: [spacer, UP, spacer]
                dpad
                  .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                  })
                  .with_children(|r| {
                    r.spawn(Node { width: Val::Px(56.0), height: Val::Px(40.0), ..default() });
                    sim_button(
                      r,
                      sim_buttons[0].2,
                      sim_buttons[0].1,
                      sim_buttons[0].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      56.0,
                      40.0,
                    );
                    r.spawn(Node { width: Val::Px(56.0), height: Val::Px(40.0), ..default() });
                  });

                // Middle row: [LEFT, center spacer, RIGHT]
                dpad
                  .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                  })
                  .with_children(|r| {
                    sim_button(
                      r,
                      sim_buttons[2].2,
                      sim_buttons[2].1,
                      sim_buttons[2].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      56.0,
                      40.0,
                    );
                    r.spawn((
                      Node {
                        width: Val::Px(56.0),
                        height: Val::Px(40.0),
                        border: UiRect::all(Val::Px(1.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        margin: UiRect::all(Val::Px(2.5)),
                        ..default()
                      },
                      BorderColor(Color::srgb(0.2, 0.2, 0.2)),
                      BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
                    ));
                    sim_button(
                      r,
                      sim_buttons[3].2,
                      sim_buttons[3].1,
                      sim_buttons[3].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      56.0,
                      40.0,
                    );
                  });

                // Bottom row: [spacer, DOWN, spacer]
                dpad
                  .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                  })
                  .with_children(|r| {
                    r.spawn(Node { width: Val::Px(56.0), height: Val::Px(40.0), ..default() });
                    sim_button(
                      r,
                      sim_buttons[1].2,
                      sim_buttons[1].1,
                      sim_buttons[1].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      56.0,
                      40.0,
                    );
                    r.spawn(Node { width: Val::Px(56.0), height: Val::Px(40.0), ..default() });
                  });
              });

            // --- Bottom-Right: Action Buttons (2x2 Grid) ---
            bottom_row
              .spawn(Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(6.0),
                ..default()
              })
              .with_children(|actions| {
                // Action Row 1: [1 (E/Enter), 2 (Q/Tab)]
                actions
                  .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                  })
                  .with_children(|r| {
                    sim_button(
                      r,
                      sim_buttons[4].2,
                      sim_buttons[4].1,
                      sim_buttons[4].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      96.0,
                      42.0,
                    );
                    sim_button(
                      r,
                      sim_buttons[5].2,
                      sim_buttons[5].1,
                      sim_buttons[5].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      96.0,
                      42.0,
                    );
                  });

                // Action Row 2: [BACK (Esc), HOME (H)]
                actions
                  .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    ..default()
                  })
                  .with_children(|r| {
                    sim_button(
                      r,
                      sim_buttons[6].2,
                      sim_buttons[6].1,
                      sim_buttons[6].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      96.0,
                      42.0,
                    );
                    sim_button(
                      r,
                      sim_buttons[7].2,
                      sim_buttons[7].1,
                      sim_buttons[7].0,
                      ipc.zenoh_session.clone(),
                      ipc.tokio_handle.clone(),
                      button_states.clone(),
                      96.0,
                      42.0,
                    );
                  });
              });
          });
      });

    debug!("Spawned unified debug testing window on PrimaryWindow!");
  }
}

/// Handles keyboard shortcuts for simulated buttons (WASD + Arrows, E/Enter, Q/Tab, Esc, H)
pub fn handle_sim_keyboard_controls(
  keyboard_input: Res<ButtonInput<KeyCode>>,
  ipc: Res<SystemUIIPC>,
  button_states: Option<Res<SimButtonStates>>,
  mut button_query: Query<(&SimButtonMarker, &mut BackgroundColor)>,
) {
  let Some(button_states) = button_states else { return; };

  let key_mappings: [(&str, &[KeyCode]); 8] = [
    ("sim_btn_up", &[KeyCode::KeyW, KeyCode::ArrowUp]),
    ("sim_btn_down", &[KeyCode::KeyS, KeyCode::ArrowDown]),
    ("sim_btn_left", &[KeyCode::KeyA, KeyCode::ArrowLeft]),
    ("sim_btn_right", &[KeyCode::KeyD, KeyCode::ArrowRight]),
    ("sim_btn_1", &[KeyCode::Enter, KeyCode::KeyE]),
    ("sim_btn_2", &[KeyCode::Tab, KeyCode::KeyQ]),
    ("sim_btn_back", &[KeyCode::Escape]),
    ("sim_btn_home", &[KeyCode::KeyH]),
  ];

  for (comp_id, keys) in key_mappings {
    let any_just_pressed = keys.iter().any(|k| keyboard_input.just_pressed(*k));
    let any_just_released = keys.iter().any(|k| keyboard_input.just_released(*k));
    let any_pressed = keys.iter().any(|k| keyboard_input.pressed(*k));

    if any_just_pressed {
      {
        let mut map = button_states.0.lock().unwrap();
        map.insert(comp_id.to_string(), true);
      }
      let session = ipc.zenoh_session.clone();
      let id = comp_id.to_string();
      ipc.tokio_handle.spawn(async move {
        publish_sim_sensor_alert(&session, &id, true).await;
      });
    } else if any_just_released && !any_pressed {
      {
        let mut map = button_states.0.lock().unwrap();
        map.insert(comp_id.to_string(), false);
      }
      let session = ipc.zenoh_session.clone();
      let id = comp_id.to_string();
      ipc.tokio_handle.spawn(async move {
        publish_sim_sensor_alert(&session, &id, false).await;
      });
    }
  }

  // Update button visual tints based on current active state
  let states = button_states.0.lock().unwrap();
  for (marker, mut bg) in &mut button_query {
    let is_active = states.get(&marker.id).copied().unwrap_or(false);
    bg.0 = if is_active {
      marker.active_color
    } else {
      marker.default_color
    };
  }
}

/// Dynamically links live offscreen rendered display textures into the unified testing window.
pub fn sync_testing_window_displays(
  proxy_targets: Query<&ProxiedDisplayTarget>,
  internal_query: Query<Entity, (With<InternalDisplayPreviewMarker>, Without<ImageNode>)>,
  external_query: Query<Entity, (With<ExternalDisplayPreviewMarker>, Without<ImageNode>)>,
  mut commands: Commands,
) {
  for target in &proxy_targets {
    if target.id == "debug_internal_display" {
      for entity in &internal_query {
        commands.entity(entity).insert(ImageNode {
          image: target.image_handle.clone(),
          ..default()
        });
      }
    } else if target.id == "debug_external_display" {
      for entity in &external_query {
        commands.entity(entity).insert(ImageNode {
          image: target.image_handle.clone(),
          ..default()
        });
      }
    }
  }
}

/// Makes windows visible after rendering the first few frames to prevent initial flash.
pub fn make_visible(mut windows: Query<&mut Window>, frames: Res<FrameCount>) {
  if frames.0 == 3 {
    for mut window in &mut windows {
      window.visible = true;
    }
  }
}

/// Updates the scroll position of scrollable nodes in response to mouse input
pub fn update_scroll_position(
  mut mouse_wheel_events: EventReader<MouseWheel>,
  hover_map: Res<HoverMap>,
  mut scrolled_node_query: Query<&mut ScrollPosition>,
  keyboard_input: Res<ButtonInput<KeyCode>>,
) {
  for mouse_wheel_event in mouse_wheel_events.read() {
    let (mut dx, mut dy) = match mouse_wheel_event.unit {
      MouseScrollUnit::Line => (mouse_wheel_event.x * 20., mouse_wheel_event.y * 20.),
      MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
    };

    if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
      std::mem::swap(&mut dx, &mut dy);
    }

    for (_pointer, pointer_map) in hover_map.iter() {
      for (entity, _hit) in pointer_map.iter() {
        if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
          scroll_position.offset_x -= dx;
          scroll_position.offset_y -= dy;
        }
      }
    }
  }
}
