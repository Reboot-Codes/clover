//! # Module Commands
//!
//! Module Commands get sent to components which primarily handle output.
//! Commands set a target for the component to reach in a specific ammount of time (usually the frame time that the hub is running at).
//!
//! ## Components
//!
//! - [Indicators](models::components::indicators) - Shows basic status and output.
//! - [Movement](models::components::modement) - Physically move things around.
//! - [Displays](models::components::display) - Show complex output from the Hub's renderer process.
//! - [Speakers](models::components::speaker) - Outputs either canned audio samples, or audio from the Hub.
//!

pub mod hub;
pub mod models;
pub mod module;
