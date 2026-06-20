use std::sync::Arc;

use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use i2c;
use i2cdev;

#[derive(Debug, Clone)]
pub struct I2CBus {}

impl Bus for I2CBus {
  async fn subscribe_to_bus(
    mut self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::I2C
  }
}
