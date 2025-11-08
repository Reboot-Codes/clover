#[cfg(feature = "compositor")]
use bevy::window::Monitor;
use bevy::{
  core::FrameCount,
  ecs::system::Query,
  input::mouse::{
    MouseScrollUnit,
    MouseWheel,
  },
  picking::focus::HoverMap,
  prelude::*,
  window::Window,
};
use bevy::{
  render::camera::RenderTarget,
  window::{
    PresentMode,
    PrimaryWindow,
    WindowRef,
    WindowTheme,
  },
};

pub fn sim_button(parent: &mut ChildBuilder<'_>, color: Color, text: &str) {
  let button_id = text.to_owned();

  parent
    .spawn((
      Node {
        width: Val::Px(50.0),
        height: Val::Px(50.0),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
      },
      BackgroundColor(color),
    ))
    .with_children(|button| {
      button.spawn((
        Text::new(text),
        TextFont {
          font_size: 10.0,
          ..Default::default()
        },
        TextColor(Color::srgb(0.0, 0.0, 0.0)),
      ));
    })
    .observe(move |_: Trigger<Pointer<Click>>| {
      debug!("{} was clicked!", button_id);
    });
}

#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct SimulatedControlsWindow {}

#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct SimulatedControlsCamera {}

#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct SimulatedControlsRootNode {}

pub fn sim_setup(
  mut commands: Commands,
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

    debug!("Spawning simulated controls window...");

    let ui_window = commands
      .spawn((
        Window {
          title: "Clover Simulated Controls".into(),
          resolution: (440.0, 75.0).into(),
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
          visible: true,
          ..Default::default()
        },
        SimulatedControlsWindow {},
      ))
      .id();

    // Camera
    let ui_camera = commands
      .spawn((
        Camera2d::default(),
        IsDefaultUiCamera,
        SimulatedControlsCamera {},
        Camera {
          target: RenderTarget::Window(WindowRef::Entity(ui_window)),
          ..default()
        },
      ))
      .id();

    // root node
    commands
      .spawn((
        Node {
          width: Val::Percent(100.0),
          height: Val::Percent(100.0),
          justify_content: JustifyContent::SpaceBetween,
          ..default()
        },
        TargetCamera(ui_camera),
        SimulatedControlsRootNode {},
      ))
      .insert(PickingBehavior {
        should_block_lower: false,
        is_hoverable: false,
      })
      .with_children(|parent| {
        // Directions
        sim_button(parent, Color::srgb(0.65, 0.65, 0.65), "Up");
        sim_button(parent, Color::srgb(0.65, 0.65, 0.65), "Down");
        sim_button(parent, Color::srgb(0.65, 0.65, 0.65), "Left");
        sim_button(parent, Color::srgb(0.65, 0.65, 0.65), "Right");

        // Primary
        sim_button(parent, Color::srgb(0.1, 0.65, 0.1), "1");

        // Seccondary
        sim_button(parent, Color::srgb(0.1, 0.1, 0.65), "2");

        // Back
        sim_button(parent, Color::srgb(0.65, 0.1, 0.1), "Back");

        // Home
        sim_button(parent, Color::srgb(0.65, 0.65, 0.1), "Home");
      });

    debug!("Spawned simulated controls window!");
  }
}

pub fn make_visible(mut windows: Query<&mut Window>, frames: Res<FrameCount>) {
  // info!("Frame: {}", frames.0);
  // The delay may be different for your app or system.
  if frames.0 == 3 {
    // At this point the gpu is ready to show the app so we can make the window visible.
    // Alternatively, you could toggle the visibility in Startup.
    // It will work, but it will have one white frame before it starts rendering
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
