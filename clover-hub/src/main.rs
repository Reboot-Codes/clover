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

use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;
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

  // let exporter = opentelemetry_stdout::LogExporter::default();
  let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
    .with_resource(Resource::builder().with_service_name("clover-hub").build())
    // TODO: Send logs to Zenoh log endpoint.
    //  .with_simple_exporter(exporter)
    .build();

  // To prevent a telemetry-induced-telemetry loop, OpenTelemetry's own internal
  // logging is properly suppressed. However, logs emitted by external components
  // (such as reqwest, tonic, etc.) are not suppressed as they do not propagate
  // OpenTelemetry context. Until this issue is addressed
  // (https://github.com/open-telemetry/opentelemetry-rust/issues/2877),
  // filtering like this is the best way to suppress such logs.
  //
  // The filter levels are set as follows:
  // - Allow `info` level and above by default.
  // - Completely restrict logs from `hyper`, `tonic`, `h2`, and `reqwest`.
  //
  let otel_filter = EnvFilter::new("info")
    .add_directive("hyper=off".parse().unwrap())
    .add_directive("tonic=off".parse().unwrap())
    .add_directive("h2=off".parse().unwrap())
    .add_directive("reqwest=off".parse().unwrap());
  let otel_layer = layer::OpenTelemetryTracingBridge::new(&provider).with_filter(otel_filter);

  let env_filter = EnvFilter::try_from_env("CLOVER_LOG")
    .unwrap_or_else(|_| EnvFilter::new("info").add_directive("clover_hub=debug".parse().unwrap()));
  let stdout_layer = tracing_subscriber::fmt::layer()
    .with_thread_names(true)
    .with_filter(env_filter);

  let subscriber = tracing_subscriber::registry()
    .with(otel_layer)
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

  let _ = provider.shutdown();

  Ok(())
}
