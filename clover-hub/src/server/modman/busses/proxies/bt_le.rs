use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use bluer;
use nexus::server::{
  models::IPCMessageWithId,
  websockets::WsIn,
};

#[derive(Debug, Clone)]
pub struct BluetoothLEBus {}

impl Bus for BluetoothLEBus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::BTLE
  }
}
