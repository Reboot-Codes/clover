use crate::commands::{
  models::components::indicators::IndicatorComponent,
  module::{
    CommandComponent,
    CommandComponentBuilder,
    components::CommandComponentPipeline,
    interpolation::{
      CubicInterpolation,
      Interpolation,
      LinearInterpolation,
      SmoothInterpolation,
    },
  },
};

impl IndicatorComponent<LinearInterpolation> {
  pub fn new(id: u8) -> Self {
    IndicatorComponent {
      id: id,
      interpolator: LinearInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
    }
  }
}

impl<I: Interpolation> CommandComponent for IndicatorComponent<I> {
  fn id(&self) -> u8 {
    self.id
  }

  fn on_target(
    &mut self,
    target: crate::commands::models::components::ComponentTarget,
    transition_ms: u16,
    flags: Option<crate::commands::models::components::GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()> {
    todo!()
  }

  fn step(
    &mut self,
    dt_ms: u32,
  ) -> Option<(
    [u8; 16],
    crate::commands::models::components::ComponentTelemetry,
  )> {
    todo!()
  }

  fn backtrack(&mut self, return_ms: u16) {
    self.pipeline.backtrack(return_ms);
  }
}

impl<I: Interpolation> CommandComponentBuilder for IndicatorComponent<I> {
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

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<crate::commands::module::interpolation::CubicInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
    }
  }
}

impl<I: Interpolation> IndicatorComponent<I> {
  pub fn neopixels_pin(self) -> Self {
    todo!()
  }
}
