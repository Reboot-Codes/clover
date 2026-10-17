use crate::commands::{
  models::components::movement::{
    MovementComponent,
    MovementTarget,
    TentacleConfig,
  },
  module::{
    components::ComponentPipeline,
    interpolation::{
      CustomInterpolation,
      Interpolation,
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
      pipeline: ComponentPipeline::new(),
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

  pub fn kinematic_off_of(mut self, parent_id: u8) -> Self {
    self.kinematic_parent = Some(parent_id);
    self
  }

  pub fn interpolate_smooth(
    self,
  ) -> MovementComponent<P1, P2, P3, P4, P5, P6, SmoothInterpolation> {
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
      interpolator: SmoothInterpolation {},
      pipeline: self.pipeline,
    }
  }

  pub fn interpolate_custom<F>(
    self,
    f: F,
  ) -> MovementComponent<P1, P2, P3, P4, P5, P6, CustomInterpolation<F>>
  where
    F: Fn(MovementTarget, MovementTarget, f32) -> MovementTarget,
  {
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
      interpolator: CustomInterpolation(f),
      pipeline: self.pipeline,
    }
  }
}
