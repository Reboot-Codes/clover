mod gestures;

use anyhow::anyhow;
use gestures::handle_gesture_cmd;
use log::{
  debug,
  error,
  warn,
};
use nexus::server::models::IPCMessageWithId;
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;
use tokio::sync::broadcast::Sender;
use url::Url;

use crate::{
  server::modman::models::{
    GestureCommand,
    ModManStore,
  },
  utils::deserialize_base64,
};
use std::{
  str::FromStr,
  sync::Arc,
};

#[derive(Debug, PartialEq)]
pub enum Events {
  Status,
  Gesture,
  None,
}

impl FromStr for Events {
  type Err = anyhow::Error;

  fn from_str(input: &str) -> Result<Events, Self::Err> {
    match input {
      "/gesture" => Ok(Events::Gesture),
      "/status" => Ok(Events::Status),
      "" => Ok(Events::None),
      "/" => Ok(Events::None),
      _ => Err(anyhow!("String \"{}\" not part of enum!", input)),
    }
  }
}

pub async fn handle_ipc_msg(store: ModManStore, ipc_rx: Sender<IPCMessageWithId>) {
  let store_arc = Arc::new(store.clone());

  while let Ok(msg) = ipc_rx.subscribe().recv().await {
    let kind = Url::parse(&msg.kind.clone()).unwrap();

    // Verify that we care about this event.
    if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.modman") {
      debug!("Processing: \"{}\"...", kind.path());

      match Events::from_str(kind.path()) {
        Ok(event_type) => {
          match event_type {
            Events::Status => {
              debug!("Return status?");
            }
            Events::Gesture => {
              debug!("Parsing event data...");

              let mut gesture_id = None;
              for (key, val) in kind.query_pairs() {
                if key == "gesture_id" {
                  gesture_id = Some(val.to_string());
                }
              }

              match gesture_id {
                Some(gesture_id_str) => {
                  debug!("Parsed gesture id: {}", gesture_id_str.clone());
                  let mut gesture_command = None;

                  match deserialize_base64::<GestureCommand>(msg.message.clone().as_bytes()) {
                    Ok(obj) => gesture_command = Some(obj),
                    Err(e) => {
                      // TODO
                      error!("Error when parsing gesture command data: {}", e);
                    }
                  }

                  match gesture_command {
                    Some(cmd) => {
                      handle_gesture_cmd(
                        &mut store_arc.clone(),
                        gesture_id_str.clone(),
                        cmd.clone(),
                      )
                      .await
                    }
                    None => {
                      error!("Parsed gesture ID and data, but it was not set??");
                    }
                  }
                }
                None => {
                  // TODO reply!
                  warn!("Gesture ID not included! Use state event instead.");
                }
              }
            }
            _ => {
              debug!("Blank event... doing nothing.");
            }
          }
        }
        Err(e) => {
          debug!("Failed to parse path: {}, due to: {}", kind.path(), e);
        }
      }
    }
  }
}
