use super::models::{
  AudioInputComponent,
  AudioOutputComponent,
};
use crate::server::modman::components::models::CloverComponentTrait;
use std::sync::Arc;

impl CloverComponentTrait for AudioInputComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}

impl CloverComponentTrait for AudioOutputComponent {
  async fn init(
    &mut self,
    store: Arc<crate::server::modman::models::ModManStore>,
  ) -> Result<(), anyhow::Error> {
    todo!()
  }
}
