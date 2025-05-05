use nexus::server::{
  models::IPCMessageWithId,
  websockets::WsIn,
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
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error>;
  fn get_type() -> BusTypes;
}
