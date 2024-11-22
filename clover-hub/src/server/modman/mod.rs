pub mod models;
pub mod displays;
pub mod movement;

use std::sync::Arc;
use displays::init_display_component;
use log::{debug, error, info, warn};
use models::Module;
use movement::init_movement_component;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::CancellationToken;
use url::Url;
use crate::utils::send_ipc_message;

use super::evtbuzz::models::{IPCMessageWithId, CoreUserConfig, Store};

async fn init_module(store: &Store, id: String, module: Module) -> (bool, usize) {
  let mut initialized_module = module.initialized;
  let mut initialized_module_components = 0;

  if !initialized_module {
    if module.components.len() == 0 {
      warn!("Module: {}, does not have any components, skipping.", id.clone());
      initialized_module = true;
    } else {
      for (component_id, component) in module.components.iter() {
        if component.component_type.clone().starts_with(&"com.reboot-codes.clover.display".to_string()) {
          if init_display_component(&store, module.clone(), component_id.clone(), component.clone()) { initialized_module_components += 1; }
        }

        if component.component_type.clone().starts_with(&"com.reboot-codes.clover.movement".to_string()) {
          if init_movement_component(&store, module.clone(), component_id.clone(), component.clone()) { initialized_module_components += 1; }
        }
  
        // TODO: Add init functions for other component types.
      }
  
      if initialized_module_components != module.components.len() {
        if initialized_module_components > 0 {
          warn!("Module: {}, only initialized {} out of {} components!", id.clone(), initialized_module_components, module.components.len());
          initialized_module = true;
        } else {
          error!("Module: {}, failed to initialize!", id.clone());
        }
      }
    }
  }

  if initialized_module {
    // Update the store with new state of the module.
    if initialized_module {
      store.modules.lock().await.insert(id.clone(), Module {
        module_type: module.module_type.clone(),
        pretty_name: module.pretty_name.clone(),
        initialized: true,
        components: module.components.clone(),
        registered_by: module.registered_by.clone()
      });
      info!("Module: {} ({}), Initialized!", module.pretty_name.clone(), id.clone());
    }
  }

  (initialized_module, initialized_module_components)
}

pub async fn modman_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>,
  cancellation_tokens: (CancellationToken, CancellationToken)
) {
  info!("Starting ModMan...");

  let init_store = Arc::new(store.clone());
  let init_user = Arc::new(user_config.clone());
  let (init_from_tx, mut init_from_rx) = unbounded_channel::<IPCMessageWithId>();
  cancellation_tokens.0.run_until_cancelled(async move {
    let modules = init_store.modules.lock().await;
    if modules.len() > 0 {
      // Initialize modules that were registered already via persistence.
      for (id, module) in modules.iter() {
        info!("Initializing pre configured module: {}:\n  type: {}\n  name: {}", id.clone(), module.module_type.clone(), module.pretty_name.clone());
        let _components_initialized = init_module(&init_store, id.clone(), module.clone()).await;
      }
    } else {
      info!("No pre-configured modules to initialize.");
    }

    let _ = send_ipc_message(
      &init_store, 
      &init_user, 
      init_from_tx, 
      "clover://modman.clover.reboot-codes.com/status".to_string(), 
      "finished-init".to_string()
    ).await;
  }).await;

  let ipc_recv_token = cancellation_tokens.0.clone();
  let ipc_recv_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = ipc_recv_token.cancelled() => {
        debug!("ipc_recv exited");
      },
      _ = async move {
        while let Some(msg) = ipc_rx.recv().await {
          let kind = Url::parse(&msg.kind.clone()).unwrap();

          // Verify that we care about this event.
          if kind.host().unwrap() == url::Host::Domain("modman.clover.reboot-codes.com") {
            debug!("Processing: {}", msg.kind.clone());
          }
        }
      } => {}
    }
  });

  let ipc_trans_token = cancellation_tokens.0.clone();
  let ipc_trans_tx = Arc::new(ipc_tx.clone());
  let ipc_trans_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = async move {
        while let Some(msg) = init_from_rx.recv().await {
          match ipc_trans_tx.send(msg) {
            Ok(_) => {},
            Err(_) => {
              debug!("Failed to send message to IPC bus!");
            }
          }
        }
      } => {},
      _ = ipc_trans_token.cancelled() => {
        debug!("ipc_trans exited");
      }
    }
  });

  let mod_clean_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = mod_clean_token.cancelled() => {
      ipc_recv_handle.abort();
      ipc_trans_handle.abort();

      // Clean up all modules on shutdown.
      info!("Cleaning up modules...");

      tokio::select! {
        modules = store.modules.lock() => {
          debug!("done waiting for lock");
          if modules.len() > 0 {
            for (id, module) in modules.iter() {
              if module.initialized {
                info!("De-initializing configured module: {}:\n  type: {}\n  name: {}", id.clone(), module.module_type.clone(), module.pretty_name.clone());
                // let (de_initialized, _components_de_initialized) = de_init_module(&store, id.clone(), module.clone()).await;
                let de_initialized = true;
                
                // Update the store with new state of the module.
                if de_initialized {
                  store.modules.lock().await.insert(id.clone(), Module {
                    module_type: module.module_type.clone(),
                    pretty_name: module.pretty_name.clone(),
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
