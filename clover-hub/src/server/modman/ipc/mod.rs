mod gestures;

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
  server::modman::models::GestureCommand,
  utils::deserialize_base64,
};

#[derive(Deserialize, Serialize, VariantNames)]
pub enum Events {
  #[serde(rename = "/status")]
  #[strum(serialize = "/status")]
  Status,
  #[serde(rename = "/gesture")]
  #[strum(serialize = "/gesture")]
  Gesture,
}

pub async fn handle_ipc_msg(ipc_rx: Sender<IPCMessageWithId>) {
  while let Ok(msg) = ipc_rx.subscribe().recv().await {
    let kind = Url::parse(&msg.kind.clone()).unwrap();

    // Verify that we care about this event.
    if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.modman") {
      debug!("Processing: {}", msg.kind.clone());

      match serde_json_lenient::from_str::<Events>(&format!("\"{}\"", kind.path())) {
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
                    Some(cmd) => handle_gesture_cmd(gesture_id_str.clone(), cmd.clone()),
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
          }
        }
        Err(e) => {}
      }
    }
  }
}
