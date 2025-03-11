use bevy::prelude::*;

/// A stream of frames to display in a display's composition. No rendering commands can be applied to it, however it does respect position, sizing, and blending rules. The overhead is very large for this view!
#[derive(Component)]
pub struct StreamView {}
