mod system_ui;

use self::system_ui::system_ui_main;
use crate::{
  server::evtbuzz::models::{
    CoreUserConfig,
    IPCMessageWithId,
    Store,
  },
  utils::{
    RecvSync,
    send_ipc_message,
  },
};
use log::{
  debug,
  info,
};
use nexus::user::NexusUser;
use queues::*;
use std::sync::Arc;
use system_ui::{
  CustomBevyIPC,
  ExitState,
};
use tokio::sync::mpsc::{
  UnboundedReceiver,
  UnboundedSender,
  unbounded_channel,
};
use tokio_util::sync::CancellationToken;
use url::Url;
use nexus::{server::models::UserConfig, arbiter::models::ApiKeyWithoutUID, user::NexusUser};

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.renderer",
    pretty_name: "Clover: Renderer",
    api_keys: vec![
      ApiKeyWithoutUID {
        allowed_events_to: "^nexus://com.reboot-codes.clover.renderer(\\.(.*))*(\\/.*)*$"
        allowed_events_from: "^nexus://com.reboot-codes.clover.renderer(\\.(.*))*(\\/.*)*$",
        echo: false,
        proxy: false
      }
    ]
  }
}

#[derive(Debug, Clone)]
pub struct RendererStore {}

pub async fn renderer_main(
  store: Arc<RendererStore>,
  user_config: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Renderer...");

  let (bevy_cancel_tx, bevy_cancel_rx) = std::sync::mpsc::channel();
  let bevy_cancel_ipc = CustomBevyIPC {
    exit_channel: RecvSync(bevy_cancel_rx),
    display_registration_queue: queue![],
  };

  std::thread::spawn(|| system_ui_main(bevy_cancel_ipc));

  // let display_handles = Arc::new(HashMap::new());
  let (from_tx, mut from_rx) = unbounded_channel::<IPCMessageWithId>();

  let init_user = Arc::new(user.clone());
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      client.send(
        "nexus://com.reboot-codes.clover.renderer/status".to_string(),
        "finished-init".to_string(),
      )
      .await;
    })
    .await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let (ipc_rx, ipc_handle) = client.subscribe();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
        _ = ipc_recv_token.cancelled() => {
            debug!("ipc_recv exited");
        },
        _ = async move {
            while let Ok(msg) = ipc_rx.recv().await {
                let kind = Url::parse(&msg.kind.clone()).unwrap();

                // Verify that we care about this event.
                if kind.host().unwrap() == url::Host::Domain("com.reboot-codes.clover.renderer") {
                    debug!("Processing: {}", msg.kind.clone());

                    if kind.path() == "/register-display" {

                    }
                }
            }
        } => {}
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
