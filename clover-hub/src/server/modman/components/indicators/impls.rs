use std::sync::Arc;

use crate::server::modman::{
  components::{
    indicators::models::IndicatorComponent,
    models::CloverComponentTrait,
  },
  models::store::ModManStore,
};

impl CloverComponentTrait for IndicatorComponent {
  async fn init(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }

  async fn deinit(&mut self, store: Arc<ModManStore>) -> Result<(), anyhow::Error> {
    todo!()
  }
}
