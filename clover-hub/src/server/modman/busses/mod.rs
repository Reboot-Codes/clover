pub mod models;
pub mod proxies;

use super::models::ModManStore;
use log::info;
use models::Bus;
use nexus::server::websockets::WsIn;
use nexus::{
  server::MAX_SIZE,
  user::NexusUser,
};
use proxies::uart::UARTBus;
use std::sync::Arc;

pub async fn start_busses(
  store: Arc<ModManStore>,
  user: Arc<NexusUser>,
) -> futures::future::JoinAll<tokio::task::JoinHandle<()>> {
  info!("Starting ModMan Proxy Busses...");
  let mut handles = vec![];
  let (tx, mut rx) = tokio::sync::broadcast::channel::<WsIn>(MAX_SIZE);

  let proxy_user = user.clone();
  handles.push(tokio::task::spawn(async move {
    while let Ok(raw_msg) = rx.recv().await {
      proxy_user.send(&raw_msg.kind, &raw_msg.message, &raw_msg.replying_to);
    }
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
    let uart_channel = user.subscribe();
    handles.push(uart_channel.1);
    handles.push(tokio::task::spawn(async move {
      info!("Starting UART Bus...");
      match (UARTBus {
        store: store.clone(),
      })
      .subscribe_to_bus(tx.clone(), uart_channel.0)
      .await
      {
        Ok(handles) => {
          futures::future::join_all(handles).await;
        }
        Err(_) => todo!(),
      }
    }));
  }

  futures::future::join_all(handles)
}
