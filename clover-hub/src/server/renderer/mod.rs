pub mod ipc;
pub mod models;
pub mod system_ui;

use self::system_ui::system_ui_main;
use crate::utils::RecvSync;
use ipc::handle_ipc_msg;
use log::error;
use log::{
  debug,
  info,
};
use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
  user::NexusUser,
};
use queues::*;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use system_ui::SystemUIIPC;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use super::warehouse::config::models::Config;

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.renderer".to_string(),
    pretty_name: "Clover: Renderer".to_string(),
    api_keys: vec![ApiKeyWithoutUID {
      allowed_events_to: vec![
        "^nexus://com.reboot-codes.clover.renderer(\\.(.*))*(\\/.*)*$".to_string(),
      ],
      allowed_events_from: vec![
        "^nexus://com.reboot-codes.clover.renderer(\\.(.*))*(\\/.*)*$".to_string(),
        "^nexus://com.reboot-codes.clover.modman(\\.(.*))*(\\/.*)*$".to_string(),
      ],
      echo: false,
      proxy: false,
    }],
  }
}

#[derive(Debug, Clone)]
pub struct RendererStore {
  pub config: Arc<Mutex<Config>>,
}

impl RendererStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      Option::None => Arc::new(Mutex::new(Config::default())),
    };

    RendererStore { config }
  }
}

pub async fn renderer_main(
  store: RendererStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Renderer...");

  let display_registration_queue = Arc::new(StdMutex::new(queue![]));

  let (bevy_cancel_tx, bevy_cancel_rx) = std::sync::mpsc::channel();
  let system_ui_ipc = SystemUIIPC {
    exit_channel: RecvSync(bevy_cancel_rx),
    display_registration_queue: display_registration_queue.clone(),
  };

  // TODO: Add this as a CLI/Config option!
  let custom_bevy_ipc = system_ui_ipc;
  std::thread::spawn(move || system_ui_main(custom_bevy_ipc, Some(false)));

  // let display_handles = Arc::new(HashMap::new());

  let init_user = Arc::new(user.clone());
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      match init_user.send(
        &"nexus://com.reboot-codes.clover.renderer/status".to_string(),
        &"finished-init".to_string(),
        &None,
      ) {
        Err(e) => {
          error!(
            "Error when letting peers know about completed init state: {}",
            e
          );
        }
        _ => {}
      }

      info!("Requesting displays to setup in SystemUI from ModMan...");
      match init_user.send(
        &"nexus://com.reboot-codes.clover.modman/init-displays".to_string(),
        &"first-time".to_string(),
        &None,
      ) {
        Err(e) => {
          error!(
            "Error when requesting display configurations from peers: {}",
            e
          );
        }
        _ => {}
      }
    })
    .await;

  let ipc_display_registration_queue = display_registration_queue.clone();
  let ipc_recv_token = cancellation_tokens.0.clone();
  let (ipc_rx, ipc_handle) = user.subscribe();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
        _ = ipc_recv_token.cancelled() => {
          debug!("ipc_recv exited");
        },
        _ = handle_ipc_msg(ipc_rx, ipc_display_registration_queue) => {}
    }
  });

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_handle.abort();

      info!("Cleaning up displays...");

      let _ = bevy_cancel_tx.send(bevy::prelude::AppExit::Success);

      // TODO: Clean up registered displays when server is shutting down.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Renderer has stopped!");
}
