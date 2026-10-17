#![cfg_attr(not(feature = "std"), no_std)]

pub mod adoption;
pub mod alerts;
pub mod commands;
pub mod models;
pub mod module;

const fn parse_u8(s: &str) -> u8 {
  let bytes = s.as_bytes();
  let mut val: u8 = 0;
  let mut i = 0;
  while i < bytes.len() {
    let digit = bytes[i] - b'0';
    val = val * 10 + digit;
    i += 1;
  }
  val
}

pub const PROTOCOL_VERSION_MAJOR: u8 = parse_u8(env!("CARGO_PKG_VERSION_MAJOR"));
pub const PROTOCOL_VERSION_MINOR: u8 = parse_u8(env!("CARGO_PKG_VERSION_MINOR"));
pub const PROTOCOL_VERSION_PATCH: u8 = parse_u8(env!("CARGO_PKG_VERSION_PATCH"));

pub const PROTOCOL_VERSION: (u8, u8, u8) = (
  PROTOCOL_VERSION_MAJOR,
  PROTOCOL_VERSION_MINOR,
  PROTOCOL_VERSION_PATCH,
);
