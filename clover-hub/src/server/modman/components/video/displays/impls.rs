use super::models::{
  PhysicalDisplayComponent,
  VirtualDisplayComponent,
};
use crate::server::modman::{
  components::models::CloverComponentTrait,
  models::store::ModManStore,
};
use crate::server::renderer::MODULE_EVT_ID as RENDERER_EVT_ID;
use std::sync::Arc;
use std::time::Duration;
use tracing::{
  debug,
  info,
};
use zenoh_ext::{
  AdvancedSubscriberBuilderExt,
  HistoryConfig,
  RecoveryConfig,
};

async fn wait_for_renderer(session: &zenoh::Session, component_id: &str) -> Result<(), anyhow::Error> {
  let status_topic = format!("{RENDERER_EVT_ID}/status");
  debug!("Display component {component_id} waiting for renderer on {status_topic}...");

  // First check if renderer status is already published/cached
  if let Ok(replies) = session.get(&status_topic).await {
    if let Ok(Ok(reply)) = tokio::time::timeout(Duration::from_millis(100), replies.recv_async()).await {
      if let Ok(sample) = reply.into_result() {
        if let Ok(status) = sample.payload().try_to_string() {
          if status.starts_with("ready") {
            debug!("Renderer already ready ({status}) for display component {component_id}");
            return Ok(());
          }
        }
      }
    }
  }

  // Subscribe and wait with timeout
  let subscriber = session
    .declare_subscriber(&status_topic)
    .history(HistoryConfig::default().detect_late_publishers())
    .recovery(RecoveryConfig::default().heartbeat())
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;

  let wait_result = tokio::time::timeout(Duration::from_secs(5), async {
    while let Ok(sample) = subscriber.recv_async().await {
      if let Ok(status) = sample.payload().try_to_string() {
        if status.starts_with("ready") {
          return Ok(status.to_string());
        }
      }
    }
    Err(anyhow::anyhow!("Subscriber channel closed before renderer became ready"))
  })
  .await;

  match wait_result {
    Ok(Ok(status)) => {
      info!("Renderer ready ({status}) for display component {component_id}");
      Ok(())
    }
    Ok(Err(e)) => Err(e),
    Err(_) => Err(anyhow::anyhow!(
      "Timed out after 5s waiting for renderer to initialize for display component {component_id}"
    )),
  }
}

impl CloverComponentTrait for PhysicalDisplayComponent {
  async fn init(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Initializing PhysicalDisplayComponent {component_id}...");
    wait_for_renderer(&session, component_id).await?;
    debug!("PhysicalDisplayComponent {component_id} initialized successfully.");
    Ok(())
  }

  async fn deinit(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Deinitializing PhysicalDisplayComponent {component_id}...");
    Ok(())
  }
}

impl CloverComponentTrait for VirtualDisplayComponent {
  async fn init(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Initializing VirtualDisplayComponent {component_id}...");
    wait_for_renderer(&session, component_id).await?;
    debug!("VirtualDisplayComponent {component_id} initialized successfully.");
    Ok(())
  }

  async fn deinit(
    &mut self,
    _store: Arc<ModManStore>,
    _module_id: &str,
    component_id: &str,
    _session: Arc<zenoh::Session>,
  ) -> Result<(), anyhow::Error> {
    info!("Deinitializing VirtualDisplayComponent {component_id}...");
    Ok(())
  }
}
