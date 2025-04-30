use nexus::server::models::{
  IPCMessage,
  IPCMessageWithId,
};
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
  BTClassic,
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
  /// Send a message to the Bus.
  fn send_message(msg: IPCMessageWithId) -> Result<(), anyhow::Error>;
  /// Send a message and expect a reply to that message.
  /// Listen to the Bus (does NOT contain IDs.)
  fn subscribe_to_bus() -> tokio::sync::broadcast::Receiver<IPCMessage>;
}
