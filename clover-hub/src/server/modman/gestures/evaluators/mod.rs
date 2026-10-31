//! # Gesture Component Evaluators
//!
//! Modular evaluators converting active gesture timelines and keyframe channels
//! into concrete hardware setpoints for specific Clover component families.
//!
//! ## Submodules
//! - [`movement`]: Multi-DOF movement actuator setpoint evaluation ([`evaluate_movement`]).
//! - [`indicator`]: RGB, dimmer, and binary LED setpoint evaluation ([`evaluate_indicator`]).
//! - [`speaker`]: Buzzer/speaker tone frequency and volume evaluation ([`evaluate_speaker`]).

pub mod indicator;
pub mod movement;
pub mod speaker;

pub use indicator::evaluate_indicator;
pub use movement::evaluate_movement;
pub use speaker::evaluate_speaker;
