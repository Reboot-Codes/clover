use bevy::prelude::*;
#[cfg(feature = "compositor")]
use bevy::window::Monitor;
use std::env;

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

pub fn setup(
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

    if &env::var("CLOVER_SIMULATED_CONTROLS").unwrap_or("false".to_string()) == "true" {
      // Camera
      commands.spawn((Camera2d::default(), IsDefaultUiCamera));

      // root node
      commands
        .spawn(Node {
          width: Val::Percent(100.0),
          height: Val::Percent(100.0),
          justify_content: JustifyContent::SpaceBetween,
          ..default()
        })
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
    }
  }
}
