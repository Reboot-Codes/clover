use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use i2c;
use i2cdev;
use nexus::server::{
  models::IPCMessageWithId,
  websockets::WsIn,
};
use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone)]
pub struct I2CBus {}

impl Bus for I2CBus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::I2C
  }
}
