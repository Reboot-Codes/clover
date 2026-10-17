//! # Module-Specific Adoption Code
//!
//! Modules have a unique responsibility to produce a pairing code; and unique,
//! and cryptographically secure salt value to exchange encryption keys with the Hub.
//!
//! Internally, a [ModuleAdoptionContext] is created to keep track of the process, and
//! to provide standardized methods to produce the required data to be serialized and
//! sent over the wire.
//!
//! Code here is generally meant to serve as convinience logic, and does not contain
//! vendor-specific code, though, it will call upon it for crypto, obviously.
//!

use crate::{
  adoption::models::{
    AbortionReason,
    ConfirmMessage,
    HelloMessage,
    KeyHIMOMessage,
    KeyHOMIMessage,
    ProbeMessage,
  },
  alerts::module::ModuleAlertContext,
  commands::module::ModuleCommandContext,
};

/// Convinience struct for modules waiting to be adopted.
/// It generally calls on utility functions to do cryptographic
/// maths, and as such, requires the vendor struct.
pub struct ModuleAdoptionContext<'a> {
  pub module_id: &'a str,
  pub module_rfqdn: &'a str,
  pub pin: [u8; 6],
}

impl<'a> ModuleAdoptionContext<'a> {
  pub fn new() -> Self {
    todo!();
  }

  pub fn parse_probe(msg: &ProbeMessage) -> Result<(), AbortionReason> {
    todo!();
  }

  pub fn gen_hello() -> HelloMessage<'a> {
    todo!();
  }

  pub fn parse_key_homi(msg: &KeyHOMIMessage) -> Result<(), AbortionReason> {
    todo!();
  }

  pub fn gen_key_himo() -> KeyHIMOMessage<'a> {
    todo!();
  }

  pub fn parse_confirm(msg: &ConfirmMessage) -> Result<(), AbortionReason> {
    todo!();
  }

  pub fn gen_command_context<const NUM_COMPONENTS: usize>()
  -> Result<(), (ModuleAlertContext, ModuleCommandContext<NUM_COMPONENTS>)> {
    todo!();
  }
}
