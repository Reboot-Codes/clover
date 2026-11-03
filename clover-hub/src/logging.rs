//! # Logging & Diagnostic Telemetry
//!
//! Provides in-process log event capturing via tracing subscriber layers and forwards
//! structured logs over Zenoh for remote diagnostics and the Clover Terminal UI.
//!
//! ## Zenoh Endpoints
//!
//! ### Publishers
//! - `com/reboot-codes/clover/hub/logs` ([`LOGS_TOPIC`]): Streams live structured JSON log records.
//!   Implemented by [`start_zenoh_log_publisher`].

use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::Resource;
use serde::{
  Deserialize,
  Serialize,
};
use std::sync::Arc;
use std::sync::OnceLock;
use tokio::sync::broadcast;
use tokio_util::sync::CancellationToken;
use tracing::Subscriber;
use tracing_log::LogTracer;
use tracing_subscriber::{
  layer::Context,
  prelude::*,
  EnvFilter,
  Layer,
};

pub const LOGS_TOPIC: &str = "com/reboot-codes/clover/hub/logs";


/// Structured log record published over Zenoh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloverLogRecord {
  pub timestamp: String,
  pub level: String,
  pub target: String,
  pub message: String,
  pub file: Option<String>,
  pub line: Option<u32>,
}

static LOG_SENDER: OnceLock<broadcast::Sender<CloverLogRecord>> = OnceLock::new();

fn get_sender() -> &'static broadcast::Sender<CloverLogRecord> {
  LOG_SENDER.get_or_init(|| {
    let (tx, _rx) = broadcast::channel(4096);
    tx
  })
}

/// Subscribe to live in-process log records
pub fn subscribe_logs() -> broadcast::Receiver<CloverLogRecord> {
  get_sender().subscribe()
}

/// Broadcast a log record directly into the channel
pub fn broadcast_log(record: CloverLogRecord) {
  let _ = get_sender().send(record);
}

/// Tracing subscriber layer that forwards log events to the broadcast channel
pub struct CloverLogLayer;

fn is_allowed_log(target: &str, level: &tracing::Level) -> bool {
  if target.starts_with("clover_hub::logging") {
    return false;
  }

  // Allow all logs from Clover ecosystem crates
  let is_clover_ecosystem = target.starts_with("clover")
    || target.starts_with("carbon_")
    || target.starts_with("sensor_fusion");

  if is_clover_ecosystem {
    return true;
  }

  *level == tracing::Level::INFO
}

impl<S: Subscriber> Layer<S> for CloverLogLayer {
  fn enabled(&self, metadata: &tracing::Metadata<'_>, _ctx: Context<'_, S>) -> bool {
    is_allowed_log(metadata.target(), metadata.level())
  }

  fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
    let metadata = event.metadata();
    let target = metadata.target();
    let level = metadata.level();

    if !is_allowed_log(target, level) {
      return;
    }

    let mut visitor = LogEventVisitor::default();
    event.record(&mut visitor);

    let record = CloverLogRecord {
      timestamp: chrono::Utc::now().to_rfc3339(),
      level: level.to_string(),
      target: target.to_string(),
      message: visitor.message,
      file: metadata.file().map(|s| s.to_string()),
      line: metadata.line(),
    };

    let _ = get_sender().send(record);
  }
}

#[derive(Default)]
struct LogEventVisitor {
  message: String,
}

impl tracing::field::Visit for LogEventVisitor {
  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    if field.name() == "message" {
      let mut formatted = format!("{value:?}");
      if formatted.starts_with('"') && formatted.ends_with('"') && formatted.len() >= 2 {
        formatted = formatted[1..formatted.len() - 1].to_string();
      }
      self.message = formatted;
    } else {
      if !self.message.is_empty() {
        self.message.push_str(", ");
      }
      self
        .message
        .push_str(&format!("{}={:?}", field.name(), value));
    }
  }

  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    if field.name() == "message" {
      self.message = value.to_string();
    } else {
      if !self.message.is_empty() {
        self.message.push_str(", ");
      }
      self
        .message
        .push_str(&format!("{}={}", field.name(), value));
    }
  }
}

