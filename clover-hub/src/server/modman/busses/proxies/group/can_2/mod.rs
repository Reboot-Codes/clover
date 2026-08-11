//! # CAN 2.0 Bus Proxy
//!
//!
//!

use std::sync::Arc;

use crate::server::modman::{
  busses::models::{
    Bus,
    BusTypes,
  },
  models::store::ModManStore,
};

use can_iso_tp;
use linux_socketcan_iso_tp;
use nix::net::if_::if_nameindex;

#[derive(Debug, Clone)]
pub struct CAN2Bus {
  pub store: Arc<ModManStore>,
}

#[derive(Debug, Clone)]
pub struct InterfaceToBind {
  pub module_id: String,
  pub path: String,
}

impl Bus for CAN2Bus {
  async fn subscribe_to_bus(
    mut self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    todo!()
  }

  fn get_type() -> BusTypes {
    BusTypes::CAN2
  }
}
