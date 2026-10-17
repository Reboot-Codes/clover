//! # Video Command Targets
//!
//! We send an inital **I**ntra-coded frame that has the full inital state of what the display should look like; then, we send a **P**precticted-picture frame that only contains updates to the I-Frame.
//!
//! After a few I-Frames, a new P-Frame is sent to keep the display in sync if there were any drops.
//!

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VideoIFrameTarget {}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VideoPFrameTarget {}
