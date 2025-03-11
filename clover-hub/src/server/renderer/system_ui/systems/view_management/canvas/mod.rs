use bevy::prelude::*;

/// A view that allows for direct rendering commands in its bounds. Can contain other views with minimal overhead and some boilerplate in the rendering command source via Tesseract's `CustomFrame`.
#[derive(Component)]
pub struct CanvasView {}
