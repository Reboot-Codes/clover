//! # Group Proxies
//!
//! Group proxies handle proxying Zenoh traffic to a wire-level *bus* protocol. Think CAN, SPI, I2C, etc. These handle multiple modules on one interface, and use configuration paradigms that are useful when configuring interfaces.
//!
//! They maintain 3 types of threads:
//!
//! ## Bus Lookout Threads
//!
//! Lookouts run custom code that is used to check for the appearance of the specified interface for that bus. For example, the CAN 2 lookout thread will check all network interfaces for configured interfaces to check if they should be bound using the SocketCAN paradigm using a Bus Manager thread.
//!
//! ## Bus Manager Threads
//!
//! Bus manager threads handle the creation and deletion of module threads. These handle the initial handshake between a module and ModMan, and then creat a module manager to handle Zenoh proxying.
//!
//! ## Module Manager Threads
//!
//! Module manager threads handle the actual zenoh endpoints for modules (like how drivers expose device paths in linux).
//!

use serde::{
  Deserialize,
  Serialize,
};

#[cfg(feature = "can_2")]
use crate::server::modman::busses::proxies::group::can_2::CAN2Config;

#[cfg(feature = "can_2")]
pub mod can_2;
#[cfg(feature = "can_fd")]
pub mod can_fd;
#[cfg(feature = "i2c")]
pub mod i2c;
#[cfg(feature = "spi")]
pub mod spi;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct GroupBusConfigs {
  #[cfg(feature = "can_2")]
  pub can_2: CAN2Config,
}
