use std::sync::Arc;

use tracing::{
  debug,
  instrument,
};

use crate::server::modman::{
  connections::CAN2Connection,
  models::{
    modules::Module,
    store::ModManStore,
    PortStatus,
  },
};

#[instrument(skip(store, session))]
pub async fn setup_can_2_connection(
  store: &ModManStore,
  module: &Module,
  id: &String,
  connection: CAN2Connection,
  session: Arc<zenoh::Session>,
) -> Result<(), anyhow::Error> {
  // can0/0x201:0x101
  let requested_port = format!(
    "{}/{}:{}",
    connection.bus_id, connection.reply_id, connection.device_id
  );

  debug!("Requesting CAN 2 port: {requested_port}, for module: {id}...");

  let mut can_2_ports = store.port_statuses.can_2.lock().await;
  can_2_ports.insert(requested_port, PortStatus::Requested(id.clone()));
  drop(can_2_ports);

  Ok(())
}
