use std::sync::Arc;

use crate::server::modman::{
  components::{
    indicators::models::IndicatorComponent,
    models::{run_module_self_test, CloverComponentTrait},
  },
  models::store::ModManStore,
};

use tracing::{
  debug,
  info,
};

impl CloverComponentTrait for IndicatorComponent {
  async fn init(
    &mut self,
    _store: Arc<ModManStore>,
    module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Initializing IndicatorComponent {component_id} on {module_id}...");
    run_module_self_test(&session, module_id, component_id).await?;
    debug!("IndicatorComponent initialized successfully.");
    Ok(())
  }

  async fn deinit(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Deinitializing IndicatorComponent {component_id}, clearing RGB/LED states...");
    debug!("IndicatorComponent deinitialized successfully.");
    Ok(())
  }
}
