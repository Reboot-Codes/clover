use std::sync::Arc;

use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use spidev;

#[derive(Debug, Clone)]
pub struct SPIBus {}

impl Bus for SPIBus {
  async fn subscribe_to_bus(
    mut self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::SPI
  }
}
