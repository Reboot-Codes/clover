use super::models::{
  PhysicalDisplayComponent,
  VirtualDisplayComponent,
};
use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::store::ModManStore,
};
use std::sync::Arc;

impl CloverComponentTrait for PhysicalDisplayComponent {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    Ok(())
  }

  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    Ok(())
  }
}

impl CloverComponentTrait for VirtualDisplayComponent {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    Ok(())
  }

  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    Ok(())
  }
}
