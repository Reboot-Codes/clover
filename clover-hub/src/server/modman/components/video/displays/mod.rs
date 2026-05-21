//! # Video Displays
//!
//! Display components are registered with ModMan, and then registered with [Renderer](crate::server::renderer) to make use of hardware graphical acceleration and a unified rendering pipeline, they can then also play along with a stream using a Media Player activity ([CarbonSteel]/[Tesseract])
//!

pub mod impls;
pub mod models;
