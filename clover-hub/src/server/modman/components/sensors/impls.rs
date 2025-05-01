use super::models::{
  InputSensorComponent,
  OutputSensorComponent,
};
use crate::server::modman::components::models::CloverComponentTrait;
use std::sync::Arc;

impl CloverComponentTrait for InputSensorComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}

impl CloverComponentTrait for OutputSensorComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}
