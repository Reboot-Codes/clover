//! # Video Command Targets
//!
//! We send an inital **I**ntra-coded frame that has the full inital state of what the display should look like; then, we send a **P**recticted-picture frame that only contains updates to the I-Frame.
//!
//! After a few I-Frames, a new P-Frame is sent to keep the display in sync if there were any drops.
//!

use embedded_graphics::prelude::DrawTarget;

use crate::commands::module::{
  components::CommandComponentPipeline,
  interpolation::Interpolation,
};

/// Intra-coded frame, contains full frame data.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VideoIFrameTarget {}

/// Prediction frame, contains delta from last frame.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VideoPFrameTarget {}

/// The Command Component which handles Displays.
///
/// This Component is widely based on the embedded_graphics library
/// which enables us to support any display which has a driver that exposes
/// a struct compatible with [embedded_graphics::draw_target::DrawTarget].
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DisplayComponent<I: Interpolation, Dt: DrawTarget> {
  pub id: u8,
  pub interpolator: I,
  /// The pipeline for this component merges any P-frames
  /// into an I-frame to be placed into `.prev_target`.
  pub pipeline: CommandComponentPipeline,
  pub display: Dt,
  pub dirty: bool,
}
