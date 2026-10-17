use crate::{
  alerts::module::AlertComponent,
  commands::{
    models::components::ComponentTarget,
    module::CommandComponent,
  },
};

pub mod comms;
pub mod hardware;
pub mod rand;

pub struct Module<
  'a,
  B,
  const MAX_COMMAND_COMPONENTS: usize = 8,
  const MAX_ALERT_COMPONENTS: usize = 8,
> {
  bus: B,
  command_components: [Option<&'a mut dyn CommandComponent>; MAX_COMMAND_COMPONENTS],
  alert_components: [Option<&'a mut dyn AlertComponent>; MAX_ALERT_COMPONENTS],
  count: usize,
}

impl<'a, B, const MAX_COMMAND_COMPONENTS: usize, const MAX_ALERT_COMPONENTS: usize>
  Module<'a, B, MAX_COMMAND_COMPONENTS, MAX_ALERT_COMPONENTS>
{
  const NONE_CMD: Option<&'a mut dyn CommandComponent> = None;
  const NONE_ALERT: Option<&'a mut dyn AlertComponent> = None;

  pub const fn new(bus: B) -> Self {
    Self {
      bus,
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
}
