use super::{
  components::models::CloverComponentTrait,
  models::{
    ModManStore,
    Module,
  },
};
use log::{
  error,
  info,
  warn,
};
use std::sync::Arc;

pub async fn init_module(store: &ModManStore, id: String, module: Module) -> (bool, usize) {
  let mut initialized_module = module.initialized;
  let mut initialized_module_components = 0;

  if !initialized_module {
    if module.components.len() == 0 {
      warn!(
        "Module: {}, does not have any components, skipping.",
        id.clone()
      );
      initialized_module = true;
    } else {
      let mut critical_failiure = None;

      for (component_id, component_arc) in module.components.iter() {
        let component_arc_binding = component_arc.clone();
        let component_guard = component_arc_binding.lock().await;
        let (component_meta, mut component) = (
          component_guard.clone().0.clone(),
          component_guard.clone().1.clone(),
        );

        if component_meta.critical {
          info!(
            "Module: {}, initalizing CRITICAL component: {}...",
            id.clone(),
            component_id.clone()
          );
        } else {
          info!(
            "Module: {}, initalizing component: {}...",
            id.clone(),
            component_id.clone()
          );
        }

        let component_initalized = match component.init(Arc::new(store.clone())).await {
          Ok(_) => true,
          Err(e) => {
            error!(
              "Failed to initalize component \"{}\", due to: {}",
              component_id.clone(),
              e
            );
            false
          }
        };

        if component_initalized {
          info!(
            "Module: {}, successfully initalized component: {}!",
            id.clone(),
            component_id.clone()
          );
          initialized_module_components += 1;
        } else {
          if component_meta.critical {
            critical_failiure = Some(component_id.clone());
          } else {
            warn!(
              "Module: {}, failed to initialize component: {}!",
              id.clone(),
              component_id.clone()
            );
          }
        }

        match critical_failiure {
          Some(_) => {
            break;
          }
          None => {}
        }

        std::mem::drop(component_guard);
      }

      match critical_failiure {
        Some(component_id) => {
          error!(
            "Module: {}, failed to initalize critical component: {}!",
            id.clone(),
            component_id.clone()
          );
        }
        None => {
          if initialized_module_components != module.components.len() {
            if initialized_module_components > 0 {
              warn!(
                "Module: {}, only initialized {} out of {} components!",
                id.clone(),
                initialized_module_components,
                module.components.len()
              );
              initialized_module = true;
            } else {
              error!("Module: {}, failed to initialize!", id.clone());
            }
          }
        }
      }
    }
  }

  if initialized_module {
    // Update the store with new state of the module.
    if initialized_module {
      store.modules.lock().await.insert(
        id.clone(),
        Module {
          module_type: module.module_type.clone(),
          module_name: module.module_name.clone(),
          custom_name: module.custom_name.clone(),
          initialized: true,
          components: module.components.clone(),
          registered_by: module.registered_by.clone(),
        },
      );

      info!(
        "Module: {} ({}), Initialized!",
        module.get_name(),
        id.clone()
      );
    }
  }

  (initialized_module, initialized_module_components)
}

// TODO: deinit_module(); impl!
