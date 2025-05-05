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
pub struct CANFDBus {}

impl Bus for CANFDBus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::CANFD
  }
}
