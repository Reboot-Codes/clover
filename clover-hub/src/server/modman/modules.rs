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

pub async fn init_component(
  store: &ModManStore,
  module_id: String,
  component_id: String,
  is_critical: &bool,
) -> Result<(), anyhow::Error> {
  let components = store.components.lock().await;
  let mut failiure = None;

  match components.get(&component_id.clone()) {
    Some(component_tuple) => {
      let (component_meta, mut component) = (
        component_tuple.clone().0.clone(),
        component_tuple.clone().1.clone(),
      );

      if component_meta.critical {
        info!(
          "Module: {}, initalizing CRITICAL component: {}...",
          module_id.clone(),
          component_id.clone()
        );
      } else {
        info!(
          "Module: {}, initalizing component: {}...",
          module_id.clone(),
          component_id.clone()
        );
      }

      let component_initalized = match component.init(Arc::new(store.clone())).await {
        Ok(_) => true,
        Err(e) => {
          failiure = Some(e);
          false
        }
      };

      if component_initalized {
        info!(
          "Module: {}, successfully initalized component: {}!",
          module_id.clone(),
          component_id.clone()
        );
      } else {
        if !is_critical.to_owned() {
          warn!(
            "Module: {}, failed to initialize component: {}!",
            module_id.clone(),
            component_id.clone()
          );
        }
      }
    }
    None => todo!(),
  }

  match failiure {
    Some(e) => Err(e),
    Option::None => Ok(()),
  }
}

pub async fn init_module(store: &ModManStore, id: String, module: Module) -> (bool, usize) {
  let mut initialized_module = module.initialized;
  let mut initialized_module_components = 0;

  info!(
    "Initializing module: {}:\n  type: {}\n  name: {}",
    id.clone(),
    module.module_type.clone(),
    module.get_name()
  );

  if !initialized_module {
    if module.components.len() == 0 {
      warn!(
        "Module: {}, does not have any components, skipping.",
        id.clone()
      );
      initialized_module = true;
    } else {
      let mut critical_failiure = None;

      for (component_id, is_critical) in module.components.iter() {
        match init_component(
          &store.clone(),
          id.clone(),
          component_id.clone(),
          is_critical,
        )
        .await
        {
          Ok(_) => {
            initialized_module_components += 1;
          }
          Err(e) => {
            if is_critical.to_owned() {
              critical_failiure = Some((component_id.clone(), e));
            } else {
              error!(
                "Module: {}, Failed to initalize component \"{}\", due to: {}",
                id.clone(),
                component_id.clone(),
                e
              );
            }
          }
        }
      }

      match critical_failiure {
        Some(failiure) => {
          let (component_id, e) = failiure;
          error!(
            "Module: {}, failed to initalize critical component: {}, due to: {}\nSkipping rest of Module init...",
            id.clone(),
            component_id,
            e
          );
        }
        Option::None => {
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

pub async fn deinit_component(
  store: &ModManStore,
  module_id: String,
  component_id: String,
  is_critical: &bool,
) -> Result<(), anyhow::Error> {
  let components = store.components.lock().await;
  let mut failiure = None;

  match components.get(&component_id.clone()) {
    Some(component_tuple) => {
      let (component_meta, mut component) = (
        component_tuple.clone().0.clone(),
        component_tuple.clone().1.clone(),
      );

      if component_meta.critical {
        info!(
          "Module: {}, deinitalizing CRITICAL component: {}...",
          module_id.clone(),
          component_id.clone()
        );
      } else {
        info!(
          "Module: {}, deinitalizing component: {}...",
          module_id.clone(),
          component_id.clone()
        );
      }

      let component_initalized = match component.deinit(Arc::new(store.clone())).await {
        Ok(_) => true,
        Err(e) => {
          failiure = Some(e);
          false
        }
      };

      if component_initalized {
        info!(
          "Module: {}, successfully deinitalized component: {}!",
          module_id.clone(),
          component_id.clone()
        );
      } else {
        if !is_critical.to_owned() {
          warn!(
            "Module: {}, failed to deinitialize component: {}!",
            module_id.clone(),
            component_id.clone()
          );
        }
      }
    }
    None => todo!(),
  }

  match failiure {
    Some(e) => Err(e),
    Option::None => Ok(()),
  }
}

pub async fn deinit_module(store: &ModManStore, id: String, module: Module) -> (bool, usize) {
  let mut initialized_module = module.initialized;
  let mut deinitialized_module_components = 0;

  info!(
    "De-initializing module: {}:\n  type: {}\n  name: {}",
    id.clone(),
    module.module_type.clone(),
    module.get_name()
  );

  if initialized_module {
    if module.components.len() == 0 {
      warn!(
        "Module: {}, does not have any components, skipping.",
        id.clone()
      );
      initialized_module = false;
    } else {
      let mut critical_failiure = None;

      for (component_id, is_critical) in module.components.iter() {
        match deinit_component(
          &store.clone(),
          id.clone(),
          component_id.clone(),
          is_critical,
        )
        .await
        {
          Ok(_) => {
            deinitialized_module_components += 1;
          }
          Err(e) => {
            if is_critical.to_owned() {
              critical_failiure = Some((component_id.clone(), e));
            } else {
              error!(
                "Module: {}, Failed to deinitalize component \"{}\", due to: {}",
                id.clone(),
                component_id.clone(),
                e
              );
            }
          }
        }
      }

      match critical_failiure {
        Some(failiure) => {
          let (component_id, e) = failiure;
          error!(
            "Module: {}, failed to deinitalize critical component: {}, due to: {}\nSkipping rest of Module init...",
            id.clone(),
            component_id,
            e
          );
        }
        Option::None => {
          if deinitialized_module_components != module.components.len() {
            if deinitialized_module_components > 0 {
              warn!(
                "Module: {}, only deinitialized {} out of {} components!",
                id.clone(),
                deinitialized_module_components,
                module.components.len()
              );
              initialized_module = true;
            } else {
              error!("Module: {}, failed to deinitialize!", id.clone());
            }
          }
        }
      }
    }
  }

  if !initialized_module {
    // Update the store with new state of the module.
    if !initialized_module {
      store.modules.lock().await.insert(
        id.clone(),
        Module {
          module_type: module.module_type.clone(),
          module_name: module.module_name.clone(),
          custom_name: module.custom_name.clone(),
          initialized: false,
          components: module.components.clone(),
          registered_by: module.registered_by.clone(),
        },
      );

      info!(
        "Module: {} ({}), Deinitialized!",
        module.get_name(),
        id.clone()
      );
    }
  }

  (!initialized_module, deinitialized_module_components)
}
