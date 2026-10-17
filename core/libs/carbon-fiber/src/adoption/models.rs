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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ProbeMessage<'a> {
  version: (u8, u8, u8),
  id: &'a str,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct HelloMessage<'a> {
  version: (u8, u8, u8),
  rfqdn: &'a str,
  salt: [u8; 16],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyHOMIMessage<'a> {
  hub_pub: &'a [u8],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct KeyHIMOMessage<'a> {
  module_pub: &'a [u8],
  hub_conf: &'a [u8],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct ConfirmMessage<'a> {
  module_conf: &'a [u8],
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AbortionMessage {
  reason: AbortionReason,
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
