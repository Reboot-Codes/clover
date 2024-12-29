pub mod compose;

/// A view is an area of the system UI that needs to have its components rendered in a display's composition. Depending on the type (Compose, Canvas, or Stream), it may need extra processing.
pub trait View {

}

/// A view that allows for direct rendering commands in its bounds. Can contain other views with minimal overhead and some boilerplate in the rendering command source via Tesseract's `CustomFrame`.
pub struct CanvasView {

}

/// A stream of frames to display in a display's composition. No rendering commands can be applied to it, however it does respect position, sizing, and blending rules. The overhead is very large for this view!
pub struct StreamView {

}
