pub mod components;

use bevy::prelude::*;

/// Contains Composable Components that automatically recieve themeing info, responsive sizing, etc. Can also contain other views within itself with no greater overhead.
#[derive(Component)]
pub struct ComposeView {}
