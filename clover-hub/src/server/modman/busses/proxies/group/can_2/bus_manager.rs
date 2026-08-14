use std::{
  collections::HashMap,
  sync::Arc,
};

use anyhow::anyhow;
use embedded_can::Id;
use linux_socketcan_iso_tp::{
  self,
  flags,
  IsoTpFlowControlOptions,
  IsoTpKernelOptions,
  IsoTpSocketOptions,
  TokioSocketCanIsoTp,
};
use regex::Regex;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
};

use crate::server::modman::{
  busses::proxies::group::can_2::{
    module_listener::{
      can_module_rx,
      can_module_tx,
    },
    CAN2Bus,
  },
  models::PortStatus,
};

#[instrument]
pub fn parse_id_str(id_str: &str) -> Result<u16, anyhow::Error> {
  todo!()
}

pub fn match_to_str(match_struct: regex::Match<'_>) -> &str {
  <regex::Match<'_> as Into<&str>>::into(match_struct)
}

#[instrument(skip(ctx, cancellation_token))]
pub async fn can_bus_manager(
  ctx: Arc<CAN2Bus>,
  cancellation_token: CancellationToken,
  iface_name: String,
) {
  let can_2_port_status_mutex = ctx.store.port_statuses.can_2.clone();
  let listener_registry: Arc<Mutex<HashMap<String, CancellationToken>>> =
    Arc::new(Mutex::new(HashMap::new()));

  while !cancellation_token.is_cancelled() {
    let port_statuses = can_2_port_status_mutex.lock().await;
    let mut port_statuses_snapshot = Vec::new();

    // Matches against strings like: `"can0/0xFFF:0xFFF"`.
    // This regex does not validate if the specified IDs are within the CAN range though.
    // That's done later by `embedded_can`.
    let port_specifier_re = Regex::new(
      r"^(?<iface>(?:\\w|[0-9])+)\\/(?<rx_id>0[xX][0-9a-fA-F]{3}):(?<tx_id>0[xX][0-9a-fA-F]{3})$",
    )
    .unwrap();

    // We need to make sure that we're not leaving that mutex locked for too long.
    for port_status in port_statuses.iter() {
      port_statuses_snapshot.push((port_status.0.clone(), port_status.1.clone()));
    }

    drop(port_statuses);

    for port_status_tuple in port_statuses_snapshot {
      let (port_path, port_status) = port_status_tuple;

      match port_specifier_re.captures(&port_path) {
        Some(re_captures) => {
          // Known good value since the regex is static, and the haystack matched.
          let requested_iface = re_captures.name("iface").unwrap();

          if match_to_str(requested_iface) == &iface_name {
            match port_status {
              PortStatus::Requested(module_id) => {
                setup_listener(
                  ctx.clone(),
                  port_path.clone(),
                  module_id,
                  (
                    match_to_str(re_captures.name("rx_id").unwrap()),
                    match_to_str(re_captures.name("tx_id").unwrap()),
                  ),
                  listener_registry.clone(),
                )
                .await;
              }
              PortStatus::Unavailable(module_id) => {
                setup_listener(
                  ctx.clone(),
                  port_path.clone(),
                  module_id,
                  (
                    match_to_str(re_captures.name("tx_id").unwrap()),
                    match_to_str(re_captures.name("tx_id").unwrap()),
                  ),
                  listener_registry.clone(),
                )
                .await;
              }
              PortStatus::Unrequested(module_id) => {
                match listener_registry.lock().await.get(&module_id) {
                  Some(listener_token) => {
                    info!("Shutting down CAN 2 listener for Module: {module_id}...");
                    listener_token.cancel();
                  }
                  None => {
                    error!("Listener for Module: {module_id}, was asked to be unbound, but there's no listener for that module; this is a bug and should be reported!");
                  }
                }
              }
              _ => {}
            }
          }
        }
        None => {}
      }
    }
  }

  for (module_id, listener_token) in listener_registry.lock().await.iter() {
    debug!("Shutting down CAN 2 listener for module: {module_id}...");
    listener_token.cancel();
  }
}

#[instrument(skip(ctx, listener_registry))]
pub async fn setup_listener(
  ctx: Arc<CAN2Bus>,
  iface_name: String,
  module_id: String,
  id_tuple: (&str, &str),
  listener_registry: Arc<Mutex<HashMap<String, CancellationToken>>>,
) {
  let mut ret: Option<anyhow::Error> = None;

  match parse_id_str(id_tuple.0) {
    Ok(raw_rx_id) => match parse_id_str(id_tuple.1) {
      Ok(raw_tx_id) => {
        let rx_options = IsoTpKernelOptions::default();
        let tx_options = IsoTpKernelOptions {
          socket: IsoTpSocketOptions {
            flags: flags::CAN_ISOTP_LISTEN_MODE,
            ..Default::default()
          },
          ..Default::default()
        };

        match embedded_can::StandardId::new(raw_rx_id).ok_or(anyhow!(
          "We expect that an RX ID of {}, is valid. Check your config or there's a bug in manifest validation!",
          raw_rx_id
        )) {
          Ok(rx_id) => {
            match embedded_can::StandardId::new(raw_tx_id).ok_or(anyhow!(
              "We expect that a TX ID of {}, is valid. Check your config or there's a bug in manifest validation!",
              raw_tx_id
            )) {
              Ok(tx_id) => {
                match TokioSocketCanIsoTp::open(
                  &iface_name,
                  Id::Standard(rx_id),
                  Id::Standard(tx_id),
                  &rx_options,
                ) {
                  Ok(rx_socket) => {
                    match TokioSocketCanIsoTp::open(
                      &iface_name,
                      Id::Standard(rx_id),
                      Id::Standard(tx_id),
                      &tx_options
                    ) {
                      Ok(tx_socket) => {
                        let listener_token = CancellationToken::new();

                        listener_registry.lock().await.insert(module_id.clone(), listener_token.clone());

                        let rx_session = ctx.session.clone();
                        let rx_token = listener_token.clone();
                        let rx_id = module_id.clone();
                        tokio::task::spawn(async move {
                          can_module_rx(rx_session, rx_token, rx_socket, rx_id).await;
                        });

                        let tx_session = ctx.session.clone();
                        let tx_token = listener_token.clone();
                        let tx_id = module_id.clone();
                        tokio::task::spawn(async move {
                          can_module_tx(tx_session, tx_token, tx_socket, tx_id).await;
                        });
                      },
                      Err(err) => todo!(),
                    }
                  },
                  Err(err) => {
                    ret = Some(err.into());
                  },
                }
              },
              Err(err) => {
                ret = Some(err.into());
              },
            }
          },
          Err(err) => {
            ret = Some(err.into());
          },
        }
      }
      Err(err) => {
        ret = Some(err.into());
      }
    },
    Err(err) => {
      ret = Some(err.into());
    }
  }

  match ret {
    Some(err) => todo!(),
    None => {}
  }
}
