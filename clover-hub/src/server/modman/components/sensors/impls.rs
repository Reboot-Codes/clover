use super::models::{
  IndicatorComponent,
  SensorComponent,
};
use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::store::ModManStore,
};
use std::sync::Arc;

impl CloverComponentTrait for SensorComponent {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }

  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }
}

impl CloverComponentTrait for IndicatorComponent {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }

  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }
}
