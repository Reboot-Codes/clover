use super::models::CameraComponent;
use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::store::ModManStore,
};
use std::sync::Arc;

impl CloverComponentTrait for CameraComponent {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }

  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }
}
