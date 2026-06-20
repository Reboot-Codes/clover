use std::sync::Arc;

use crate::server::modman::busses::models::{
  Bus,
  BusTypes,
};

pub struct AppBus {}

impl Bus for AppBus {
  async fn subscribe_to_bus(
    mut self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::App
  }
}
