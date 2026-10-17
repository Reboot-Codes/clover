use crate::commands::module::{
  components::ComponentPipeline,
  interpolation::Interpolation,
};

pub struct TentacleConfig {}

pub enum MovementTarget {
  OneDof(u32),
  TwoDof((u32, u32)),
}

pub struct MovementComponent<P1, P2, P3, P4, P5, P6, I>
where
  I: Interpolation,
{
  pub id: u8,
  pub actuator1: Option<P1>,
  pub actuator2: Option<P2>,
  pub actuator3: Option<P3>,
  pub actuator4: Option<P4>,
  pub actuator5: Option<P5>,
  pub actuator6: Option<P6>,
  pub tentacle_cfg: Option<TentacleConfig>,
  pub kinematic_parent: Option<u8>,
  pub interpolator: I,
  pub pipeline: ComponentPipeline,
}
