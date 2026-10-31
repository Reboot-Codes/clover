//! # Hub-Specific Adoption Code
//!
//! For simplicity, adoption crypto and data structures are
//! fully defined in the CarbonFiber crate to make testing and
//! updates easier.
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
    UNI_HUB,
  },
  alerts::hub::HubAlertContext,
  commands::hub::HubCommandContext,
};

pub struct HubAdoptionContext<'a> {
  pub module_id: &'a str,
  pub module_rfqdn: &'a str,
  pub pin: [u8; 6],
  pub salt: [u8; 16],
  pub key_share: [u8; 32],
  pub confirmation: [u8; 32],
  pub state: AdoptionState,
}

impl<'a> HubAdoptionContext<'a> {
  /// Initializes the hub adoption context with blank defaults.
  pub fn init_context() -> Self {
    Self::new("", "", [0u8; 6])
  }

  /// Initializes the hub adoption context with module ID, expected RFQDN, and user-entered pairing PIN.
  pub fn new(module_id: &'a str, module_rfqdn: &'a str, pin: [u8; 6]) -> Self {
    Self {
      module_id,
      module_rfqdn,
      pin,
      salt: [0u8; 16],
      key_share: [0u8; 32],
      confirmation: [0u8; 32],
      state: AdoptionState::Initial,
    }
  }

  /// Generates the initial ProbeMessage to discover and verify module compatibility.
  pub fn gen_probe(&self) -> ProbeMessage<'a> {
    ProbeMessage {
      version: crate::PROTOCOL_VERSION,
      id: self.module_id,
    }
  }

  /// Parses the HelloMessage from the module and ingests the module salt.
  pub fn parse_hello(&mut self, msg: &HelloMessage) -> Result<(), AbortionReason> {
    if msg.version.0 != crate::PROTOCOL_VERSION_MAJOR {
      self.state = AdoptionState::Aborted(AbortionReason::Incompatible);
      return Err(AbortionReason::Incompatible);
    }
    self.salt = msg.salt;
    self.state = AdoptionState::Probed;
    Ok(())
  }

  /// Computes the Hub's ephemeral key share and emits KeyHOMIMessage.
  pub fn gen_key_homi(&mut self) -> KeyHOMIMessage<'_> {
    let mut hasher = sha2::Sha256::new();
    hasher.update(&self.pin);
    hasher.update(&self.salt);
    hasher.update(UNI_HUB);
    self.key_share = hasher.finalize().into();
    KeyHOMIMessage {
      hub_pub: &self.key_share,
    }
  }

  /// Parses KeyHIMOMessage from the module and verifies the confirmation tag.
  pub fn parse_key_himo(&mut self, msg: &KeyHIMOMessage) -> Result<(), AbortionReason> {
    if msg.module_pub.is_empty() || msg.hub_conf.is_empty() {
      self.state = AdoptionState::Aborted(AbortionReason::KeyHIMOFail);
      return Err(AbortionReason::KeyHIMOFail);
    }

    // Verify confirmation tag from module
    let mut conf_hasher = sha2::Sha256::new();
    conf_hasher.update(&self.key_share);
    conf_hasher.update(msg.module_pub);
    conf_hasher.update(&self.pin);
    let expected_conf: [u8; 32] = conf_hasher.finalize().into();

    if msg.hub_conf != &expected_conf[..] {
      self.state = AdoptionState::Aborted(AbortionReason::KeyHOMIFail);
      return Err(AbortionReason::KeyHOMIFail);
    }

    self.confirmation = expected_conf;
    self.state = AdoptionState::KeyExchanged;
    Ok(())
  }

  /// Generates the ConfirmMessage containing mutual confirmation.
  pub fn gen_confirm(&self) -> ConfirmMessage<'_> {
    ConfirmMessage {
      module_conf: &self.confirmation,
    }
  }

  /// Generates hub command and alert contexts upon successful adoption.
  pub fn gen_command_context(&self) -> Result<(), (HubAlertContext, HubCommandContext)> {
    if self.state == AdoptionState::Confirmed {
      Ok(())
    } else {
      Err((HubAlertContext {}, HubCommandContext {}))
    }
  }
}
