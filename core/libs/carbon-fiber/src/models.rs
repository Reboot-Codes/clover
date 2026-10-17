use serde::{
  Deserialize,
  Serialize,
};

use crate::adoption::models::AdoptionMessage;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'a"))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BusMessage<'a> {
  Adoption(AdoptionMessage<'a>),
  Content(RawContentMessage<'a>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawContentMessage<'a> {
  pub nonce: [u8; 12],
  pub data: &'a [u8],
  pub hmac: [u8; 32],
}
