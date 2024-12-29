pub mod canvas;
pub mod compose;
pub mod stream;

/// A view is an area of the system UI that needs to have its components rendered in a display's composition. Depending on the type (Compose, Canvas, or Stream), it may need extra processing.
pub trait View {}
