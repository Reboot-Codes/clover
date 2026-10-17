use crate::commands::{
  models::components::movement::{
    MovementComponent,
    TentacleConfig,
  },
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
      tentacle_cfg: None,
      kinematic_parent: None,
      interpolator: SmoothInterpolation {},
      pipeline: CommandComponentPipeline::new(),
    }
  }
}

impl<P1, P2, P3, P4, P5, P6, I: Interpolation> MovementComponent<P1, P2, P3, P4, P5, P6, I> {
  pub fn tentacle(mut self, config: TentacleConfig) -> Self {
    self.tentacle_cfg = Some(config);
    self
  }

  pub fn degree1_servo<NewP1>(self, pin: NewP1) -> MovementComponent<NewP1, P2, P3, P4, P5, P6, I> {
    MovementComponent {
      id: self.id,
      actuator1: Some(pin),
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      tentacle_cfg: self.tentacle_cfg,
      kinematic_parent: self.kinematic_parent,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
    }
  }

  pub fn degree2_servo(mut self, pin: P2) -> Self {
    self.actuator2 = Some(pin);
    self
  }

  pub fn degree3_servo(mut self, pin: P3) -> Self {
    self.actuator3 = Some(pin);
    self
  }
  pub fn degree4_servo(mut self, pin: P4) -> Self {
    self.actuator4 = Some(pin);
    self
  }
  pub fn degree5_servo(mut self, pin: P5) -> Self {
    self.actuator5 = Some(pin);
    self
  }
  pub fn degree6_servo(mut self, pin: P6) -> Self {
    self.actuator6 = Some(pin);
    self
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
      tentacle_cfg: self.tentacle_cfg,
      kinematic_parent: self.kinematic_parent,
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
      tentacle_cfg: self.tentacle_cfg,
      kinematic_parent: self.kinematic_parent,
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
      tentacle_cfg: self.tentacle_cfg,
      kinematic_parent: self.kinematic_parent,
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
      tentacle_cfg: self.tentacle_cfg,
      kinematic_parent: self.kinematic_parent,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
    }
  }
}
