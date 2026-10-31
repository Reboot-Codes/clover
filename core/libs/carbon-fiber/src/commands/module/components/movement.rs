use crate::commands::{
  models::components::movement::MovementComponent,
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

impl MovementComponent<(), (), (), (), (), (), SmoothInterpolation> {
  pub const fn new(id: u8) -> Self {
    Self {
      id,
      actuator1: None,
      actuator2: None,
      actuator3: None,
      actuator4: None,
      actuator5: None,
      actuator6: None,
      interpolator: SmoothInterpolation {},
      pipeline: CommandComponentPipeline::new(),
    }
  }
}

impl<A1, A2, A3, A4, A5, A6, I: Interpolation> MovementComponent<A1, A2, A3, A4, A5, A6, I> {
  /// Set the Actuator for the 1st degree of freedom (x).
  pub fn set_degree1<NewP1>(self, pin: NewP1) -> MovementComponent<NewP1, A2, A3, A4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: Some(pin),
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }

  /// Set the Actuator for the 2nd degree of freedom (y).
  pub fn set_degree2<NewP2>(self, pin: NewP2) -> MovementComponent<A1, NewP2, A3, A4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: Some(pin),
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }

  /// Set the Actuator for the 3rd degree of freedom (z).
  pub fn set_degree3<NewP3>(self, pin: NewP3) -> MovementComponent<A1, A2, NewP3, A4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: Some(pin),
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }

  /// Set the Actuator for the 4th degree of freedom (u).
  pub fn set_degree4<NewP4>(self, pin: NewP4) -> MovementComponent<A1, A2, A3, NewP4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: Some(pin),
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }

  /// Set the Actuator for the 5th degree of freedom (v).
  pub fn set_degree5<NewP5>(self, pin: NewP5) -> MovementComponent<A1, A2, A3, A4, NewP5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: Some(pin),
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }

  /// Set the Actuator for the 6th degree of freedom (w).
  pub fn set_degree6<NewP6>(self, pin: NewP6) -> MovementComponent<A1, A2, A3, A4, A5, NewP6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: Some(pin),
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }
}

impl<P1, P2, P3, P4, P5, P6, I: Interpolation> CommandComponent
  for MovementComponent<P1, P2, P3, P4, P5, P6, I>
{
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

impl<P1, P2, P3, P4, P5, P6, I: Interpolation> CommandComponentBuilder
  for MovementComponent<P1, P2, P3, P4, P5, P6, I>
{
  type WithInterpolator<NewI: Interpolation> = MovementComponent<P1, P2, P3, P4, P5, P6, NewI>;

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
    }
  }

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator,
      pipeline: self.pipeline,
    }
  }

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<CubicInterpolation> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
    }
  }
}
