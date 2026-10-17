pub mod interpolation;

use crate::{
  commands::{
    models::components::{
      ComponentFaliure,
      ComponentTarget,
      ComponentTelemetry,
      GestureFlags,
      GestureTargetCommand,
    },
    module::{
      components::ComponentPipeline,
      interpolation::{
        Interpolation,
        LinearInterpolation,
        SmoothInterpolation,
      },
    },
  },
  module::component::{
    Component,
    ComponentTypes,
  },
};

pub mod components;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModuleCommandContext<const NUM_COMPONENTS: usize> {
  pub components: [ComponentPipeline; NUM_COMPONENTS],
}

impl<const NUM_COMPONENTS: usize> ModuleCommandContext<NUM_COMPONENTS> {
  pub fn new() -> Self {
    Self {
      components: [ComponentPipeline::new(); NUM_COMPONENTS],
    }
  }

  pub fn enqueue(
    &mut self,
    cmd: GestureTargetCommand,
    tx_id: [u8; 16],
  ) -> Result<bool, ComponentFaliure> {
    let idx = cmd.component_id as usize;
    if idx < NUM_COMPONENTS {
      self.components[idx].push(cmd, tx_id)
    } else {
      Err(ComponentFaliure::InvalidComponent)
    }
  }

  pub fn trigger_backtrack(&mut self, component_id: u8, return_ms: u16) -> Option<[u8; 16]> {
    let idx = component_id as usize;
    if idx < NUM_COMPONENTS {
      self.components[idx].backtrack(return_ms)
    } else {
      None
    }
  }

  pub fn get_pipeline(&self, component_id: u8) -> Option<&ComponentPipeline> {
    self.components.get(component_id as usize)
  }

  pub fn get_pipeline_mut(&mut self, component_id: u8) -> Option<&mut ComponentPipeline> {
    self.components.get_mut(component_id as usize)
  }
}

pub trait CommandComponent {
  /// Called by CarbonFiber when a new target arrives from the comms thread.
  fn on_target(
    self,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()>;

  /// Called in regular intervals by the runtime loop.
  fn step(self, dt_ms: u32) -> Option<([u8; 16], ComponentTelemetry)>;

  /// Backtrack to previous pose
  fn backtrack(self, return_ms: u16);

  // ---

  /// The resulting component type when the interpolator is changed
  type WithInterpolator<NewI: Interpolation>: CommandComponent + Component;

  /// Interpolate with this specific struct.
  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI>;

  /// Linearly interpolate to targets.
  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation>;

  /// Interpolate to targets with the default cubic smoothing curve meant for physical movements.
  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation>;

  fn type_of() -> ComponentTypes {
    ComponentTypes::Command
  }
}
