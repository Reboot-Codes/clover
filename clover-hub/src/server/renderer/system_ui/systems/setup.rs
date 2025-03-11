use bevy::prelude::*;
#[cfg(feature = "compositor")]
use bevy::window::Monitor;
use std::env;

pub fn setup(
    mut commands: Commands,
    #[cfg(feature = "compositor")] monitor_entities_query: Query<(Entity, &Monitor)>,
) {
    #[cfg(feature = "compositor")]
    {
        debug!("Available Monitors:");
        for (monitor_eid, monitor) in &monitor_entities_query {
            debug!("Monitor {} with EntityID of {}.", monitor.name.clone().unwrap_or("unknown".into()), monitor_eid.to_bits());
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
                .insert(PickingBehavior::IGNORE)
                .with_children(|parent| {
                    parent.spawn((
                        Node {
                            width: Val::Px(200.),
                            border: UiRect::all(Val::Px(2.)),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.65, 0.65, 0.65)),
                    ));
                });
        }
    }
}
