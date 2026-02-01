use std::sync::Arc;

use crate::server::modman::{
  busses::models::{
    Bus,
    BusTypes,
  },
  models::{
    ModManStore,
    PortStatus,
  },
};
use log::{
  debug,
  warn,
};
use nexus::server::{
  models::IPCMessageWithId,
  websockets::WsIn,
};
use serialport;
use tokio::io::{
  split,
  AsyncReadExt,
  AsyncWriteExt,
};
use tokio_serial::{
  self,
  SerialStream,
};

#[derive(Debug, Clone)]
pub struct UARTBus {
  pub store: Arc<ModManStore>,
}

impl Bus for UARTBus {
  async fn subscribe_to_bus(
    &mut self,
    from_bus: tokio::sync::broadcast::Sender<WsIn>,
    to_bus: tokio::sync::broadcast::Sender<IPCMessageWithId>,
  ) -> Result<Vec<tokio::task::JoinHandle<()>>, anyhow::Error> {
    let warn_config = self.store.config.lock().await;
    if !(warn_config.modman.uart_ports.len() > 0) {
      warn!("No UART ports configured to proxy messages to! You may see message delivery errors from `nexus::user` due to a closed channel (the UARTBus proxy); they can be safely ignored.");
    }
    drop(warn_config);

    match serialport::available_ports() {
      Ok(port_info) => {
        let mut handles = vec![];

        for port in port_info {
          debug!("Found port: {:#?}!", port.clone());

          let mut attempt_bind = None;
          let config = self.store.config.lock().await;
          let mut port_statuses = self.store.port_statuses.uart.lock().await;

          for allowed_port in config.modman.uart_ports.clone() {
            if allowed_port.0 == port.port_name.clone() {
              let mut bound = false;
              let port_path = allowed_port.0.clone();

              for port_status in port_statuses.iter() {
                match port_status.1 {
                  PortStatus::Available => {}
                  PortStatus::Requested(component_id) => {
                    if port_status.0 == &port_path {
                      attempt_bind = Some((component_id.clone(), allowed_port));
                    }
                    break;
                  }
                  PortStatus::Bound(_) => {
                    bound = true;
                  }
                  PortStatus::Unavailable(component_id) => {
                    if port_status.0 == &port_path {
                      attempt_bind = Some((component_id.clone(), allowed_port));
                    }
                    break;
                  }
                }
              }

              match attempt_bind {
                Some(_) => {}
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

          match attempt_bind {
            Some(bind_info) => {
              // TODO: Create new Nexus user for this component!!

              let (component_id, port_info) = bind_info;
              debug!(
                "Component: {}, Attempting to bind to: {}...",
                component_id.clone(),
                port.port_name.clone()
              );
              match SerialStream::open(&tokio_serial::new(port_info.0.clone(), port_info.1)) {
                Ok(bound_port) => {
                  let (mut port_read, mut port_write) = split(bound_port);
                  let from_port = from_bus.clone();
                  let mut to_port_rx = to_bus.subscribe();
                  self
                    .store
                    .port_statuses
                    .uart
                    .lock()
                    .await
                    .insert(port_info.0.clone(), PortStatus::Bound(component_id.clone()));
                  debug!(
                    "Component: {}, Port: {}, bound!",
                    component_id.clone(),
                    port.port_name.clone()
                  );

                  handles.push(tokio::task::spawn(async move {
                    let mut sub_handles = vec![];

                    let tx_port_ctx = (component_id.clone(), port.port_name.clone());
                    sub_handles.push(tokio::task::spawn(async move {
                      while let Ok(msg) = to_port_rx.recv().await {
                        debug!(
                          "Component: {}, Port: {}, sending message from Nexus...",
                          tx_port_ctx.0.clone(),
                          tx_port_ctx.1.clone()
                        );

                        // TODO: Encrypt
                        match rmp_serde::to_vec(&msg) {
                          Ok(msg_vec) => {
                            port_write.write(msg_vec.as_slice()).await;
                          }
                          Err(_) => {
                            // TODO:
                          }
                        }
                      }
                    }));

                    let rx_port_ctx = (component_id.clone(), port.port_name.clone());
                    sub_handles.push(tokio::task::spawn(async move {
                      let mut buffer = Vec::new();
                      while let Ok(msg_size) = port_read.read(buffer.as_mut_slice()).await {
                        debug!(
                          "Component: {}, Port: {}, got message of length: {}, parsing...",
                          rx_port_ctx.0.clone(),
                          rx_port_ctx.1.clone(),
                          msg_size
                        );

                        // TODO: Decryption
                        match rmp_serde::from_slice::<WsIn>(buffer.as_slice()) {
                          Ok(msg) => {
                            debug!(
                              "Component: {}, Port: {}, parsed message: {:#?}",
                              rx_port_ctx.0.clone(),
                              rx_port_ctx.1.clone(),
                              msg.clone()
                            );
                            from_port.send(msg);
                          }
                          Err(_) => {
                            // TODO:
                          }
                        }

                        buffer = vec![];
                      }
                    }));

                    futures::future::join_all(sub_handles).await;
                  }));
                }
                Err(_) => todo!(),
              }
            }
            None => {}
          }

          std::mem::drop(config);
        }

        Ok(handles)
      }
      Err(e) => Err(e.into()),
    }
  }

  fn get_type() -> BusTypes {
    BusTypes::UART
  }
}
