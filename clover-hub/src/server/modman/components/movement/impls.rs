use super::models::MovementComponent;
use crate::server::modman::components::models::CloverComponentTrait;
use std::sync::Arc;

impl CloverComponentTrait for MovementComponent {
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
