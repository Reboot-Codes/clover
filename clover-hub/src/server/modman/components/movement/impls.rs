use super::models::MovementComponent;
use crate::server::modman::{
  components::models::{run_module_self_test, CloverComponentTrait},
  models::store::ModManStore,
};
use std::sync::Arc;

use tracing::{
  debug,
  info,
};

impl CloverComponentTrait for MovementComponent {
  async fn init(
    &mut self,
    _store: Arc<ModManStore>,
    module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!(
      "Initializing MovementComponent {component_id} on {module_id} with initial position: {:?}...",
      self.initial_position
    );
    run_module_self_test(&session, module_id, component_id).await?;
    debug!("MovementComponent initialized successfully.");
    Ok(())
  }

  async fn deinit(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Deinitializing MovementComponent {component_id}, parking to neutral...");
    // Clear active kinematic overrides and park to neutral
    debug!("MovementComponent deinitialized successfully.");
    Ok(())
  }
}
