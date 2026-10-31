use super::models::CameraComponent;
use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::store::ModManStore,
};
use std::sync::Arc;

use super::models::ConnectionType;
use anyhow::anyhow;
use tracing::{
  debug,
  info,
};

impl CloverComponentTrait for CameraComponent {
  async fn init(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    _component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    match &self.connection {
      ConnectionType::Video4Linux(path) => {
        info!("Initializing V4L CameraComponent on: {path}...");
        if !std::path::Path::new(path).exists() {
          return Err(anyhow!("V4L Camera device does not exist: {path}"));
        }
        debug!("V4L CameraComponent verified on: {path}");
        Ok(())
      }
      ConnectionType::ModManProxy => todo!("ModmanProxy for CameraComponent scheduled for later sprint"),
      ConnectionType::Stream(_) => todo!(),
    }
  }

  async fn deinit(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    _component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    match &self.connection {
      ConnectionType::Video4Linux(path) => {
        info!("Deinitializing V4L CameraComponent on: {path}...");
        Ok(())
      }
      ConnectionType::ModManProxy => todo!("ModmanProxy for CameraComponent scheduled for later sprint"),
      ConnectionType::Stream(_) => todo!(),
    }
  }
}
