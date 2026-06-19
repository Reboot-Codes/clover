//! # ModMan
//!
//! Short for Module Manager.
//!
//! Manages [communication](busses) with [Modules](modules) and their [Components](components), as well as managing message generation for [Gestures](gestures).
//! Primary execution starts at [`modman_main`]
//!

pub mod busses;
pub mod components;
pub mod gestures;
pub mod ipc;
pub mod models;
pub mod modules;

use busses::start_busses;
use models::ModManStore;
use modules::{
  deinit_module,
  init_module,
};
use std::{
  collections::HashMap,
  sync::Arc,
};
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
  warn,
};
use zenoh_ext::{
  AdvancedPublisherBuilderExt,
  CacheConfig,
};

use crate::server::modman::{
  ipc::handle_ipc,
  models::Module,
};

pub const MODULE_EVT_ID: &str = "com/reboot-codes/clover/hub/modman";

/// Begin the ModMan threads and sub-processes to ensure module/compoent communications.
#[instrument(skip(store, cancellation_tokens))]
pub async fn modman_main(
  store: ModManStore,
  cancellation_tokens: (CancellationToken, CancellationToken),
) {
  info!("Starting ModMan...");

  let mut zenoh_config = zenoh::Config::default();

  zenoh_config.insert_json5("connect/endpoints", "tcp/localhost:6699");
  zenoh_config
    .insert_json5(
      "timestamping/enabled",
      r#"{ router: true, peer: true, client: true }"#,
    )
    .unwrap();

  debug!("Connecting to Zenoh...");
  let session = Arc::new(zenoh::open(zenoh_config).await.unwrap());
  debug!("Connected to Zenoh!");

  let status_publisher = session
    .declare_publisher(format!("{MODULE_EVT_ID}/status"))
    .cache(CacheConfig::default().max_samples(1))
    .await
    .unwrap();

  let ipc_token = cancellation_tokens.0.clone();
  let ipc_session = session.clone();
  let ipc_store = store.clone();
  let ipc_handle = tokio::task::spawn(handle_ipc(ipc_store, ipc_token, ipc_session));

  let bus_store = Arc::new(store.clone());
  let bus_token = cancellation_tokens.0.clone();
  let bus_session = session.clone();
  let bus_handle = tokio::task::spawn(async move {
    tokio::select! {
      _ = bus_token.cancelled() => {
        debug!("bus_handle exited");
      },
      _ = start_busses(bus_store, bus_session) => {}
    }
  });

  let init_store = Arc::new(store.clone());
  let init_results = cancellation_tokens
    .0
    .run_until_cancelled(async move {
      let config = init_store.config.lock().await;
      let static_modules = &config.modman.static_modules;
      let static_components = &config.modman.static_components;
      let mut modules_to_init: HashMap<String, Module> = {
        let mut hashmap = HashMap::new();
        let modules = init_store.modules.lock().await;

        for (module_id, module_config) in modules.iter() {
          hashmap.insert(module_id.clone(), module_config.clone());
        }

        hashmap
      };
      let mut modules_initalized: usize = 0;

      debug!("Checking for statically defined modules to init...");
      if static_modules.len() > 0 {
        debug!(
          "Adding {} statically defined module(s) to init queue...",
          static_modules.len()
        );
        for (module_id, module) in static_modules {
          modules_to_init.insert(module_id.clone(), module.clone());
        }
      } else {
        debug!("No statically defined modules to put into store, skipping!");
      }

      debug!("Checking for statically defined components to init...");
      if static_components.len() > 0 {
        debug!(
          "Adding {} statically defined components(s) to init queue...",
          static_components.len()
        );
        let mut components = init_store.components.lock().await;

        for (component_id, component) in static_components {
          components.insert(
            component_id.clone(),
            Arc::new((component.0.clone(), component.1.clone())),
          );
        }
      } else {
        debug!("No statically defined components to put into store, skipping!");
      }

      let total_modules = modules_to_init.len();

      drop(config);

      info!("Initalizing modules...");
      if total_modules > 0 {
        // Initialize modules that were registered already via configuration and persistence.
        for (id, module) in modules_to_init.iter() {
          let (initialized, _components_initialized) =
            init_module(&init_store, id.clone(), module.clone()).await;

          if initialized {
            modules_initalized += 1;
          }
        }
      } else {
        info!("No static modules to initialize.");
      }

      (modules_initalized, total_modules)
    })
    .await;

  if let Some((modules_initalized, total_modules)) = init_results {
    if modules_initalized != total_modules {
      warn!("Initalized {modules_initalized} out of {total_modules} module(s)!");
      status_publisher
        .put("ready:incomplete")
        .await
        .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
    } else {
      if modules_initalized != 0 {
        info!("Initalized all {modules_initalized} module(s)");
      }
      status_publisher
        .put("ready")
        .await
        .unwrap_or_else(|e| error!("Failed to publish status due to:\n{e}"));
    }

    info!("ModMan Ready!");
  }

  let mod_clean_token = cancellation_tokens.0.clone();
  tokio::select! {
    _ = mod_clean_token.cancelled() => {
      bus_handle.abort();
      ipc_handle.abort();
      drop(status_publisher);

      info!("Cleaning up modules...");

      // TODO: Add override cancellation token to force stop!

      let modules_snapshot: Vec<(String, Module)> = {
        let modules = store.modules.lock().await;
        debug!("done waiting for lock");
        modules
          .iter()
          .filter(|(_, m)| m.initialized)
          .map(|(id, m)| (id.clone(), m.clone()))
          .collect()
        // lock drops here
      };

      let total = modules_snapshot.len();
      let mut modules_deinitalized: usize = 0;

      if total > 0 {
        for (id, module) in modules_snapshot {
          let (de_initialized, _) = deinit_module(&store, id, module).await;
          if de_initialized { modules_deinitalized += 1; }
        }

        if modules_deinitalized != total {
          warn!("Deinitalized {} out of {} module(s)!", modules_deinitalized, total);
        } else {
          info!("Deinitalized all {} module(s)", modules_deinitalized);
        }
      } else {
        debug!("No modules to deinit.");
      }

      std::mem::drop(store);

      cancellation_tokens.1.cancel();
    }
  }

  info!("ModMan has stopped!");
}
