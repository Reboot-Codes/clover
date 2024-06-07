#![feature(stmt_expr_attributes)]
#![feature(let_chains)]

mod server;
mod tui;
mod utils;

use log::{info, warn};
use env_logger;
use std::env;
use std::ffi::OsString;
use std::num::ParseIntError;
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
async fn main() {
    env_logger::Builder::new()
        .parse_filters(&env::var("CLOVER_LOG").unwrap_or("info".to_string()))
        .init();

    info!("Starting Clover Hub.");

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("run", sub_matches)) => {
            let run_command = sub_matches.subcommand().unwrap_or(("aio", sub_matches));
            match run_command {
                ("aio", sub_matches) => {
                    let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default set in Clap.").parse::<u16>());

                    info!("Running Backend Server and Terminal UI (All-In-One)...");

                    let server_port = Arc::new(port);
                    let server_handle = tokio::task::spawn(async move { server_main(*server_port.to_owned()).await; });
                    let tui_port = Arc::new(port);
                    let tui_handle = tokio::task::spawn(async move { let _ = tui_main(*tui_port.to_owned(), Ok::<String, ()>("localhost".to_string()).ok()).await; });

                    futures::future::join_all(vec![tui_handle, server_handle]).await;
                }
                ("server", sub_matches) => {
                    let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default provided in Clap.").parse::<u16>());

                    info!("Running Backend Server...");
                    server_main(port).await;
                }
                ("tui", sub_matches) => {
                    let host = sub_matches.get_one::<String>("host").expect("Default set in Clap.");
                    let port = unwrap_port_arg(sub_matches.get_one::<String>("port").expect("Default set in Clap.").parse::<u16>());

                    info!("Running Terminal UI...");
                    let tui_host = Arc::new(host);
                    tui_main(port, Ok::<String, ()>((*tui_host.to_owned()).to_string()).ok()).await.err();
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

    // Continued program logic goes here...
}