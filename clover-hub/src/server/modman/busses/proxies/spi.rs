use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use nexus::server::{
  models::IPCMessageWithId,
  websockets::WsIn,
};
use spidev;

#[derive(Debug, Clone)]
pub struct SPIBus {}

impl Bus for SPIBus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::SPI
  }
}
