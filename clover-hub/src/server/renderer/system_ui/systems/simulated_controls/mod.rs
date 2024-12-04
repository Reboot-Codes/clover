use bevy::{
  prelude::*,
  core::FrameCount, 
  ecs::system::Query, 
  input::mouse::{
    MouseScrollUnit, 
    MouseWheel
  }, 
  picking::focus::HoverMap, 
  window::{
    PresentMode, 
    Window, 
    WindowPlugin, 
    WindowTheme
  }
};
use std::env;

pub fn make_visible(mut window: Query<&mut Window>, frames: Res<FrameCount>) {
  // info!("Frame: {}", frames.0);
  // The delay may be different for your app or system.
  if frames.0 == 3 {
    // At this point the gpu is ready to show the app so we can make the window visible.
    // Alternatively, you could toggle the visibility in Startup.
    // It will work, but it will have one white frame before it starts rendering
    window.get_single_mut().unwrap().visible = true;
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

      if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight)
      {
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
