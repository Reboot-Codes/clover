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
use tracing_log::LogTracer;
use tracing_subscriber::{
  prelude::*,
  EnvFilter,
};

use clover_hub::run;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn Error>> {
  LogTracer::init()?;

  let env_filter = EnvFilter::try_from_env("CLOVER_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
  let stdout_layer = tracing_subscriber::fmt::layer();
  // TODO: Send logs to Zenoh log endpoint.
  // let zenoh_layer = ZenohLogLayer { session: active_session, host_id: "hub-sbc".into() };

  let subscriber = tracing_subscriber::registry()
    .with(env_filter)
    .with(stdout_layer);
  // .with(zenoh_layer);

  tracing::subscriber::set_global_default(subscriber)
    .expect("Failed to set global tracing subscriber");

  // Debugging console for tokio
  // console_subscriber::init();

  let big_boy_token = CancellationToken::new();

  tokio::select! {
    _ = run(big_boy_token.clone()) => {},
    _ = big_boy_token.cancelled() => {}
  }

  Ok(())
}
