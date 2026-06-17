pub mod displays;
pub mod gestures;

use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::server::modman::{
  ipc::{
    displays::display_queryable,
    gestures::gesture_queryable,
  },
  models::ModManStore,
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

  futures::future::join_all(vec![displays_handle, gestures_handle]).await;
}
