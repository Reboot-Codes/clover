//! # Renderer
//!
//! Uses graphical acceleration to render and display graphics on any connected [display components](super::modman::components::video::displays). Primary thread execution starts with [`renderer_main`].
//!
//! The renderer service is *only* responsible for creating, managing, destroying, and writing to an arbitrary number of OpenGL/Vulkan contexts who's frames are captured and sent to displays registered with modman when permitted by the user.
//!
//! The internal bevy-based engine is found within the [System UI](system_ui) package.
//!

pub mod ipc;
pub mod models;
pub mod system_ui;

use self::system_ui::system_ui_main;
use crate::server::renderer::ipc::handle_ipc;
use crate::utils::one_off_message;
use crate::utils::RecvSync;
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
use tracing::{
  debug,
  error,
  info,
  instrument,
};

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

#[instrument(skip(store, user, cancellation_tokens))]
pub async fn renderer_main(
  store: RendererStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting Renderer...");

  let mut zenoh_config = zenoh::Config::default();

  zenoh_config.insert_json5("connect/endpoints", "tcp/localhost:6699");

  debug!("Connecting to Zenoh...");
  let session = Arc::new(zenoh::open(zenoh_config).await.unwrap());
  debug!("Connected to Zenoh!");

  let ipc_token = cancellation_tokens.0.clone();
  let ipc_session = session.clone();
  let ipc_handle = tokio::task::spawn(handle_ipc(ipc_token, ipc_session));

  let display_registration_queue = Arc::new(StdMutex::new(queue![]));

  let (bevy_cancel_tx, bevy_cancel_rx) = std::sync::mpsc::channel();
  let system_ui_ipc = SystemUIIPC {
    exit_channel: RecvSync(bevy_cancel_rx),
    display_registration_queue: display_registration_queue.clone(),
  };

  // TODO: Add this as a CLI/Config option!
  let custom_bevy_ipc = system_ui_ipc;
  let bevy_thread = std::thread::spawn(move || system_ui_main(custom_bevy_ipc, Some(false)));

  // let display_handles = Arc::new(HashMap::new());

  let init_session = session.clone();
  let init_user = Arc::new(user.clone());
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
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

      one_off_message(
        init_session.clone(),
        &"com/reboot-codes/clover/server/renderer/status".to_string(),
        &"ready".to_string(),
      )
      .await;
    })
    .await;

  let cleanup_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = cleanup_token.cancelled() => {
      ipc_handle.abort();

      info!("Cleaning up displays...");

      let _ = bevy_cancel_tx.send(bevy::prelude::AppExit::Success);

      info!("Waiting for Bevy to shut down...");
      match bevy_thread.join() {
        Ok(_) => {
          info!("Bevy thread exited cleanly");
        }
        Err(e) => {
          error!("Bevy thread panicked during shutdown: {:?}", e);
        }
      }

      // TODO: Clean up registered displays when server is shutting down.

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("Renderer has stopped!");
}
