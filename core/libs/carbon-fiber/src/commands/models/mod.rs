pub mod components;
pub mod self_test;

use serde::{
  Deserialize,
  Serialize,
};

use crate::commands::models::{
  components::{
    GestureTargetCommand,
    GestureTargetFailReply,
    GestureTargetSuccessReply,
  },
  self_test::{
    SelfTestCommand,
    SelfTestFailReply,
    SelfTestSuccessReply,
  },
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MessageContent {
  transaction_id: [u8; 16],
  payload: Command,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Command {
  // POST
  SelfTestCommand(SelfTestCommand),
  SelfTestReply(Result<SelfTestSuccessReply, SelfTestFailReply>),
  // Gestures
  GestureTargetCommand(GestureTargetCommand),
  GestureTargetReply(Result<GestureTargetSuccessReply, GestureTargetFailReply>),
}
