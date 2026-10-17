use crate::{
  alerts::module::AlertComponent,
  commands::{
    models::components::{
      ComponentTarget,
      GestureFlags,
    },
    module::CommandComponent,
  },
};

pub trait ComponentGroup {
  fn dispatch_target(
    &mut self,
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()>;

  fn step_all(&mut self, dt_ms: u32);
}

// Blanket implementation for any 3-tuple of components:
impl<C0, C1, C2> ComponentGroup for (C0, C1, C2)
where
  C0: Component,
  C1: Component,
  C2: Component,
{
  fn dispatch_target(
    &mut self,
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()> {
    if self.0.id() == component_id {
      self.0.on_target(target, transition_ms, flags, tx_id)
    } else if self.1.id() == component_id {
      self.1.on_target(target, transition_ms, flags, tx_id)
    } else if self.2.id() == component_id {
      self.2.on_target(target, transition_ms, flags, tx_id)
    } else {
      Err(())
    }
  }

  fn step_all(&mut self, dt_ms: u32) {
    self.0.step(dt_ms);
    self.1.step(dt_ms);
    self.2.step(dt_ms);
  }
}

pub enum ComponentTypes {
  Command,
  Alert,
}

pub trait Component {
  /// Create a new instance of this component with default values.
  fn new<C: Component>(id: u8) -> C;

  /// ID of the module
  fn id(&self) -> u8;

  fn type_of() -> ComponentTypes;
}
