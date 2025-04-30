pub mod cameras;
pub mod displays;
use serde::{
  Deserialize,
  Serialize,
};
use std::num::NonZero;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResolution {
  pub height: NonZero<u64>,
  pub width: NonZero<u64>,
}
