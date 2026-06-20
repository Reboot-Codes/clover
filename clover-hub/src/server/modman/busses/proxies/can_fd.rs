use std::sync::Arc;

use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use can;
use socketcan;

#[derive(Debug, Clone)]
pub struct CANFDBus {}

impl Bus for CANFDBus {
  async fn subscribe_to_bus(
    mut self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::CANFD
  }
}
