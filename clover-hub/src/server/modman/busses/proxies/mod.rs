//! # Supported ModMan Proxy Busses
//!
//! All busses here are officially supported, and are designed to work with the CarbonFiber and CarbonSteel reference APIs to provide standardized communication according to the [Module's configuration](crate::server::modman::modules)
//!
//! # Group Proxies
//!
//! Group proxies handle proxying Zenoh traffic to a wire-level *bus* protocol. Think CAN, SPI, I2C, etc. These handle multiple modules on one interface, and use configuration paradigms that are useful when configuring interfaces.
//!
//! # Individual Proxies
//!
//! Individual proxies handle wire-level protocols which rely upon individual ports being connected to each module. Think bluetooth and UART.
//!

pub mod group;
pub mod individual;
