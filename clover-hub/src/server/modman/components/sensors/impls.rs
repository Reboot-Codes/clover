use super::models::SensorComponent;
use crate::server::modman::{
  components::models::{run_module_self_test, CloverComponentTrait},
  models::store::ModManStore,
};
use std::sync::Arc;

use tracing::{
  debug,
  info,
};

impl CloverComponentTrait for SensorComponent {
  async fn init(
    &mut self,
    _store: Arc<ModManStore>,
    module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Initializing SensorComponent {component_id} on {module_id}...");
    run_module_self_test(&session, module_id, component_id).await?;
    debug!("SensorComponent initialized successfully.");
    Ok(())
  }

  async fn deinit(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Deinitializing SensorComponent {component_id}...");
    debug!("SensorComponent deinitialized successfully.");
    Ok(())
  }
}
