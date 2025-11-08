use anyhow::anyhow;
use log::debug;
use nexus::server::models::IPCMessageWithId;
use queues::Queue;
use std::sync::Mutex as StdMutex;
use std::{
  str::FromStr,
  sync::Arc,
};
use tokio::sync::broadcast::Sender;
use url::Url;

use crate::server::renderer::system_ui::AnyDisplayComponent;

#[derive(Debug, PartialEq)]
pub enum Events {
  None,
}

impl FromStr for Events {
  type Err = anyhow::Error;

  fn from_str(input: &str) -> Result<Events, Self::Err> {
    match input {
      "" => Ok(Events::None),
      "/" => Ok(Events::None),
      _ => Err(anyhow!("String \"{}\" not part of enum!", input)),
    }
  }
}

pub async fn handle_ipc_msg(
  ipc_rx: Sender<IPCMessageWithId>,
  display_registration_queue: Arc<StdMutex<Queue<(String, AnyDisplayComponent)>>>,
) {
  while let Ok(msg) = ipc_rx.subscribe().recv().await {
    let kind = Url::parse(&msg.kind.clone()).unwrap();

    // Verify that we care about this event.
    if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.renderer") {
      debug!("Processing: {}", msg.kind.clone());

      match Events::from_str(kind.path()) {
        Ok(event_type) => {
          match event_type {
            _ => {
              // TODO
              debug!("TODO!");
            }
          }
        }
        Err(_e) => {
          // TODO
          debug!("No match!");
        }
      }
    }
  }
}
