pub mod busses;
pub mod components;
pub mod ipc;
pub mod models;
pub mod modules;

use busses::start_busses;
use ipc::handle_ipc_msg;
use log::{
  debug,
  info,
  warn,
};
use models::ModManStore;
use modules::{
  deinit_module,
  init_module,
};
use nexus::{
  arbiter::models::ApiKeyWithoutUID,
  server::models::UserConfig,
  user::NexusUser,
};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

/// The minimum required permissions and configuration for ModMan to use Nexus.
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

/// Begin the ModMan threads and sub-processes to ensure module/compoent communications.
pub async fn modman_main(
  store: ModManStore,
  user: NexusUser,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting ModMan...");

  let bus_store = Arc::new(store.clone());
  let bus_token = cancellation_tokens.0.clone();
  let bus_user = Arc::new(user.clone());
  let bus_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = bus_token.cancelled() => {
        debug!("bus_handle exited");
      },
      _ = start_busses(bus_store, bus_user) => {}
    }
  });

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user.clone());
  cancellation_tokens
    .0
    .run_until_cancelled(async move {
      let modules = init_store.modules.lock().await;
      let mut modules_initalized: usize = 0;

      if modules.len() > 0 {
        // Initialize modules that were registered already via configuration and persistence.
        for (id, module) in modules.iter() {
          let (initialized, _components_initialized) =
            init_module(&init_store, id.clone(), module.clone()).await;

          if initialized {
            modules_initalized += 1;
          }
        }
      } else {
        info!("No pre-configured modules to initialize.");
      }

      if modules_initalized != modules.len() {
        warn!(
          "Initalized {} out of {} module(s)!",
          modules_initalized,
          modules.len()
        );
        init_user.send(
          &"nexus://com.reboot-codes.clover.appd/status".to_string(),
          &"incomplete-init".to_string(),
          &None,
        );
      } else {
        if modules_initalized != 0 {
          info!("Initalized all {} module(s)", modules_initalized);
        }
        init_user.send(
          &"nexus://com.reboot-codes.clover.modman/status".to_string(),
          &"finished-init".to_string(),
          &None,
        );
      }
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
      bus_handle.abort();
      ipc_recv_handle.abort();
      ipc_handle.abort();

      info!("Cleaning up modules...");

      // TODO: Add override cancellation token to force stop!

      tokio::select! {
        modules = store.modules.lock() => {
          debug!("done waiting for lock");
          if modules.len() > 0 {
            let mut modules_deinitalized: usize = 0;

            for (id, module) in modules.iter() {
              if module.initialized {
                let (de_initialized, _components_deinitialized) = deinit_module(&store, id.clone(), module.clone()).await;

                if de_initialized { modules_deinitalized += 1; }
              }
            }

            if modules_deinitalized != modules.len() {
              warn!("Deinitalized {} out of {} module(s)!", modules_deinitalized, modules.len());
            } else {
              info!("Deinitalized all {} module(s)", modules_deinitalized);
            }
          } else {
            debug!("No modules to deinit.");
          }
        }
      }

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("ModMan has stopped!");
}
