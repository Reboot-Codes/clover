use log::{
  debug,
  info,
};
use nexus::server::models::IPCMessageWithId;
use serde::{
  Deserialize,
  Serialize,
};
use strum::VariantNames;
use tokio::sync::broadcast::Sender;
use url::Url;

#[derive(Deserialize, Serialize, VariantNames)]
pub enum Events {
  #[serde(rename = "/status")]
  #[strum(serialize = "/status")]
  Status,
  #[serde(rename = "/gesture/begin")]
  #[strum(serialize = "/gesture/begin")]
  GestureBegin,
}

pub async fn handle_ipc_msg(ipc_rx: Sender<IPCMessageWithId>) {
  while let Ok(msg) = ipc_rx.subscribe().recv().await {
    let kind = Url::parse(&msg.kind.clone()).unwrap();

    // Verify that we care about this event.
    if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.renderer") {
      debug!("Processing: {}", msg.kind.clone());

      match serde_json_lenient::from_str::<Events>(&format!("\"{}\"", kind.path())) {
        Ok(event_type) => {
          match event_type {
            Events::Status => {
              debug!("Return status?");
            }
            Events::GestureBegin => {
              let mut gesture_id = None;
              for (key, val) in kind.query_pairs() {
                if key == "gesture_id" {
                  gesture_id = Some(val.to_string());
                }
              }

              match gesture_id {
                Some(gesture_id_str) => {
                  info!("Begining gesture \"{}\"...", gesture_id_str.clone());
                }
                Option::None => {
                  // TODO
                }
              }
            }
          }
        }
        Err(_e) => {
          // TODO
          todo!();
        }
      }
    }
  }
}
