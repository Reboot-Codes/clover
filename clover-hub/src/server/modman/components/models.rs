use crate::server::modman::models::store::ModManStore;
use serde::{
  Deserialize,
  Serialize,
};
use std::sync::Arc;

/// All components must implement this trait, ensures standardization between component types, etc.
pub trait CloverComponentTrait: Sized {
  /// Should initialize the component in the store, and ensure that 2-way communication is setup.
  fn init(
    &mut self,
    store: Arc<ModManStore>,
    module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
  /// Tells the component that it will not be used in the *near* future, and may even power it down.
  fn deinit(
    &mut self,
    store: Arc<ModManStore>,
    module_id: &str,
    component_id: &str,
    session: Arc<zenoh::Session>,
  ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}

/// Helper to dispatch self-test command to module hardware over Zenoh
pub async fn run_module_self_test(
  session: &zenoh::Session,
  module_id: &str,
  component_id: &str,
) -> Result<(), anyhow::Error> {
  use crate::server::modman::MODULE_EVT_ID;
  use carbon_fiber::comms::models::{BusMessage, RawContentMessage};
  use carbon_fiber::module::comms::CommsToModule;
  use std::time::Duration;
  use tracing::{debug, warn};

  let key_expr = format!("{MODULE_EVT_ID}/modules/by-id/{module_id}/send");
  let dispatch_bytes = postcard::to_allocvec(&CommsToModule::RunSelfTest)?;
  let raw = RawContentMessage {
    nonce: [0u8; 12],
    data: &dispatch_bytes,
    hmac: [0u8; 32],
  };
  let bus_msg = BusMessage::Content(raw);
  let payload_json = serde_json::to_string(&bus_msg)?;

  match session.get(&key_expr).payload(payload_json).await {
    Ok(replies) => {
      match tokio::time::timeout(Duration::from_millis(500), replies.recv_async()).await {
        Ok(Ok(reply)) => match reply.into_result() {
          Ok(sample) => {
            let reply_str = sample.payload().try_to_string().unwrap_or_default();
            debug!("Self-test query reply from {module_id}/{component_id}: {reply_str}");
            if reply_str.contains(r#""status":"error""#) {
              return Err(anyhow::anyhow!("Module self-test failed: {reply_str}"));
            }
            Ok(())
          }
          Err(e) => {
            warn!("Self-test reply error from module {module_id}: {e}");
            Ok(())
          }
        },
        Ok(Err(_)) => {
          debug!("No queryable responded to self-test for module {module_id}/{component_id}; proceeding.");
          Ok(())
        }
        Err(_) => {
          debug!("Self-test query timed out for module {module_id}/{component_id}; proceeding.");
          Ok(())
        }
      }
    }
    Err(e) => {
      debug!("Failed to query module {module_id} for self-test: {e}");
      Ok(())
    }
  }
}

/// Helper to publish an [`carbon_fiber::alerts::models::components::AlertMessage`] for an alert component over Zenoh.
pub async fn publish_component_alert(
  session: &zenoh::Session,
  component_id: &str,
  alert: &carbon_fiber::alerts::models::components::AlertMessage,
) -> Result<(), anyhow::Error> {
  use crate::server::modman::MODULE_EVT_ID;
  let key_expr = format!("{MODULE_EVT_ID}/components/by-id/{component_id}/alerts");
  let json = serde_json::to_string(alert)?;
  session
    .put(&key_expr, json)
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  Ok(())
}

/// Known and supported streaming protocols for Video and Audio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StreamProtocol {
  RTMP,
  RTSP,
  HTTP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamingConnection {
  /// IPv4 address to listen to connections from.
  pub v4_address: std::net::Ipv4Addr,
  /// Optional IPv6 address to listen to connections from.
  pub v6_address: Option<std::net::Ipv6Addr>,
  /// Port to bind to.
  pub port: u16,
  /// Either RTMP or RTSP
  pub protocol: StreamProtocol,
  pub path: Option<String>,
}
