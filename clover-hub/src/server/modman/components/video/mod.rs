//! # Video I/O Components
//!
//! Video Components are comprised of either a [Camera](cameras), or a [Display](displays). Both require a host board with hardware accelerated en-/decoding to keep everything at a usable framerate. To make use of hardware accelerated inference, you'll need to define an [App Module](crate::server::modman::modules#configuration), and then make use of [the Inference Engine](crate::server::inference_engine) in your app via Tesseract.
//!

pub mod cameras;
pub mod displays;
use serde::{
  Deserialize,
  Serialize,
};
use std::num::NonZero;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoResolution {
  pub height: NonZero<u32>,
  pub width: NonZero<u32>,
}
