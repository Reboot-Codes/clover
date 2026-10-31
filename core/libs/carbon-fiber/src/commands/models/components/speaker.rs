use crate::commands::module::{
  components::CommandComponentPipeline,
  interpolation::Interpolation,
};

/// Play audio embeded in the module firmware/fs.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioCueTarget {}

/// Raw audio chunk from the comms bus.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioChunkTarget {}

/// Component to output audio anywhere.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SpeakerComponent<I: Interpolation> {
  pub id: u8,
  pub interpolator: I,
  pub pipeline: CommandComponentPipeline,
  pub ohms: Option<u16>,
}
