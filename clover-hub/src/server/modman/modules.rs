//! # Clover Module Management
//!
//! This *rust* module defines how clover modules should be registered to begin usage.
//!
//! ## Configuration
//!
//! Module configuration is located in [`models`](super::models), which provides two types of modules:
//!
//! - Basic modules, controlled by ModMan, which get commands that are generated from [Gestures](super::gestures),
//! - and App modules, which are controlled by [Apps](crate::server::appd), and only use their module manifest entry to give them permissions.
//!
//! In general, when we need to register a module, we look for it on a [bus](super::busses) (modules share a Bus (or direct Zenoh) connection with their components), and then try to double check that all the [Components](super::components) are present (and perhaps run a few checks if they're configured). We do the inverse when stopping the server process.
//!
//! ## Security
//!
//! Supported modules are required to use an asynchronous signature for all communications with Zenoh over a ModMan Proxy. An asymmetric encryption key is registered during the registration flow to ensure that all messages are legitimate.
//!
//! This is due to Clover's security first design. No security is not an option.
//!
//! ### Security Levels
//!
//! Level 1 and 2 are designed for simple modules that do not have movement components. Regardless, Level 3 or 4 (using post-quantum encryption algorithms) is suggested for production modules and especially for modules with movement components.
//!
//! If a module does not use Post-Quantum Level 3 or 4 security and have a movement component, users will be warned of this fact using a non-dismissible UI component if the configuration application is CORE/Spanner compliant!
//!
//! #### Level 1
//!
//! The async key provided to Clover is private to a single instance and should be deleted after registration, or be hidden during normal usage if that is not feasible. (such as when using an embossed QR code on a module without a built-in display.)
//!
//! #### Level 2
//!
//! Clover will generate an async key and provide it to the module during the registration flow to ensure that messages from Clover are legitimate.
//!
//! #### Level 3
//!
//! Similar to Level 2, however, all messages are encrypted using symmetric encryption, with the key attached to the message, encrypted using the private key of the transmitting party, a.k.a. Hybrid Cryptography.
//!
//! #### Level 4
//!
//! Similar to Level 3, however, the asymmetric keys are changed constantly to ensure perfect forward secrecy.
//!

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
    None => {
      failiure = Some(anyhow::anyhow!(
        "Unable to find component {} in store!",
        component_id.clone()
      ));
    }
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
          } else {
            initialized_module = true;
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
