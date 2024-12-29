use bevy::prelude::*;
use std::env;

pub fn setup(mut commands: Commands) {
    #[cfg(feature = "compositor")]
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
