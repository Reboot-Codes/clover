//! # CloverHub
//! The Rust-based nerve centre for all communication between modules, their components, you, and configuration tools. It is built to be secure and performant, while allowing for flexible implementation for everything from spicing up your cosplay, to surgical body mods.
//!
//! ## CLI
//! Found here in [`cli`], will spin up the required tokio threads.
//!
//! ## [Server](server)
//! Contains the modular, core logic to run a clover instance. You probably want this.
//!
//! ## [TUI](tui)
//! Terminal User Interface to manage a clover instance over UART/SSH.
//!

use std::error::Error;
use tokio_util::sync::CancellationToken;

use clover_hub::run;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
  let big_boy_token = CancellationToken::new();

  tokio::select! {
    _ = run(big_boy_token.clone()) => {},
    _ = big_boy_token.cancelled() => {}
  }

  Ok(())
}
