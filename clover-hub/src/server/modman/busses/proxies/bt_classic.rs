use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use bluer;

#[derive(Debug, Clone)]
pub struct BluetoothBus {}

impl Bus for BluetoothBus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::BT
  }
}
