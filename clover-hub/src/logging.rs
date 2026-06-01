use std::sync::Arc;

use opentelemetry_appender_tracing::layer;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;
use tracing_log::LogTracer;
use tracing_subscriber::{
  prelude::*,
  EnvFilter,
};

pub fn setup_logging(use_stdout: bool) -> SdkLoggerProvider {
  // This *shouldn't* ever break.
  LogTracer::init().unwrap();

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
    .with_writer(std::io::stderr)
    .with_thread_names(true)
    .with_filter(env_filter);

  // No, I can't just option the .with(...) because rust is stupid when it comes to traits as types in uninitalized variables.
  if use_stdout {
    let subscriber = tracing_subscriber::registry()
      .with(otel_layer)
      .with(stdout_layer);
    // .with(zenoh_layer);

    tracing::subscriber::set_global_default(subscriber)
      .expect("Failed to set global tracing subscriber");
  } else {
    let subscriber = tracing_subscriber::registry()
      .with(otel_layer)
      .with(tui_logger::TuiTracingSubscriberLayer);
    // .with(zenoh_layer);

    tui_logger::init_logger(tui_logger::LevelFilter::Debug).unwrap();

    tracing::subscriber::set_global_default(subscriber)
      .expect("Failed to set global tracing subscriber");
  }

  // Debugging console for tokio
  // console_subscriber::init();

  provider
}
