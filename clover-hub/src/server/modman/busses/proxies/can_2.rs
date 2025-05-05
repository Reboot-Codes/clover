use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use can;
use nexus::server::{
  models::IPCMessageWithId,
  websockets::WsIn,
};
use socketcan;

#[derive(Debug, Clone)]
pub struct CAN2Bus {}

impl Bus for CAN2Bus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::CAN2
  }
}
