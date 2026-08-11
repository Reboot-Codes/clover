//! # UART Proxy Bus
//!
//! The UART proxy bus is designed bind one serial port per module, then expose I/O over Zenoh.
//!

pub mod reader;
pub mod rx;
pub mod tx;

use std::sync::Arc;

use crate::server::modman::{
  busses::{
    models::{
      Bus,
      BusMessage,
      BusTypes,
    },
    proxies::uart::{
      reader::uart_reader,
      rx::uart_rx_thread,
      tx::uart_tx_thread,
    },
  },
  connections::ModuleConnection,
  models::{
    ModManStore,
    PortStatus,
  },
};
use anyhow::anyhow;
use serialport::{
  self,
  SerialPortInfo,
};
use tokio::{
  io::split,
  task::JoinHandle,
};
use tokio_serial::{
  self,
  SerialStream,
};
use tokio_util::io::SyncIoBridge;
use tracing::{
  debug,
  error,
  instrument,
};
#[derive(Debug, Clone)]
pub struct UARTBus {
  pub store: Arc<ModManStore>,
}

#[derive(Debug, Clone)]
pub struct PortToBind {
  module_id: String,
  path: String,
}

impl Bus for UARTBus {
  #[instrument(name = "uart_bus", skip(self, session))]
  async fn subscribe_to_bus(
    self,
    session: Arc<zenoh::Session>,
  ) -> Result<tokio::task::JoinHandle<()>, anyhow::Error> {
    match serialport::available_ports() {
      Ok(port_info) => {
        return Ok(tokio::task::spawn(async move {
          loop {
            let mut handles = vec![];

            for port in port_info.clone() {
              let mut attempt_bind: Option<PortToBind> = None;
              let config = self.store.config.lock().await;
              let mut port_statuses = self.store.port_statuses.uart.lock().await;
              let allowed_ports = config.modman.uart_ports.clone();
              drop(config);

              for allowed_port in allowed_ports {
                if allowed_port == port.port_name.clone() {
                  let mut bound = false;
                  let port_path = allowed_port.clone();

                  for port_status in port_statuses.iter() {
                    match port_status.1 {
                      PortStatus::Available => {}
                      PortStatus::Requested(module_id) => {
                        debug!("Port: {allowed_port}, requested for Module ID: {module_id}");

                        if port_status.0 == &port_path {
                          attempt_bind = Some(PortToBind {
                            module_id: module_id.clone(),
                            path: allowed_port,
                          });
                        }

                        break;
                      }
                      PortStatus::Bound(_port_status) => {
                        bound = true;
                      }
                      PortStatus::Unavailable(module_id) => {
                        if port_status.0 == &port_path {
                          attempt_bind = Some(PortToBind {
                            module_id: module_id.clone(),
                            path: allowed_port,
                          });
                        }
                        break;
                      }
                    }
                  }

                  match attempt_bind.clone() {
                    Some(_bind_info) => {}
                    Option::None => {
                      if !bound {
                        port_statuses.insert(port_path.clone(), PortStatus::Available);
                      }
                    }
                  }

                  break;
                }
              }

              std::mem::drop(port_statuses);

              match attempt_bind.clone() {
                Some(bind_info) => {
                  match bind_uart_port(bind_info, port, self.store.clone(), session.clone()).await {
                    Ok(mut sub_handles) => {
                      handles.append(&mut sub_handles);
                    }
                    Err(err) => {
                      error!("Failed to bind port due to:\n{err}");
                    }
                  }
                }
                None => {}
              }
            }

            futures::future::join_all(handles).await;
          }
        }));
      }
      Err(err) => Err(anyhow!("Failed to get available ports due to:\n{err}")),
    }
  }

  fn get_type() -> BusTypes {
    BusTypes::UART
  }
}

/// Reusable code to bind a serial port for a function, and then run Zenoh endpoints for that binding.
/// Should be called on startup for static modules, and then dynamically for app modules.
#[instrument(skip(store, session))]
pub async fn bind_uart_port(
  bind_info: PortToBind,
  port: SerialPortInfo,
  store: Arc<ModManStore>,
  session: Arc<zenoh::Session>,
) -> Result<Vec<JoinHandle<()>>, anyhow::Error> {
  debug!(
    "Module: {}, Attempting to bind to: {}...",
    bind_info.module_id.clone(),
    port.port_name.clone()
  );

  let modules_mutex = store.modules.lock().await;
  let module_config;
  match modules_mutex.get(&bind_info.module_id) {
    Some(remote_module_config) => {
      module_config = remote_module_config.clone();
    }
    None => {
      error!("Unable to find Module ID: {} in the modules store. Won't bind UART port: {} for a non-existant module!", bind_info.module_id, bind_info.path);
      return Err(anyhow!("Module not in store."));
    }
  }
  drop(modules_mutex);

  match module_config.connection {
    ModuleConnection::UART(connection_config) => {
      match SerialStream::open(&tokio_serial::new(
        bind_info.path.clone(),
        connection_config.baud,
      )) {
        Ok(bound_port) => {
          let port_session = session.clone();
          let (port_read, port_write) = split(bound_port);

          store.port_statuses.uart.lock().await.insert(
            bind_info.path.clone(),
            PortStatus::Bound(bind_info.module_id.clone()),
          );

          debug!(
            "Module: {}, Port: {}, bound!",
            bind_info.module_id, port.port_name
          );

          let mut sub_handles = vec![];

          let tx_port_ctx = (bind_info.module_id.clone(), port.port_name.clone());
          let tx_bind_info = bind_info.clone();
          let tx_session = port_session.clone();
          sub_handles.push(tokio::task::spawn(async move {
            uart_tx_thread(tx_bind_info, tx_session, tx_port_ctx, port_write).await;
          }));

          let (read_channel, rx_channel) = tokio::sync::mpsc::unbounded_channel::<BusMessage>();

          let rx_port_ctx = (bind_info.module_id.clone(), port.port_name.clone());
          let rx_session = port_session.clone();
          sub_handles.push(tokio::task::spawn(async move {
            uart_rx_thread(rx_port_ctx, rx_channel, rx_session).await;
          }));

          let bridge = SyncIoBridge::new(port_read);
          sub_handles.push(tokio::task::spawn_blocking(move || {
            uart_reader(bridge, read_channel);
          }));

          Ok(sub_handles)
        }
        Err(err) => {
          return Err(err.into());
        }
      }
    }
    _ => {
      error!("UART port: {}, was requested for Module ID: {}, but the module connection configuration is not for UART. This is unnaceptable and a bug, please report!", bind_info.path, bind_info.module_id);
      Err(anyhow!("Module Connection is not UART."))
    }
  }
}
