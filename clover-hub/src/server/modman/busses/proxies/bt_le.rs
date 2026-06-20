use std::sync::Arc;

use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};
use bluer;

#[derive(Debug, Clone)]
pub struct BluetoothLEBus {}

impl Bus for BluetoothLEBus {
  async fn subscribe_to_bus(
    mut self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::BTLE
  }
}
