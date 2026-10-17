use std::sync::Arc;

use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BusTypes {
  App,
  #[cfg(feature = "can_fd")]
  CANFD,
  #[cfg(feature = "can_2")]
  CAN2,
  #[cfg(feature = "bt_classic")]
  BT,
  #[cfg(feature = "bt_le")]
  BTLE,
  #[cfg(feature = "spi")]
  SPI,
  #[cfg(feature = "i2c")]
  I2C,
  #[cfg(feature = "uart")]
  UART,
}

pub trait Bus {
  /// Send a message and expect a reply to that message.
  /// Listen to the Bus (does NOT contain IDs.)
  fn subscribe_to_bus(
    self,
    session: Arc<zenoh::Session>,
  ) -> impl std::future::Future<Output = Result<tokio::task::JoinHandle<()>, anyhow::Error>> + Send;
  fn get_type() -> BusTypes;
}
