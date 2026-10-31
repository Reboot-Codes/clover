//! # Common Data Structures and Constants for Adoption
//!

use serde::{
  Deserialize,
  Serialize,
};

pub const UNI_HUB: &[u8] = b"CARBON_FIBER_HUB";
pub const UNI_MODULE: &[u8] = b"CARBON_FIBER_MODULE";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdoptionMessage<'a> {
  Probe(ProbeMessage<'a>),
  Hello(HelloMessage<'a>),
  KeyHOMI(KeyHOMIMessage<'a>),
  KeyHIMO(KeyHIMOMessage<'a>),
  Confirm(ConfirmMessage<'a>),
  Abortion(AbortionMessage),
}

/// Lifecycle state machine for the adoption process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AdoptionState {
  Initial,
  Probed,
  KeyExchanged,
  Confirmed,
  Aborted(AbortionReason),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProbeMessage<'a> {
  pub version: (u8, u8, u8),
  pub id: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HelloMessage<'a> {
  pub version: (u8, u8, u8),
  pub rfqdn: &'a str,
  pub salt: [u8; 16],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyHOMIMessage<'a> {
  pub hub_pub: &'a [u8],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyHIMOMessage<'a> {
  pub module_pub: &'a [u8],
  pub hub_conf: &'a [u8],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfirmMessage<'a> {
  pub module_conf: &'a [u8],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AbortionMessage {
  pub reason: AbortionReason,
}

/// Reason for connection abortion.
/// - `Incompatible`: CarbonFiber version missmatch.
/// - `*HOMIFail`: Hub used the wrong pairing key/failed to do math.
/// - `*HIMOFail`: Module had a corruption of the pairing key/failed to do math.
/// - `UserCancel`: User cancelled adoption.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AbortionReason {
  Incompatible = 0xFA,
  KeyHOMIFail = 0x10,
  KeyHIMOFail = 0x01,
  ConfirmHOMIFail = 0x20,
  ConfirmHIMOFail = 0x02,
  UserCancel = 0xBA,
}
