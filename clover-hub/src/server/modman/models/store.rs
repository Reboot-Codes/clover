use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::server::modman::models::components::{
  CloverComponent,
  CloverComponentMeta,
};
use crate::server::modman::models::gestures::GestureStates;
use crate::server::modman::models::modules::Module;
use crate::server::modman::models::PortStatus;
use crate::server::warehouse::config::models::Config;

/// Used for [Bus](super::busses::models::Bus) statuses, etc
#[derive(Debug, Clone)]
pub struct PortStatuses {
  /// Used by the [UART Bus](super::busses::proxies::uart::UARTBus).
  pub uart: Arc<Mutex<HashMap<String, PortStatus>>>,
  /// Used by the [CAN2 Bus](super::busses::proxies::can_2::CAN2Bus).
  ///
  /// Identifier string format is `$IFACE/$RX:$TX` where (e.g. `can0/0x101:0x201`):
  /// - `$IFACE` is the network interface for the needed CAN bus
  /// - `$RX` is the ***HEX*** representation of the CAN ID used for recieving messages from the module on. (Linux kernel will automatically filter replies for us.)
  /// - `$TX` is the ***HEX*** representation of the CAN ID of the module that we're trying to communicate with.
  ///
  /// Matches regex: `r"^(?<iface>(?:\\w|[0-9])+)\\/(?<rx_id>0[xX][0-9a-fA-F]{3}):(?<tx_id>0[xX][0-9a-fA-F]{3})$"`
  pub can_2: Arc<Mutex<HashMap<String, PortStatus>>>,
}

/// In memory data-store for components, modules, and any needed configuration.
#[derive(Debug, Clone)]
pub struct ModManStore {
  /// All currently known modules and their states.
  pub modules: Arc<Mutex<HashMap<String, Module>>>,
  /// All currently known components and their states.
  pub components: Arc<Mutex<HashMap<String, Arc<(CloverComponentMeta, CloverComponent)>>>>,
  /// Global access to the current configuration.
  pub config: Arc<Mutex<Config>>,
  pub gesture_states: Arc<Mutex<HashMap<String, GestureStates>>>,
  pub foreground_gesture_priority: Arc<Mutex<Vec<String>>>,
  pub background_gesture_priority: Arc<Mutex<Vec<String>>>,
  /// Used for [Bus](super::busses::models::Bus) statuses, etc
  pub port_statuses: PortStatuses,
}

impl ModManStore {
  pub fn new(optional_config: Option<Arc<Mutex<Config>>>) -> Self {
    let config = match optional_config {
      Some(cfg) => cfg,
      Option::None => Arc::new(Mutex::new(Config::default())),
    };

    ModManStore {
      modules: Arc::new(Mutex::new(HashMap::new())),
      components: Arc::new(Mutex::new(HashMap::new())),
      gesture_states: Arc::new(Mutex::new(HashMap::new())),
      foreground_gesture_priority: Arc::new(Mutex::new(Vec::new())),
      background_gesture_priority: Arc::new(Mutex::new(Vec::new())),
      port_statuses: PortStatuses {
        uart: Arc::new(Mutex::new(HashMap::new())),
        can_2: Arc::new(Mutex::new(HashMap::new())),
      },
      config,
    }
  }
}
