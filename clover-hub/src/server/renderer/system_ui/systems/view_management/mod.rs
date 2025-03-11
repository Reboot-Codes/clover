pub mod canvas;
pub mod compose;
pub mod stream;

use std::collections::HashMap;

use bevy::prelude::*;
use canvas::CanvasView;
use compose::ComposeView;
use stream::StreamView;

/// A view is an area of the system UI that needs to have its components rendered in a display's composition. Depending on the type (Compose, Canvas, or Stream), it may need extra processing.
pub trait ViewTrait {
    fn new<'a>(commands: &mut Commands<'a, 'a>) -> EntityCommands<'a>;
}

pub enum View {
    Canvas(CanvasView),
    Compose(ComposeView),
    Stream(StreamView)
}

#[derive(Component)]
pub struct ViewPosition {
    x: f64,
    y: f64,
    z: f64
}

#[derive(Component)]
pub struct ViewDimensions {
    x: i64,
    y: i64,
    /// A 0 here will be determined as a 2D view.
    z: i64
}

#[derive(Component)]
pub struct ViewMixingProps {
    visible: bool
}

/// Each (virtual) display must have a composition attached to it to have frames ready to send.
#[derive(Component)]
pub struct Composition {
    /// Holds all Entity Ids for the Views within this Compositon.
    views: HashMap<String, Entity>,
    /// Should this composition be trusted with user-focused content? This has nothing to do with being exclusive to CloverHub usage.
    internal: bool,
}

/// Themes are set for each composition. A ThemeSet can be loaded for each type of composition.
#[derive(Component)]
pub struct Theme {}

impl Composition {
    pub fn new() -> Self {
        todo!();
    }
}
