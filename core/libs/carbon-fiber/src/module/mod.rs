use crate::{
  alerts::module::AlertComponent,
  commands::{
    models::components::ComponentTarget,
    module::CommandComponent,
  },
  module::comms::{
    CommsToModule,
    CommsWorker,
    ModuleChannels,
    ModuleToComms,
  },
};

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{
  Receiver,
  Sender,
};

pub mod comms;
pub mod hardware;
pub mod rand;

use snafu::Snafu;

#[derive(Debug, Snafu, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DispatchError {
  #[snafu(display("Component with id {component_id} not found."))]
  ComponentNotFound { component_id: u8 },

  #[snafu(display("Target variant mismatched for component {component_id}."))]
  MismatchedTarget { component_id: u8 },

  #[snafu(display("Pipeline buffer is full for component {component_id}."))]
  PipelineFull { component_id: u8 },
}

pub const MAX_COMPONENTS: usize = 256;

pub struct Module<'a> {
  pub module_id: &'static str,
  pub module_rfqdn: &'static str,
  pub command_components: [Option<&'a mut dyn CommandComponent>; MAX_COMPONENTS],
  pub alert_components: [Option<&'a mut dyn AlertComponent>; MAX_COMPONENTS],
  pub count: usize,
}

impl<'a> Module<'a> {
  const NONE_CMD: Option<&'a mut dyn CommandComponent> = None;
  const NONE_ALERT: Option<&'a mut dyn AlertComponent> = None;

  pub const fn new(module_id: &'static str, module_rfqdn: &'static str) -> Self {
    Self {
      module_id,
      module_rfqdn,
      command_components: [Self::NONE_CMD; MAX_COMPONENTS],
      alert_components: [Self::NONE_ALERT; MAX_COMPONENTS],
      count: 0,
    }
  }

  pub fn add_command_component(mut self, comp: &'a mut dyn CommandComponent) -> Self {
    let id = comp.id() as usize;
    self.command_components[id] = Some(comp);
    self.count += 1;
    self
  }

  pub fn add_alert_component(mut self, comp: &'a mut dyn AlertComponent) -> Self {
    let id = comp.id() as usize;
    self.alert_components[id] = Some(comp);
    self.count += 1;
    self
  }

  pub fn dispatch_command(
    &mut self,
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<crate::commands::models::components::GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), DispatchError> {
    if let Some(comp) = &mut self.command_components[component_id as usize] {
      comp
        .on_target(target, transition_ms, flags, tx_id)
        .map_err(|()| DispatchError::MismatchedTarget { component_id })
    } else {
      Err(DispatchError::ComponentNotFound { component_id })
    }
  }

  pub fn dispatch_target(
    &mut self,
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
  ) {
    let _ = self.dispatch_command(component_id, target, transition_ms, None, [0u8; 16]);
  }

  pub fn poll_alerts(
    &mut self,
    dt_ms: u32,
    tx_to_comms: &Sender<'static, CriticalSectionRawMutex, ModuleToComms, 16>,
  ) {
    for slot in self.alert_components.iter_mut().flatten() {
      if let Some(msg) = slot.step(dt_ms) {
        let _ = tx_to_comms.try_send(ModuleToComms::Alert(msg));
      }
    }
  }

  pub fn build<Bus>(
    self,
    bus: Bus,
  ) -> (
    CommsWorker<'static, Bus>,
    ModuleWorker<'a>,
  ) {
    self.build_with_channels(bus, &DEFAULT_CHANNELS)
  }

  pub fn build_with_channels<Bus>(
    self,
    bus: Bus,
    channels: &'static ModuleChannels<8, 16>,
  ) -> (
    CommsWorker<'static, Bus>,
    ModuleWorker<'a>,
  ) {
    let adoption = crate::adoption::module::ModuleAdoptionContext::new(
      self.module_id,
      self.module_rfqdn,
    );
    (
      CommsWorker::new(
        bus,
        adoption,
        channels.to_comms.receiver(),
        channels.to_module.sender(),
      ),
      ModuleWorker {
        module: self,
        rx_from_comms: channels.to_module.receiver(),
        tx_to_comms: channels.to_comms.sender(),
      },
    )
  }
}

pub static DEFAULT_CHANNELS: ModuleChannels<8, 16> = ModuleChannels::new();

/// Core 0: Owns the components and physical hardware pins.
/// Runs exclusively on the Core 0 local thread.
pub struct ModuleWorker<'a> {
  pub module: Module<'a>,
  pub rx_from_comms: Receiver<'static, CriticalSectionRawMutex, CommsToModule, 8>,
  pub tx_to_comms: Sender<'static, CriticalSectionRawMutex, ModuleToComms, 16>,
}

impl<'a> ModuleWorker<'a> {
  pub fn poll_alerts(&mut self, dt_ms: u32) {
    self.module.poll_alerts(dt_ms, &self.tx_to_comms);
  }

  pub fn dispatch_command(
    &mut self,
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<crate::commands::models::components::GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), DispatchError> {
    self
      .module
      .dispatch_command(component_id, target, transition_ms, flags, tx_id)
  }

  pub fn step(&mut self, dt_ms: u32) {
    // 1. Ingest commands from CommsWorker
    while let Ok(msg) = self.rx_from_comms.try_receive() {
      match msg {
        CommsToModule::DispatchTarget {
          component_id,
          target,
          transition_ms,
          flags,
          tx_id,
        } => {
          let _ = self.dispatch_command(component_id, target, transition_ms, flags, tx_id);
        }
        CommsToModule::EmergencyStop => {
          for slot in self.module.command_components.iter_mut().flatten() {
            slot.backtrack(50);
          }
        }
        CommsToModule::RunSelfTest => {
          let mut success = true;
          let mut failed_id = None;
          for slot in self.module.command_components.iter_mut().flatten() {
            if slot.self_test().is_err() {
              success = false;
              failed_id = Some(slot.id());
              break;
            }
          }
          let _ = self.tx_to_comms.try_send(ModuleToComms::SelfTestResult {
            success,
            failed_component_id: failed_id,
            error_code: if success { 0 } else { 1 },
          });
        }
      }
    }

    // 2. Step command components & emit feedback
    for slot in self.module.command_components.iter_mut().flatten() {
      if let Some((tx_id, feedback)) = slot.step(dt_ms) {
        let _ = self.tx_to_comms.try_send(ModuleToComms::Telemetry { tx_id, feedback });
      }
    }

    // 3. Poll alert components
    self.poll_alerts(dt_ms);
  }

  #[cfg(feature = "embassy-time")]
  pub async fn run(&mut self) -> ! {
    loop {
      self.step(10);
      embassy_time::Timer::after_millis(10).await;
    }
  }
}
