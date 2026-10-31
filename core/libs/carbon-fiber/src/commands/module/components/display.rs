use embedded_graphics::prelude::DrawTarget;

use crate::commands::{
  models::components::display::DisplayComponent,
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

impl<I: Interpolation, Dt: DrawTarget> CommandComponent for DisplayComponent<I, Dt> {
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

impl<I: Interpolation, Dt: DrawTarget> CommandComponentBuilder for DisplayComponent<I, Dt> {
  type WithInterpolator<NewI: Interpolation> = DisplayComponent<NewI, Dt>;

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    DisplayComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
      display: self.display,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    DisplayComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    DisplayComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
    }
  }

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<crate::commands::module::interpolation::CubicInterpolation> {
    DisplayComponent {
      id: self.id,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      display: self.display,
    }
  }
}

impl<Dt: DrawTarget> DisplayComponent<LinearInterpolation, Dt> {
  /// Create a new Display Component with the given ID and display driver.
  pub fn new(id: u8, display: Dt) -> Self {
    DisplayComponent {
      id,
      interpolator: LinearInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
      display,
    }
  }
}

impl<I: Interpolation, Dt: DrawTarget> DisplayComponent<I, Dt> {
  pub fn interpolate_smooth(self) -> DisplayComponent<SmoothInterpolation, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
    }
  }

  pub fn interpolate_linear(self) -> DisplayComponent<LinearInterpolation, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
    }
  }

  pub fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> DisplayComponent<CubicInterpolation, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      display: self.display,
    }
  }

  pub fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> DisplayComponent<NewI, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
      display: self.display,
    }
  }
}
