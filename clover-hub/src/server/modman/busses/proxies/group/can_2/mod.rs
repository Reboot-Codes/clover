//! # CAN 2.0 Bus Proxy
//!
//! The primary proxy bus for Clover.
//!

pub mod bus_manager;
pub mod lookout;

use std::{
  sync::Arc,
  time::Duration,
};

use crate::server::modman::{
  busses::proxies::group::can_2::lookout::can_lookout_thread,
  models::store::ModManStore,
};

use anyhow::anyhow;
use nix::net::if_::if_nameindex;
use serde::{
  Deserialize,
  Serialize,
};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

#[derive(Debug, Clone)]
pub struct CAN2Bus {
  pub session: Arc<zenoh::Session>,
  pub store: Arc<ModManStore>,
  pub cancellation_token: CancellationToken,
}

#[derive(Debug, Clone)]
pub struct InterfaceToBind {
  pub module_id: String,
  pub path: String,
}

#[instrument(skip(ctx))]
pub async fn spawn_lookout_thread(
  ctx: Arc<CAN2Bus>,
) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
  let mut error = None;
  let mut retries = 0;

  while retries != 5 {
    match if_nameindex() {
      Ok(_) => return Ok(tokio::task::spawn(async { can_lookout_thread(ctx).await })),
      Err(err) => {
        error = Some(err.into());
        retries += 1;
        sleep(Duration::from_millis(500)).await;
      }
    }
  }

  match error {
    Some(err) => Err(err),
    None => Err(anyhow!("")),
  }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CAN2Config {
  /// Interfaces that we should bind to.
  pub permitted_interfaces: Vec<String>,
}
