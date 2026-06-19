//! # ModMan Proxies
//!
//! A.k.a. `busses`, Proxies allow Modules to access Zenoh securely without needing a network bridge. [Each bus](proxies) is compiled into ModMan, and enabled via features.
//!

pub mod models;
pub mod proxies;

use super::models::ModManStore;
use log::info;
use std::sync::Arc;

pub async fn start_busses(
  store: Arc<ModManStore>,
  session: Arc<zenoh::Session>,
) -> futures::future::JoinAll<tokio::task::JoinHandle<()>> {
  info!("Starting ModMan Proxy Busses...");
  let mut handles = vec![];

  // TODO: ASAP: Move to creating a sub-user for each module!
  let proxy_session = session.clone();
  handles.push(tokio::task::spawn(async move {
    /*while let Ok(raw_msg) = rx.recv().await {
      debug!("Got proxy message from bus: {:?raw_msg}");
    }*/
  }));

  // TODO: Add config options for each bus!
  handles.push(tokio::task::spawn(async move {
    info!("Starting App Bus...");
  }));

  #[cfg(feature = "can_fd")]
  handles.push(tokio::task::spawn(async move {
    info!("Starting CANFD Bus...");
  }));

  #[cfg(feature = "can_2")]
  handles.push(tokio::task::spawn(async move {
    info!("Starting CAN 2 A/B Bus...");
  }));

  #[cfg(feature = "bt_classic")]
  handles.push(tokio::task::spawn(async move {
    info!("Starting Bluetooth Classic Bus...");
  }));

  #[cfg(feature = "bt_le")]
  handles.push(tokio::task::spawn(async move {
    info!("Starting Bluetooth LE Bus...");
  }));

  #[cfg(feature = "spi")]
  handles.push(tokio::task::spawn(async move {
    info!("Starting SPI Bus...");
  }));

  #[cfg(feature = "i2c")]
  handles.push(tokio::task::spawn(async move {
    info!("Starting I2C Bus...");
  }));

  #[cfg(feature = "uart")]
  {
    handles.push(tokio::task::spawn(async move {
      info!("Starting UART Bus...");
      /*match (UARTBus {
        store: store.clone(),
      })
      .subscribe_to_bus(tx.clone(), uart_channel.0)
      .await
      {
        Ok(handles) => {
          futures::future::join_all(handles).await;
        }
        Err(_) => todo!(),
      }*/
    }));
  }

  futures::future::join_all(handles)
}
