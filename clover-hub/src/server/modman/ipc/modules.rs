//! # ModMan Module & Port IPC Queryables
//!
//! Exposes Zenoh queryables for querying registered hardware modules and physical port states.
//!
//! ## Zenoh Endpoints
//!
//! ### Queryables Provided
//! - `com/reboot-codes/clover/hub/modman/modules/all`: Queries all registered hardware modules.
//!   Implemented by [`modules_queryable`].
//! - `com/reboot-codes/clover/hub/modman/ports/all`: Queries live status of ModMan hardware ports (UART, CAN_2).
//!   Implemented by [`modules_queryable`].

use std::sync::Arc;
use serde::{
  Deserialize,
  Serialize,
};
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  instrument,
  warn,
};

use crate::server::modman::{
  models::store::ModManStore,
  MODULE_EVT_ID,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModuleSummary {
  pub id: String,
  pub module_type: String,
  pub module_name: String,
  pub custom_name: Option<String>,
  pub initialized: bool,
  pub registered_by: String,
  pub connection_type: String,
  pub components: Vec<(String, bool)>,
}

/// Zenoh queryable responder that serves registered modules and port statuses.
///
/// ### Zenoh Queryables
///
/// #### 1. Modules List
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/modules/all`
/// - **Query Payload**: Empty or query parameters
/// - **Reply Format**: Serialized JSON array of [`ModuleSummary`]
///
/// ##### Example Reply
/// ```json
/// [
///   {
///     "id": "module-can2-01",
///     "module_type": "tail",
///     "module_name": "Clover Tail Controller",
///     "custom_name": "Primary Tail",
///     "initialized": true,
///     "registered_by": "can_2",
///     "connection_type": "Can(Can2)",
///     "components": [
///       ["servo_yaw", true],
///       ["servo_pitch", true]
///     ]
///   }
/// ]
/// ```
///
/// #### 2. Port Statuses
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/ports/all`
/// - **Query Payload**: Empty
/// - **Reply Format**: Serialized JSON object containing `uart` and `can_2` statuses
///
/// ##### Example Reply
/// ```json
/// {
///   "uart": {
///     "is_connected": true,
///     "baud_rate": 115200
///   },
///   "can_2": {
///     "is_connected": true,
///     "bitrate": 1000000
///   }
/// }
/// ```
#[instrument(skip(store, session, cancellation_token))]
pub async fn modules_queryable(

  store: ModManStore,
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
) {
  let modules_key_expr = format!("{MODULE_EVT_ID}/modules/all");
  let ports_key_expr = format!("{MODULE_EVT_ID}/ports/all");

  let modules_queryable = match session.declare_queryable(&modules_key_expr).await {
    Ok(q) => q,
    Err(e) => {
      error!("Failed to declare modules queryable on {modules_key_expr}: {e}");
      return;
    }
  };

  let ports_queryable = match session.declare_queryable(&ports_key_expr).await {
    Ok(q) => q,
    Err(e) => {
      error!("Failed to declare ports queryable on {ports_key_expr}: {e}");
      return;
    }
  };

  debug!("Listening for module queries on {modules_key_expr} and {ports_key_expr}!");

  while !cancellation_token.is_cancelled() {
    tokio::select! {
      _ = cancellation_token.cancelled() => {
        debug!("Shutting down modules queryables...");
        break;
      }
      recv_result = modules_queryable.recv_async() => {
        match recv_result {
          Ok(query) => {
            let modules = store.modules.lock().await;
            let list: Vec<ModuleSummary> = modules
              .iter()
              .map(|(id, m)| ModuleSummary {
                id: id.clone(),
                module_type: m.module_type.clone(),
                module_name: m.module_name.clone(),
                custom_name: m.custom_name.clone(),
                initialized: m.initialized,
                registered_by: m.registered_by.clone(),
                connection_type: format!("{:?}", m.connection),
                components: m.components.clone(),
              })
              .collect();

            if let Ok(json) = serde_json::to_string(&list) {
              let _ = query.reply(&modules_key_expr, json).await;
            }
          }
          Err(err) => {
            warn!("Error receiving from modules queryable: {err}");
          }
        }
      }
      recv_result = ports_queryable.recv_async() => {
        match recv_result {
          Ok(query) => {
            let uart = store.port_statuses.uart.lock().await.clone();
            let can_2 = store.port_statuses.can_2.lock().await.clone();
            let payload = serde_json::json!({
              "uart": uart,
              "can_2": can_2,
            });

            if let Ok(json) = serde_json::to_string(&payload) {
              let _ = query.reply(&ports_key_expr, json).await;
            }
          }
          Err(err) => {
            warn!("Error receiving from ports queryable: {err}");
          }
        }
      }
    }
  }
}
