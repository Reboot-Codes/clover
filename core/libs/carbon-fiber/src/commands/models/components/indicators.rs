use crate::commands::module::{
  components::CommandComponentPipeline,
  interpolation::Interpolation,
};

pub struct IndicatorTarget {}

pub struct IndicatorComponent<I: Interpolation> {
  pub id: u8,
  pub interpolator: I,
  pub pipeline: CommandComponentPipeline,
}
