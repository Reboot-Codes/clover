use super::models::{
  PhysicalDisplayComponent,
  VirtualDisplayComponent,
};
use crate::server::modman::components::models::CloverComponentTrait;
use std::sync::Arc;

impl CloverComponentTrait for PhysicalDisplayComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }

  async fn deinit(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}

impl CloverComponentTrait for VirtualDisplayComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }

  async fn deinit(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}
