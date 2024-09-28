#![feature(stmt_expr_attributes)]
#![feature(let_chains)]

mod server;
mod tui;
mod utils;

use log::{info, warn};
use env_logger;
use tokio_util::sync::CancellationToken;
use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::num::ParseIntError;
use std::process::exit;
use std::sync::Arc;
use clap::{Arg, Command};

use crate::server::server_main;
use crate::tui::tui_main;

pub struct Empty {}

const DEFAULT_PORT_STR: &str = "6699";
const DEFAULT_PORT: u16 = 6699;

fn cli() -> Command {
  Command::new("clover")
    .about("Central command and control for the Clover system.")
    .subcommand_required(true)
    .arg_required_else_help(true)
    .allow_external_subcommands(true)
    .subcommand(
      Command::new("run")
        .args_conflicts_with_subcommands(true)
        .flatten_help(true)
        .args(aio_args())
        .subcommand(Command::new("aio").args(aio_args()))
        .subcommand(
          Command::new("server")
            .arg(port_arg())
        )
        .subcommand(
          Command::new("tui")
            .arg(Arg::new("host").short('H').long("host").required(false).default_value("localhost").help("The host to connect to other than `localhost`"))
            .arg(port_arg())
        ),
    )
}

fn port_arg() -> Arg {
  Arg::new("port").short('p').long("port").required(false).default_value(DEFAULT_PORT_STR).help(format!("The port on the host to connect to if not on {}.", DEFAULT_PORT_STR))
}

fn aio_args() -> Vec<Arg> {
  vec![port_arg()]
}

fn unwrap_port_arg(arg: Result<u16, ParseIntError>) -> u16 {
  match arg {
    Ok(val) => { val },
    Err(e) => {
      warn!("User-provided port did not parse correctly, using default, due to:\n{}", e);
      DEFAULT_PORT
    }
  }
}

fn get_signal_handle(cancellation_token: CancellationToken) -> tokio::task::JoinHandle<()> {
  tokio::task::spawn(async move {
    tokio::select! {
      _ = wait_for_signal_impl() => {
        info!("Shutting down...");

        cancellation_token.cancel();
        
        tokio::select! {
          _ = wait_for_signal_impl() => {
            warn!("Forcibly exiting!");
            exit(1);
          },
          _ = cancellation_token.cancelled() => {}
        } 
      }
    }
  })
}

// taken from https://stackoverflow.com/questions/77585473/rust-tokio-how-to-handle-more-signals-than-just-sigint-i-e-sigquit#77591939
/// Waits for a signal that requests a graceful shutdown, like SIGTERM or SIGINT.
#[cfg(unix)]
async fn wait_for_signal_impl() {
  use log::debug;
  use tokio::signal::unix::{signal, SignalKind};

  // Infos here:
  // https://www.gnu.org/software/libc/manual/html_node/Termination-Signals.html
  let mut signal_terminate = signal(SignalKind::terminate()).unwrap();
  let mut signal_interrupt = signal(SignalKind::interrupt()).unwrap();

  tokio::select! {
    _ = signal_terminate.recv() => debug!("Received SIGTERM."),
    _ = signal_interrupt.recv() => debug!("Received SIGINT."),
  };
}

/// Waits for a signal that requests a graceful shutdown, Ctrl-C (SIGINT).
#[cfg(windows)]
async fn wait_for_signal_impl() {
  use tokio::signal::windows;

  // Infos here:
  // https://learn.microsoft.com/en-us/windows/console/handlerroutine
  let mut signal_c = windows::ctrl_c().unwrap();
  let mut signal_break = windows::ctrl_break().unwrap();
  let mut signal_close = windows::ctrl_close().unwrap();
  let mut signal_shutdown = windows::ctrl_shutdown().unwrap();

  tokio::select! {
    _ = signal_c.recv() => debug!("Received CTRL_C."),
    _ = signal_break.recv() => debug!("Received CTRL_BREAK."),
    _ = signal_close.recv() => debug!("Received CTRL_CLOSE."),
    _ = signal_shutdown.recv() => debug!("Received CTRL_SHUTDOWN."),
  };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::Builder::new()
    .parse_filters(&env::var("CLOVER_LOG").unwrap_or("info".to_string()))
    .init();

  info!("Starting CloverHub!");

  let matches = Box::leak(Box::new(cli().get_matches()));
  let subcommand = matches.subcommand();

  match subcommand {
    Some(("run", sub_matches)) => {
      let run_command = sub_matches.subcommand().unwrap_or(("aio", sub_matches));
      match run_command {
        ("aio", sub_matches) => {
          let cancellation_token = CancellationToken::new();

          let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default set in Clap.").parse::<u16>());

          info!("Running Backend Server and Terminal UI (All-In-One)...");

          let server_port = Arc::new(port);
          let server_token = cancellation_token.child_token();
          let server_handle = tokio::task::spawn(async move { 
            server_main(*server_port.to_owned(), server_token).await; 
          });

          let tui_port = Arc::new(port);
          let tui_token = cancellation_token.child_token();
          let tui_handle = tokio::task::spawn(async move { 
            let _ = tui_main(*tui_port.to_owned(), Ok::<String, ()>("localhost".to_string()).ok(), tui_token).await; 
          });

          let signal_handle = get_signal_handle(cancellation_token);

          tokio::select! {_ = futures::future::join_all(vec![signal_handle, tui_handle, server_handle]) => {
            info!("Exiting...");
            exit(0);
          }}
        }
        ("server", sub_matches) => {
          let cancellation_token = CancellationToken::new();
          let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default provided in Clap.").parse::<u16>());

          info!("Running Backend Server...");
          let server_token = cancellation_token.child_token();
          let server_handle = tokio::task::spawn(async move { 
            server_main(port, server_token).await; 
          });
          
          let signal_handle = get_signal_handle(cancellation_token);

          tokio::select! {_ = futures::future::join_all(vec![signal_handle, server_handle]) => {
            info!("Exiting...");
            exit(0);
          }}
        }
        ("tui", sub_matches) => {
          let cancellation_token = CancellationToken::new();
          let host = sub_matches.get_one::<String>("host").expect("Default set in Clap.");
          let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default set in Clap.").parse::<u16>());

          info!("Running Terminal UI...");
          let tui_host = Arc::new(host);
          let tui_token = cancellation_token.child_token();
          let tui_handle = tokio::task::spawn(async move { 
            tui_main(port, Ok::<String, ()>((*tui_host.to_owned()).to_string()).ok(), tui_token).await.err();
          });

          let signal_handle = get_signal_handle(cancellation_token);

          tokio::select! {_ = futures::future::join_all(vec![signal_handle, tui_handle]) => {
            info!("Exiting...");
            exit(0);
          }}
        }
        (name, _) => {
          unreachable!("Unsupported subcommand `{name}`")
        }
      }
    }
    Some((ext, sub_matches)) => {
      let args = sub_matches
        .get_many::<OsString>("")
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
      info!("Calling out to {ext:?} with {args:?}");
    }
    _ => unreachable!(), // If all subcommands are defined above, anything else is unreachable!()
  }

  Ok(())
}
