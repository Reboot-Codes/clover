//! # Audio I/O Components
//!
//! Audio components can be either a Microphone or a Speaker, which can be configured to play along with a stream using a Media Player activity ([CarbonSteel]/[Tesseract]), to work with the [Gesture system](crate::server::modman::gestures) in the case of a speaker, and/or to work with an app (when part of an [App Module](crate::server::modman::modules#configuration)) to run ML inference on what's heard in the case of a microphone.
//!

pub mod impls;
pub mod models;
