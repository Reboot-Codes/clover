//! # ModMan IPC Subsystems
//!
//! Coordinates all external Zenoh queryables and publishers exposed by ModMan.
//!
//! ## Zenoh Endpoints Summary
//!
//! ### Displays ([`displays`])
//! - `com/reboot-codes/clover/hub/modman/components/by-type/video/displays/all`: Registered display configs.
//!
//! ### Gestures ([`gestures`])
//! - `com/reboot-codes/clover/hub/modman/gestures/all`: Gesture catalog.
//! - `com/reboot-codes/clover/hub/modman/gestures/active`: Active priority stacks and states.
//! - `com/reboot-codes/clover/hub/modman/gestures/update`: Command gesture playback.
//! - `com/reboot-codes/clover/hub/modman/gestures/status`: Gesture event stream.
//!
//! ### Modules & Hardware Ports ([`modules`])
//! - `com/reboot-codes/clover/hub/modman/modules/all`: Registered modules and hardware components.
//! - `com/reboot-codes/clover/hub/modman/ports/all`: Live status of UART and CAN_2 interfaces.

pub mod displays;
pub mod gestures;
pub mod modules;


use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::server::modman::{
  ipc::{
    displays::display_queryable,
    gestures::gesture_queryable,
    modules::modules_queryable,
  },
  models::store::ModManStore,
};

use std::sync::Arc;

#[instrument(skip(ipc_token, ipc_session))]
pub async fn handle_ipc(
  store: ModManStore,
  ipc_token: CancellationToken,
  ipc_session: Arc<zenoh::Session>,
) {
  let display_store = store.clone();
  let display_session = ipc_session.clone();
  let display_token = ipc_token.clone();
  let displays_handle = tokio::task::spawn(async move {
    display_queryable(display_store, display_session, display_token).await;
  });

  let gesture_store = store.clone();
  let gesture_session = ipc_session.clone();
  let gesture_token = ipc_token.clone();
  let gestures_handle = tokio::task::spawn(async move {
    gesture_queryable(gesture_store, gesture_token, gesture_session).await;
  });

  let modules_store = store.clone();
  let modules_session = ipc_session.clone();
  let modules_token = ipc_token.clone();
  let modules_handle = tokio::task::spawn(async move {
    modules_queryable(modules_store, modules_session, modules_token).await;
  });

  futures::future::join_all(vec![displays_handle, gestures_handle, modules_handle]).await;
}
