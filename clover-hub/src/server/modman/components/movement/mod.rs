//! Autonomous Physical Interaction
//!
//! Clover seperates movement into it's own category of module to ensure safety, security, and reliability. Motor components take in position or acceleration values based on the configuration, and can optionally return position data. Aesthetic features which have minimal impact on the user can be configured to work with the [Gesture](crate::server::modman::gestures) system, but anything more advanced should be done with an app through [Tesseract].
//!

pub mod impls;
pub mod models;