/// Spawns an async publisher task that reads from the internal log channel and publishes to Zenoh.
///
/// ### Zenoh Publishers
/// - **Key Expression**: `com/reboot-codes/clover/hub/logs` ([`LOGS_TOPIC`])
/// - **Payload Format**: Serialized JSON string of [`CloverLogRecord`]
/// - **Trigger**: Every log event emitted inside the process (matching `CLOVER_LOG` / default filter).
///
/// #### Payload Example
/// ```json
/// {
///   "timestamp": "2026-11-03 08:30:00",
///   "level": "INFO",
///   "target": "clover_hub::server",
///   "message": "CloverHub Server has started",
///   "file": "clover-hub/src/server/mod.rs",
///   "line": 150
/// }
/// ```
pub async fn start_zenoh_log_publisher(
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
) {
  let publisher = match session.declare_publisher(LOGS_TOPIC).await {
    Ok(p) => p,
    Err(e) => {
      tracing::error!("Failed to declare publisher on {LOGS_TOPIC}: {e}");
      return;
    }
  };

  let mut rx = subscribe_logs();
  while !cancellation_token.is_cancelled() {
    tokio::select! {
      _ = cancellation_token.cancelled() => break,
      res = rx.recv() => {
        match res {
          Ok(record) => {
            if let Ok(json) = serde_json::to_string(&record) {
              let _ = publisher.put(json).await;
            }
          }
          Err(broadcast::error::RecvError::Lagged(_)) => {
            // Buffer lagged; drop missed logs without deadlocking
          }
          Err(broadcast::error::RecvError::Closed) => break,
        }
      }
    }
  }
}

/// Spawns an async publisher task that connects to Zenoh on the specified port and publishes internal logs to Zenoh
pub async fn start_zenoh_log_publisher_for_port(port: u16, cancellation_token: CancellationToken) {
  let endpoint = format!("tcp/localhost:{port}");
  let zenoh_config = match crate::utils::configure_zenoh(vec![
    ("connect/endpoints", &format!("[\"{endpoint}\"]")),
    ("mode", "\"client\""),
    (
      "timestamping/enabled",
      r#"{ router: true, peer: true, client: true }"#,
    ),
  ]) {
    Ok(cfg) => cfg,
    Err(e) => {
      tracing::warn!(
        "Failed to build Zenoh config for log publisher: {e}. Falling back to default peer mode."
      );
      zenoh::Config::default()
    }
  };

  let open_future = async {
    match zenoh::open(zenoh_config).await {
      Ok(s) => Some(Arc::new(s)),
      Err(e) => {
        tracing::warn!("Failed to open Zenoh session for log publisher on {port}: {e}. Retrying in background peer mode...");
        match zenoh::open(zenoh::Config::default()).await {
          Ok(s) => Some(Arc::new(s)),
          Err(err) => {
            tracing::error!("Could not open Zenoh session for log publisher: {err}");
            None
          }
        }
      }
    }
  };

  let session = tokio::select! {
    _ = cancellation_token.cancelled() => return,
    res = open_future => match res {
      Some(s) => s,
      None => return,
    }
  };

  start_zenoh_log_publisher(session, cancellation_token).await;
}

pub fn setup_logging(use_stdout: bool) -> SdkLoggerProvider {
  // This *shouldn't* ever break.
  LogTracer::init().unwrap();

  let provider: SdkLoggerProvider = SdkLoggerProvider::builder()
    .with_resource(Resource::builder().with_service_name("clover-hub").build())
    .build();

  let env_filter = EnvFilter::try_from_env("CLOVER_LOG")
    .unwrap_or_else(|_| EnvFilter::new("info").add_directive("clover_hub=debug".parse().unwrap()));
  let stdout_layer = tracing_subscriber::fmt::layer()
    .with_writer(std::io::stderr)
    .with_thread_names(true)
    .with_filter(env_filter);

  let zenoh_layer = CloverLogLayer;

  if use_stdout {
    let subscriber = tracing_subscriber::registry()
      .with(stdout_layer)
      .with(zenoh_layer);

    tracing::subscriber::set_global_default(subscriber)
      .expect("Failed to set global tracing subscriber");
  } else {
    let subscriber = tracing_subscriber::registry().with(zenoh_layer);

    tracing::subscriber::set_global_default(subscriber)
      .expect("Failed to set global tracing subscriber");
  }

  provider
}
