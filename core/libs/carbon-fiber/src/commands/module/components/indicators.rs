use crate::{
  commands::{
    models::components::indicators::IndicatorComponent,
    module::{
      CommandComponent,
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

impl IndicatorComponent<LinearInterpolation> {
  pub fn new(id: u8) -> Self {
    IndicatorComponent {
      id: id,
      interpolator: LinearInterpolation::new(),
      pipeline: ComponentPipeline::new(),
    }
  }
}

impl<I: Interpolation> IndicatorComponent<I> {
  pub fn neopixels_pin(self) -> Self {
    todo!()
  }
}

impl<I: Interpolation> Component for IndicatorComponent<I> {
  fn new<C: Component>(id: u8) -> C {
    todo!()
  }

  fn id(&self) -> u8 {
    todo!()
  }

  fn type_of() -> ComponentTypes {
    ComponentTypes::Command
  }
}

impl<I: Interpolation> CommandComponent for IndicatorComponent<I> {
  fn on_target(
    self,
    target: crate::commands::models::components::ComponentTarget,
    transition_ms: u16,
    flags: Option<crate::commands::models::components::GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()> {
    todo!()
  }

  fn step(
    self,
    dt_ms: u32,
  ) -> Option<(
    [u8; 16],
    crate::commands::models::components::ComponentTelemetry,
  )> {
    todo!()
  }

  fn backtrack(self, return_ms: u16) {
    todo!()
  }

  type WithInterpolator<NewI: Interpolation> = IndicatorComponent<NewI>;

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    IndicatorComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
    }
  }
}
