use super::models::CameraComponent;
use crate::server::modman::components::models::CloverComponentTrait;
use std::sync::Arc;

impl CloverComponentTrait for CameraComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}
