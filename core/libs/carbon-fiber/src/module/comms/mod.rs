use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{
  Channel,
  Receiver,
  Sender,
};

use crate::alerts::models::components::AlertMessage;
use crate::commands::models::components::{
  ComponentTarget,
  ComponentTelemetry,
  GestureFlags,
};

pub mod can2;

pub trait CommsBus {}

/// Core 1: Owns ONLY Comms/CAN and IPC queues.
/// Has zero access to physical components.
pub struct CommsWorker<Bus> {
  pub bus: Bus,
  pub rx_from_module: Receiver<'static, CriticalSectionRawMutex, ModuleToComms, 16>,
  pub tx_to_module: Sender<'static, CriticalSectionRawMutex, CommsToModule, 8>,
}

/// All message types from the Comms thread to the Module thread.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommsToModule {
  DispatchTarget {
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<GestureFlags>,
    tx_id: [u8; 16],
  },
  RunSelfTest,
  EmergencyStop,
}

/// All message types from the Module thread to the Comms thread.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModuleToComms {
  SelfTestResult {
    success: bool,
    failed_component_id: Option<u8>,
    error_code: u8,
  },
  Telemetry {
    tx_id: [u8; 16],
    feedback: ComponentTelemetry,
  },
  Alert(AlertMessage),
}

pub type CommsQueue<const N: usize> = Channel<CriticalSectionRawMutex, CommsToModule, N>;
pub type ModuleQueue<const N: usize> = Channel<CriticalSectionRawMutex, ModuleToComms, N>;

pub struct ModuleChannels<const MOD_SZ: usize = 8, const COM_SZ: usize = 16> {
  pub to_module: CommsQueue<MOD_SZ>,
  pub to_comms: ModuleQueue<COM_SZ>,
}

impl<const MOD_SZ: usize, const COM_SZ: usize> ModuleChannels<MOD_SZ, COM_SZ> {
  pub const fn new() -> Self {
    Self {
      to_module: Channel::new(),
      to_comms: Channel::new(),
    }
  }
}
