pub mod ipc;
pub mod system_ui;

use self::system_ui::system_ui_main;
use crate::utils::RecvSync;
use ipc::handle_ipc_msg;
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
use system_ui::{
  CustomBevyIPC,
  ExitState,
};
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

  let (bevy_cancel_tx, bevy_cancel_rx) = std::sync::mpsc::channel();
  let bevy_cancel_ipc = CustomBevyIPC {
    exit_channel: RecvSync(bevy_cancel_rx),
    display_registration_queue: queue![],
  };

  // TODO: Add this as a CLI/Config option!
  std::thread::spawn(|| system_ui_main(bevy_cancel_ipc, Some(true)));

  // let display_handles = Arc::new(HashMap::new());

  let init_user = Arc::new(user.clone());
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      let _ = init_user.send(
        &"nexus://com.reboot-codes.clover.renderer/status".to_string(),
        &"finished-init".to_string(),
        &None,
      );
    })
    .await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let (ipc_rx, ipc_handle) = user.subscribe();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
        _ = ipc_recv_token.cancelled() => {
            debug!("ipc_recv exited");
        },
        _ = handle_ipc_msg(ipc_rx) => {}
    }
  });

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_handle.abort();

      info!("Cleaning up displays...");

      let _ = bevy_cancel_tx.send(ExitState::Success);

      // TODO: Clean up registered displays when server is shutting down.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Renderer has stopped!");
}
