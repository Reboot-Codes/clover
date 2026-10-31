//! # CarbonFiber
//!
//! Reference Module and Component library for Clover.
//!
//! ## For Modules
//!
//! Enable the `mcu` feature, and optionally `alloc` if you have an allocator.
//! CarbonFiber includes many convinience features, especially for the officially supported bus types and configurations.
//!
//! For supported configurations and MCUs, most module code will be very simple and rely on the builder architecture. For example, a blinkie can be written with:
//!
//! ```rust,ignore
//! #![no_std]
//!
//! #[embassy_executor::main]
//! async fn main(spawner: Spawner) {
//!   let p = embassy_rp::init(Default::default());
//!   let bus = CAN2::new(p.SPI0, p.PIN_16, p.PIN_17, p.PIN_18, p.PIN_19);
//!
//!   let mut led = IndicatorComponent::new(0)
//!     .neopixel_pin(p.PIN_3, NeopixelConfig { num_pixels: 1 })
//!     .interpolate_linear();
//!
//!   let (comms_worker, mut module_runner) = Module::init(bus)
//!     .add_alert_component(&led);
//!
//!   spawner.spawn(comms_task(comms_worker)).unwrap();
//!   module_runner.run().await;
//! }
//! ```
//!
//! Note that this assumes that you're using a proper build system!
//!
//! ### Component Types
//!
//! Modules expose two main component types, those being:
//!
//! - [Command Components](crate::commands) which take in Targets from the Hub and,
//! - [Alert Components](crate::alerts) which send states back to the hub.
//!
//! ## For Hubs
//!
//! CloverHUB *should* use the same exact types as found in CarbonFiber, and this can all be treated as a standardized, and versioned protocol.
//!
//!
//!
//!

#![cfg_attr(not(feature = "std"), no_std)]

pub mod adoption;
pub mod alerts;
pub mod commands;
pub mod hub;
pub mod models;
pub mod module;

pub mod comms {
  pub use crate::models::*;
  pub use crate::module::comms::*;
  pub mod models {
    pub use crate::models::*;
  }
}

const fn parse_u8(s: &str) -> u8 {
  let bytes = s.as_bytes();
  let mut val: u8 = 0;
  let mut i = 0;
  while i < bytes.len() {
    let digit = bytes[i] - b'0';
    val = val * 10 + digit;
    i += 1;
  }
  val
}

pub const PROTOCOL_VERSION_MAJOR: u8 = parse_u8(env!("CARGO_PKG_VERSION_MAJOR"));
pub const PROTOCOL_VERSION_MINOR: u8 = parse_u8(env!("CARGO_PKG_VERSION_MINOR"));
pub const PROTOCOL_VERSION_PATCH: u8 = parse_u8(env!("CARGO_PKG_VERSION_PATCH"));

pub const PROTOCOL_VERSION: (u8, u8, u8) = (
  PROTOCOL_VERSION_MAJOR,
  PROTOCOL_VERSION_MINOR,
  PROTOCOL_VERSION_PATCH,
);
