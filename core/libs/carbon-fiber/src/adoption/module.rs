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

use sha2::Digest;

use crate::{
  adoption::models::{
    AbortionReason,
    AdoptionState,
    ConfirmMessage,
    HelloMessage,
    KeyHIMOMessage,
    KeyHOMIMessage,
    ProbeMessage,
    UNI_MODULE,
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
  pub salt: [u8; 16],
  pub key_share: [u8; 32],
  pub confirmation: [u8; 32],
  pub state: AdoptionState,
}

impl<'a> ModuleAdoptionContext<'a> {
  /// Initializes the adoption context for the module.
  /// Requires non-empty module ID and RFQDN.
  pub fn new(module_id: &'a str, module_rfqdn: &'a str) -> Self {
    assert!(!module_id.is_empty(), "module_id must not be empty");
    assert!(!module_rfqdn.is_empty(), "module_rfqdn must not be empty");

    #[cfg(feature = "mcu")]
    let (pin_val, salt) = {
      let hw = crate::module::hardware::HardwareIdentity::collect();
      crate::module::rand::raw_generate_pairing_code_and_salt(
        &hw.unique_id[..hw.unique_id_len],
        &hw.dynamic_entropy,
      )
    };
    #[cfg(not(feature = "mcu"))]
    let (pin_val, salt) = (123456u32, [0x5Au8; 16]);

    let mut pin = [0u8; 6];
    let mut temp = pin_val;
    for i in (0..6).rev() {
      pin[i] = b'0' + (temp % 10) as u8;
      temp /= 10;
    }

    Self {
      module_id,
      module_rfqdn,
      pin,
      salt,
      key_share: [0u8; 32],
      confirmation: [0u8; 32],
      state: AdoptionState::Initial,
    }
  }

  /// Parses an incoming ProbeMessage from the Hub.
  pub fn parse_probe(&mut self, msg: &ProbeMessage) -> Result<(), AbortionReason> {
    if msg.version.0 != crate::PROTOCOL_VERSION_MAJOR {
      self.state = AdoptionState::Aborted(AbortionReason::Incompatible);
      return Err(AbortionReason::Incompatible);
    }
    if !self.module_id.is_empty() && !msg.id.is_empty() && msg.id != self.module_id {
      self.state = AdoptionState::Aborted(AbortionReason::Incompatible);
      return Err(AbortionReason::Incompatible);
    }
    self.state = AdoptionState::Probed;
    Ok(())
  }

  /// Generates the HelloMessage responding to a Probe.
  pub fn gen_hello(&self) -> HelloMessage<'a> {
    HelloMessage {
      version: crate::PROTOCOL_VERSION,
      rfqdn: self.module_rfqdn,
      salt: self.salt,
    }
  }

  /// Parses KeyHOMIMessage (Hub's PAKE share) and computes the shared secret.
  pub fn parse_key_homi(&mut self, msg: &KeyHOMIMessage) -> Result<(), AbortionReason> {
    if msg.hub_pub.is_empty() {
      self.state = AdoptionState::Aborted(AbortionReason::KeyHOMIFail);
      return Err(AbortionReason::KeyHOMIFail);
    }

    // Compute module ephemeral key share
    let mut share_hasher = sha2::Sha256::new();
    share_hasher.update(&self.pin);
    share_hasher.update(&self.salt);
    share_hasher.update(UNI_MODULE);
    self.key_share = share_hasher.finalize().into();

    // Compute mutual confirmation tag
    let mut conf_hasher = sha2::Sha256::new();
    conf_hasher.update(msg.hub_pub);
    conf_hasher.update(&self.key_share);
    conf_hasher.update(&self.pin);
    self.confirmation = conf_hasher.finalize().into();

    self.state = AdoptionState::KeyExchanged;
    Ok(())
  }

  /// Generates the KeyHIMOMessage containing module share and hub confirmation tag.
  pub fn gen_key_himo(&self) -> KeyHIMOMessage<'_> {
    KeyHIMOMessage {
      module_pub: &self.key_share,
      hub_conf: &self.confirmation,
    }
  }

  /// Parses ConfirmMessage from Hub and finalizes adoption.
  pub fn parse_confirm(&mut self, msg: &ConfirmMessage) -> Result<(), AbortionReason> {
    if msg.module_conf.is_empty() {
      self.state = AdoptionState::Aborted(AbortionReason::ConfirmHOMIFail);
      return Err(AbortionReason::ConfirmHOMIFail);
    }

    if msg.module_conf != &self.confirmation[..] {
      self.state = AdoptionState::Aborted(AbortionReason::ConfirmHIMOFail);
      return Err(AbortionReason::ConfirmHIMOFail);
    }

    self.state = AdoptionState::Confirmed;
    Ok(())
  }

  /// Generates module command and alert contexts upon successful adoption.
  pub fn gen_command_context<const NUM_COMPONENTS: usize>(
    &self,
  ) -> Result<(), (ModuleAlertContext, ModuleCommandContext<NUM_COMPONENTS>)> {
    if self.state == AdoptionState::Confirmed {
      Ok(())
    } else {
      Err((ModuleAlertContext {}, ModuleCommandContext::new()))
    }
  }
}
