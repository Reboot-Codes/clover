use std::{
  sync::Arc,
  time::Duration,
};

use anyhow::anyhow;
use tokio::time::{
  sleep,
  Instant,
};
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

#[instrument(skip(store, _session))]
pub async fn setup_can_2_connection(
  store: &ModManStore,
  module: &Module,
  id: &String,
  connection: CAN2Connection,
  _session: Arc<zenoh::Session>,
) -> Result<(), anyhow::Error> {
  // can0/0x201:0x101
  let requested_port = format!(
    "{}/{}:{}",
    connection.bus_id, connection.reply_id, connection.device_id
  );

  debug!("Requesting CAN 2 port: {requested_port}, for module: {id}...");

  {
    let mut can_2_ports = store.port_statuses.can_2.lock().await;
    can_2_ports.insert(requested_port.clone(), PortStatus::Requested(id.clone()));
  }

  // Await port binding confirmation or failure from bus_manager
  let start = Instant::now();
  let timeout = Duration::from_secs(3);

  loop {
    sleep(Duration::from_millis(50)).await;

    let can_2_ports = store.port_statuses.can_2.lock().await;
    match can_2_ports.get(&requested_port) {
      Some(PortStatus::Bound(bound_id)) if bound_id == id => {
        debug!("CAN 2 port {requested_port} successfully bound and adopted for module: {id}!");
        return Ok(());
      }
      Some(PortStatus::Unavailable(failed_id)) if failed_id == id => {
        return Err(anyhow!(
          "CAN 2 port {requested_port} became unavailable for module: {id} (adoption or binding failed)"
        ));
      }
      _ => {}
    }
    drop(can_2_ports);

    if start.elapsed() > timeout {
      return Err(anyhow!(
        "Timed out waiting for CAN 2 port {requested_port} to bind for module: {id}"
      ));
    }
  }
}

#[instrument(skip(store))]
pub async fn teardown_can_2_connection(
  store: &ModManStore,
  id: &str,
  connection: CAN2Connection,
) -> Result<(), anyhow::Error> {
  let requested_port = format!(
    "{}/{}:{}",
    connection.bus_id, connection.reply_id, connection.device_id
  );

  debug!("Requesting teardown for CAN 2 port: {requested_port}, for module: {id}...");

  let mut can_2_ports = store.port_statuses.can_2.lock().await;
  can_2_ports.insert(requested_port, PortStatus::Unrequested(id.to_string()));

  Ok(())
}

