use std::sync::Arc;

use crate::server::modman::{
  busses::models::{
    Bus,
    BusTypes,
  },
  models::ModManStore,
};
use log::debug;
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
    match serialport::available_ports() {
      Ok(port_info) => {
        let mut handles = vec![];

        for port in port_info {
          debug!("Found port: {:#?}!", port.clone());

          let mut attempt_bind = None;
          let config = self.store.config.lock().await;

          for allowed_port in config.modman.uart_ports.clone() {
            if allowed_port.0 == port.port_name.clone() {
              attempt_bind = Some(allowed_port);
              break;
            }
          }

          match attempt_bind {
            Some(port_info) => {
              debug!("Attempting to bind to: {}...", port.port_name.clone());
              match SerialStream::open(&tokio_serial::new(port_info.0, port_info.1)) {
                Ok(bound_port) => {
                  let (mut port_read, mut port_write) = split(bound_port);
                  let from_port = from_bus.clone();
                  let mut to_port_rx = to_bus.subscribe();
                  handles.push(tokio::task::spawn(async move {
                    let mut sub_handles = vec![];

                    sub_handles.push(tokio::task::spawn(async move {
                      while let Ok(msg) = to_port_rx.recv().await {
                        debug!("Sending message...");

                        // TODO: Encrypt
                        match rmp_serde::to_vec(&msg) {
                          Ok(msg_vec) => {
                            port_write.write(msg_vec.as_slice());
                          }
                          Err(_) => {
                            // TODO:
                          }
                        }
                      }
                    }));

                    sub_handles.push(tokio::task::spawn(async move {
                      let mut buffer = Vec::new();
                      while let Ok(msg_size) = port_read.read(buffer.as_mut_slice()).await {
                        debug!("Got {} bytes from UART!", msg_size);

                        // TODO: Decryption
                        match rmp_serde::from_slice(buffer.as_slice()) {
                          Ok(msg) => {
                            from_port.send(msg);
                          }
                          Err(_) => {
                            // TODO:
                          }
                        }

                        buffer = vec![];
                      }
                    }));

                    futures::future::join_all(sub_handles);
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
