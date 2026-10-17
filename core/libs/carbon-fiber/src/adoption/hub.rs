use crate::{
  adoption::models::{
    AbortionReason,
    ConfirmMessage,
    HelloMessage,
    KeyHIMOMessage,
    KeyHOMIMessage,
    ProbeMessage,
  },
  alerts::hub::HubAlertContext,
  commands::hub::HubCommandContext,
};

pub struct HubAdoptionContext<'a> {
  pub module_id: &'a str,
  pub module_rfqdn: &'a str,
  pub pin: [u8; 6],
}

impl<'a> HubAdoptionContext<'a> {
  pub fn init_context() -> Self {
    todo!();
  }

  pub fn gen_probe() -> ProbeMessage<'a> {
    todo!();
  }

  pub fn parse_hello(msg: &HelloMessage) -> Result<(), AbortionReason> {
    todo!();
  }

  pub fn gen_key_homi() -> KeyHOMIMessage<'a> {
    todo!();
  }

  pub fn parse_key_himo(msg: &KeyHIMOMessage) -> Result<(), AbortionReason> {
    todo!();
  }

  pub fn gen_confirm() -> ConfirmMessage<'a> {
    todo!();
  }

  pub fn gen_command_context() -> Result<(), (HubAlertContext, HubCommandContext)> {
    todo!();
  }
}
