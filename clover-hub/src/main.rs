#![feature(stmt_expr_attributes)]
#![feature(let_chains)]

mod server;
mod tui;
mod utils;

use log::{debug, info, warn};
use env_logger;
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;
use tokio::sync::mpsc;
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
  env_logger::Builder::new()
    .parse_filters(&env::var("CLOVER_LOG").unwrap_or("info".to_string()))
    .init();

  info!("Starting Clover Hub.");

  // BLEHHHHHHHHHHH Seriously???? Anyways...
  let matches = Box::leak(Box::new(cli().get_matches()));
  let subcommand = matches.subcommand();

  match subcommand {
    Some(("run", sub_matches)) => {
      let run_command = sub_matches.subcommand().unwrap_or(("aio", sub_matches));
      match run_command {
        ("aio", sub_matches) => {
          let mut signals = Signals::new([SIGINT])?;
          let cancellation_token = CancellationToken::new();

          let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default set in Clap.").parse::<u16>());

          info!("Running Backend Server and Terminal UI (All-In-One)...");

          let server_port = Arc::new(port);
          let server_token = cancellation_token.clone();
          let server_handle = tokio::task::spawn(async move { 
            server_main(*server_port.to_owned(), server_token).await; 
          });

          let tui_port = Arc::new(port);
          let tui_token = cancellation_token.clone();
          let tui_handle = tokio::task::spawn(async move { 
            let _ = tui_main(*tui_port.to_owned(), Ok::<String, ()>("localhost".to_string()).ok(), tui_token).await; 
          });

          let signal_token = cancellation_token.clone();
          let signal_handle = tokio::task::spawn(async move {
            for signal in signals.forever() {
              if signal == 2 {
                if signal_token.is_cancelled() {
                  warn!("Forcibly exiting...");
                  exit(1);
                } else { signal_token.cancel(); }
              }
            }
          });

          tokio::select! {_ = futures::future::join_all(vec![signal_handle, tui_handle, server_handle]) => {
            info!("Exiting...");
            exit(0);
          }}
        }
        ("server", sub_matches) => {
          let mut signals = Signals::new([SIGINT])?;
          let cancellation_token = CancellationToken::new();
          let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default provided in Clap.").parse::<u16>());

          info!("Running Backend Server...");
          let server_token = cancellation_token.clone();
          let server_handle = tokio::task::spawn(async move { 
            server_main(port, server_token).await; 
          });
          
          let signal_token = cancellation_token.clone();
          let signal_handle = tokio::task::spawn(async move {
            for signal in signals.forever() {
              if signal == 2 {
                if signal_token.is_cancelled() {
                  warn!("Forcibly exiting...");
                  exit(1);
                } else { signal_token.cancel(); }
              }
            }
          });

          tokio::select! {_ = futures::future::join_all(vec![signal_handle, server_handle]) => {
            info!("Exiting...");
            exit(0);
          }}
        }
        ("tui", sub_matches) => {
          let mut signals = Signals::new([SIGINT])?;
          let cancellation_token = CancellationToken::new();
          let host = sub_matches.get_one::<String>("host").expect("Default set in Clap.");
          let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default set in Clap.").parse::<u16>());

          let signal_token = cancellation_token.clone();
          let signal_handle = tokio::task::spawn(async move {
            for signal in signals.forever() {
              if signal == 2 {
                if signal_token.is_cancelled() {
                  warn!("Forcibly exiting...");
                  exit(1);
                } else { signal_token.cancel(); }
              }
            }
          });

          info!("Running Terminal UI...");
          let tui_host = Arc::new(host);
          let tui_token = cancellation_token.clone();
          let tui_handle = tokio::task::spawn(async move { 
            tui_main(port, Ok::<String, ()>((*tui_host.to_owned()).to_string()).ok(), tui_token).await.err();
          });

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