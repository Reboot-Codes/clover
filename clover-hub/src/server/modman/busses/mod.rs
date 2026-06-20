//! # ModMan Proxies
//!
//! A.k.a. `busses`, Proxies allow Modules to access Zenoh securely without needing a network bridge. [Each bus](proxies) is compiled into ModMan, and enabled via features.
//!

pub mod models;
pub mod proxies;

use super::models::ModManStore;
use log::info;
use models::Bus;
use std::sync::Arc;

pub async fn start_busses(
  store: Arc<ModManStore>,
  session: Arc<zenoh::Session>,
) -> futures::future::JoinAll<tokio::task::JoinHandle<()>> {
  info!("Starting ModMan Proxy Busses...");
  let mut handles = vec![];

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
    use proxies::uart::UARTBus;

    let uart_session = session.clone();
    info!("Starting UART Bus...");
    match (UARTBus {
      store: store.clone(),
    })
    .subscribe_to_bus(uart_session)
    .await
    {
      Ok(handle) => {
        handles.push(handle);
      }
      Err(_) => todo!(),
    }
  }

  futures::future::join_all(handles)
}
