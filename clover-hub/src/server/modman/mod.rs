pub mod busses;
pub mod components;
pub mod ipc;
pub mod models;
pub mod modules;

use crate::server::modman::modules::init_module;
use ipc::handle_ipc_msg;
use log::{
  debug,
  info,
};
use models::{
  ModManStore,
  Module,
};
use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
  user::NexusUser,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

pub async fn gen_user() -> UserConfig {
  UserConfig {
    user_type: "com.reboot-codes.com.clover.modman".to_string(),
    pretty_name: "Clover: ModMan".to_string(),
    api_keys: vec![ApiKeyWithoutUID {
      allowed_events_to: vec![
        "^nexus://com.reboot-codes.clover.modman(\\.(.*))*(\\/.*)*$".to_string()
      ],
      allowed_events_from: vec![
        "^nexus://com.reboot-codes.clover.modman(\\.(.*))*(\\/.*)*$".to_string()
      ],
      echo: false,
      proxy: false,
    }],
  }
}

pub async fn modman_main(
  store: ModManStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting ModMan...");

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user.clone());
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      let modules = init_store.modules.lock().await;
      if modules.len() > 0 {
        // Initialize modules that were registered already via configuration and persistence.
        for (id, module) in modules.iter() {
          info!(
            "Initializing pre configured module: {}:\n  type: {}\n  name: {}",
            id.clone(),
            module.module_type.clone(),
            module.get_name()
          );
          let _components_initialized = init_module(&init_store, id.clone(), module.clone()).await;
        }
      } else {
        info!("No pre-configured modules to initialize.");
      }

      init_user.send(
        &"nexus://com.reboot-codes.clover.modman/status".to_string(),
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

  let mod_clean_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = mod_clean_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_handle.abort();

      // Clean up all modules on shutdown.
      info!("Cleaning up modules...");

      tokio::select! {
        modules = store.modules.lock() => {
          debug!("done waiting for lock");
          if modules.len() > 0 {
            for (id, module) in modules.iter() {
              if module.initialized {
                info!("De-initializing configured module: {}:\n  type: {}\n  name: {}", id.clone(), module.module_type.clone(), module.get_name());
                // let (de_initialized, _components_de_initialized) = deinit_module(&store, id.clone(), module.clone()).await;
                let de_initialized = true;

                // Update the store with new state of the module.
                if de_initialized {
                  store.modules.lock().await.insert(id.clone(), Module {
                    module_type: module.module_type.clone(),
                    module_name: module.module_name.clone(),
                    custom_name: module.custom_name.clone(),
                    initialized: false,
                    components: module.components.clone(),
                    registered_by: module.registered_by.clone()
                  });
                }
              }
            }
          } else {
            debug!("No modules to de-init.");
          }
        }
      }

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("ModMan has stopped!");
}
