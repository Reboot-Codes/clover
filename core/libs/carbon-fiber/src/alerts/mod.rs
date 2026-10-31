//! # Alerts
//!
//! An alert is used by a module to send feedback to the hub.
//!
//! Alerts are primarily used to send button presses, but can be used for Audio, Video, and Environmental Sensors as well.
//!
//! ## Components
//!
//! - [Cameras](models::components::camera) - Sends changes in video, or a full video stream to the Hub.
//! - [Microphones](models::components::microphone) - Sends changes in audio level/frequency, or a full audio stream to the Hub.
//! - [Generic Sensors](models::components::sensor) - Sends generic numerical data back to the Hub.
//!

pub mod hub;
pub mod models;
pub mod module;
