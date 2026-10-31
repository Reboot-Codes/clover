use crate::{
  alerts::module::AlertComponent,
  commands::{
    models::components::ComponentTarget,
    module::CommandComponent,
  },
  module::comms::{
    CommsBus,
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

pub struct Module<
  'a,
  const MAX_COMMAND_COMPONENTS: usize = 8,
  const MAX_ALERT_COMPONENTS: usize = 8,
> {
  pub command_components: [Option<&'a mut dyn CommandComponent>; MAX_COMMAND_COMPONENTS],
  pub alert_components: [Option<&'a mut dyn AlertComponent>; MAX_ALERT_COMPONENTS],
  pub count: usize,
}

impl<'a, const MAX_COMMAND_COMPONENTS: usize, const MAX_ALERT_COMPONENTS: usize>
  Module<'a, MAX_COMMAND_COMPONENTS, MAX_ALERT_COMPONENTS>
{
  const NONE_CMD: Option<&'a mut dyn CommandComponent> = None;
  const NONE_ALERT: Option<&'a mut dyn AlertComponent> = None;

  pub const fn new() -> Self {
    Self {
      command_components: [Self::NONE_CMD; MAX_COMMAND_COMPONENTS],
      alert_components: [Self::NONE_ALERT; MAX_ALERT_COMPONENTS],
      count: 0,
    }
  }

  pub fn add_command_component(mut self, comp: &'a mut dyn CommandComponent) -> Self {
    assert!(
      self.count < MAX_COMMAND_COMPONENTS,
      "Exceeded MAX_COMMAND_COMPONENTS"
    );
    self.command_components[self.count] = Some(comp);
    self.count += 1;
    self
  }

  pub fn add_alert_component(mut self, comp: &'a mut dyn AlertComponent) -> Self {
    assert!(
      self.count < MAX_ALERT_COMPONENTS,
      "Exceeded MAX_ALERT_COMPONENTS"
    );
    self.alert_components[self.count] = Some(comp);
    self.count += 1;
    self
  }

  pub fn dispatch_target(&mut self, component_id: u8, target: ComponentTarget, transition_ms: u16) {
    for slot in self.command_components[..self.count].iter_mut().flatten() {
      if slot.id() == component_id {
        let _ = slot.on_target(target, transition_ms, None, [0u8; 16]);
        break;
      }
    }
  }

  pub fn build<Bus: CommsBus>(
    self,
    bus: Bus,
    channels: &'static ModuleChannels<8, 16>,
  ) -> (
    CommsWorker<Bus>,
    ModuleWorker<'a, MAX_COMMAND_COMPONENTS, MAX_ALERT_COMPONENTS>,
  ) {
    (
      CommsWorker {
        bus,
        rx_from_module: channels.to_comms.receiver(),
        tx_to_module: channels.to_module.sender(),
      },
      ModuleWorker {
        module: self,
        rx_from_comms: channels.to_module.receiver(),
        tx_to_comms: channels.to_comms.sender(),
      },
    )
  }
}

/// Core 0: Owns the components and physical hardware pins.
/// Runs exclusively on the Core 0 local thread.
pub struct ModuleWorker<'a, const MAX_CMDS: usize, const MAX_ALERTS: usize> {
  pub module: Module<'a, MAX_CMDS, MAX_ALERTS>,
  pub rx_from_comms: Receiver<'static, CriticalSectionRawMutex, CommsToModule, 8>,
  pub tx_to_comms: Sender<'static, CriticalSectionRawMutex, ModuleToComms, 16>,
}
