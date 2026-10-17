pub mod interpolation;

use crate::commands::{
  models::components::{
    ComponentFaliure,
    ComponentTarget,
    ComponentTelemetry,
    GestureFlags,
    GestureTargetCommand,
  },
  module::{
    components::CommandComponentPipeline,
    interpolation::{
      CubicInterpolation,
      Interpolation,
      LinearInterpolation,
      SmoothInterpolation,
    },
  },
};

pub mod components;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ModuleCommandContext<const NUM_COMPONENTS: usize> {
  pub components: [CommandComponentPipeline; NUM_COMPONENTS],
}

impl<const NUM_COMPONENTS: usize> ModuleCommandContext<NUM_COMPONENTS> {
  pub fn new() -> Self {
    Self {
      components: [CommandComponentPipeline::new(); NUM_COMPONENTS],
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

  pub fn get_pipeline(&self, component_id: u8) -> Option<&CommandComponentPipeline> {
    self.components.get(component_id as usize)
  }

  pub fn get_pipeline_mut(&mut self, component_id: u8) -> Option<&mut CommandComponentPipeline> {
    self.components.get_mut(component_id as usize)
  }
}

/// `dyn`-compatible trait that all Command Components must implement.
///
/// This trait defines methods called during normal module operation.
pub trait CommandComponent {
  /// Get the ID of the component
  fn id(&self) -> u8;

  /// Called by CarbonFiber when a new target arrives from the comms thread.
  fn on_target(
    &mut self,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()>;

  /// Called in regular intervals by the runtime loop.
  fn step(&mut self, dt_ms: u32) -> Option<([u8; 16], ComponentTelemetry)>;

  /// Backtrack to previous pose
  fn backtrack(&mut self, return_ms: u16);
}

/// Convinience builder trait. Generally, Command Components should
/// implement this trait for developer experience consistency, but
/// it's not strictly needed for compilation type checks at all.
///
/// This trait implements the builder syntax, but does not have a
/// `.build()` method, instead Components should implement a `.new()`
/// themselves.
pub trait CommandComponentBuilder {
  /// The resulting component type when the interpolator is changed
  type WithInterpolator<NewI: Interpolation>: CommandComponentBuilder;

  /// Interpolate with this specific struct.
  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI>;

  /// Linearly interpolate to targets.
  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation>;

  /// Interpolate to targets with the default cubic smoothing curve meant for physical movements.
  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation>;

  /// Interpolate to targets with a custom cubic smoothing curve.
  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<CubicInterpolation>;
}
