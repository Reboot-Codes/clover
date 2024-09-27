pub mod models;
pub mod displays;

use std::sync::Arc;
use displays::init_display;
use log::{debug, error, info, warn};
use models::Module;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use url::Url;
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
          if init_display(&store, module.clone(), component_id.clone(), component.clone()) { initialized_module_components += 1; }
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

  (initialized_module, initialized_module_components)
}

pub async fn modman_main(
  ipc_tx: UnboundedSender<IPCMessageWithId>, 
  mut ipc_rx: UnboundedReceiver<IPCMessageWithId>, 
  store: Arc<Store>, 
  user_config: Arc<CoreUserConfig>
) {
  info!("Starting ModMan...");

  // Initialize modules that were registered already via persistence.
  for (id, module) in store.modules.lock().await.iter() {
    info!("Initializing pre configured module: {}:\n  type: {}\n  name: {}", id.clone(), module.module_type.clone(), module.pretty_name.clone());
    let (initialized, _components_initialized) = init_module(&store, id.clone(), module.clone()).await;

    // Update the store with new state of the module.
    if initialized {
      store.modules.lock().await.insert(id.clone(), Module {
        module_type: module.module_type.clone(),
        pretty_name: module.pretty_name.clone(),
        initialized: true,
        components: module.components.clone()
      });
    }
  }

  let ipc_recv_handle = tokio::task::spawn(async move {
    while let Some(msg) = ipc_rx.recv().await {
      let kind = Url::parse(&msg.kind.clone()).unwrap();

      // Verify that we care about this event.
      if kind.host().unwrap() == url::Host::Domain("modman.clover.reboot-codes.com") {
        debug!("Processing: {}", msg.kind.clone());
      }
    }
  });

  futures::future::join_all(vec![ipc_recv_handle]).await;

  // Clean up all modules on shutdown.
  info!("Cleaning up modules...");
  for (id, module) in store.modules.lock().await.iter() {
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
          components: module.components.clone()
        });
      }
    }
  }
}
