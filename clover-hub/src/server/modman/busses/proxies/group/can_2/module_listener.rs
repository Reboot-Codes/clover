use std::{
  sync::Arc,
  time::Duration,
};

use can_isotp_interface::{
  IsoTpAsyncEndpoint,
  RecvControl,
  RecvError,
  RecvStatus,
};
use linux_socketcan_iso_tp::TokioSocketCanIsoTp;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  instrument,
  warn,
};
use zenoh_ext::{
  AdvancedPublisherBuilderExt,
  CacheConfig,
};

use crate::server::modman::{
  busses::models::BusMessage,
  MODULE_EVT_ID,
};

#[instrument(skip(session, cancellation_token, socket))]
pub async fn can_module_rx(
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
  mut socket: TokioSocketCanIsoTp,
  module_id: String,
) {
  let key_expr = format!("{MODULE_EVT_ID}/modules/by-id/{}/recv", &module_id);

  match session
    .declare_publisher(&key_expr)
    .cache(CacheConfig::default().max_samples(1))
    .await
  {
    Ok(publisher) => {
      while !cancellation_token.is_cancelled() {
        let mut payload: Vec<u8> = Vec::new();
        match socket
          .recv_one(Duration::from_millis(100), |_meta, p| {
            // idek why this callback exists.
            for byte in p {
              payload.push(byte.to_owned());
            }

            Ok(RecvControl::Continue)
          })
          .await
        {
          Ok(_) => match rmp_serde::from_slice::<BusMessage>(&payload) {
            Ok(decoded_payload) => match serde_json::to_string(&decoded_payload) {
              Ok(json_payload) => match publisher.put(json_payload).await {
                Ok(_) => {
                  debug!("Successfully proxied CAN 2 message from module: {module_id}!");
                }
                Err(err) => {
                  error!("Failed to publish message to Zenoh, at this stage, we've either lost connectivity, or there's a massive problem. (Might be a bug) Due to:\n{err}");
                }
              },
              Err(err) => {
                error!("Failed to produce a JSON payload from the decoded message, this is a bug and should be reported! Due to:\n{err}");
              }
            },
            Err(err) => {
              // TODO: Do we want to tell the module that it fucked up?
              error!("Invalid message from module: {module_id}, this is a bug (or bad connection) and should (probably) be reported to the module maintainer! Happened due to:\n{err}");
            }
          },
          Err(err) => match err {
            RecvError::BufferTooSmall { needed, got } => {
              error!(
                "CAN 2 recv buffer is too small ({} byte(s) vs {} byte(s), {} byte(s) too small), this is a bug and should be reported!",
                needed, got, (needed - got)
              );
            }
            RecvError::Backend(backend_err) => match backend_err {
              linux_socketcan_iso_tp::Error::Io(sub_backend_err) => {
                error!("CAN 2 RX had a Linux I/O Error due to:\n{sub_backend_err}");
              }
              linux_socketcan_iso_tp::Error::InvalidConfig(backend_err_reason) => {
                error!("Config was invalid for the RX side of the CAN 2 listener. This is probably a bug and should be reported! Due to:\n{backend_err_reason}\nPayload:\n{payload:#?}");
              }
            },
          },
        }

        // I trust rustc, but also we need to save memory!!!
        drop(payload);
      }
    }
    Err(err) => {
      error!("Unable to create a zenoh broadcaster at: {key_expr}, there's probably an error in your configuration. Due to:\n{err}");
    }
  };
}

#[instrument(skip(session, cancellation_token, socket))]
pub async fn can_module_tx(
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
  mut socket: TokioSocketCanIsoTp,
  module_id: String,
) {
  let key_expr = format!("{MODULE_EVT_ID}/modules/by-id/{}/recv", &module_id);

  match session.declare_queryable(&key_expr).await {
    Ok(queryable) => {
      while !cancellation_token.is_cancelled() {
        if let Ok(query) = queryable.recv_async().await {
          match query.payload() {
            Some(query_payload) => match query_payload.try_to_string() {
              Ok(payload_str) => {
                match serde_json_lenient::from_str::<BusMessage>(&payload_str.to_owned()) {
                  Ok(message) => match rmp_serde::to_vec(&message) {
                    Ok(msg_bytes) => {
                      debug!(
                        "Dumping everything in the recv buffer so we don't get a memory leak."
                      );
                      loop {
                        match socket
                          .recv_one(Duration::ZERO, |_meta, _payload| {
                            // just discard whatever showed up
                            Ok(RecvControl::Continue)
                          })
                          .await
                        {
                          Ok(RecvStatus::DeliveredOne) => continue,
                          Ok(RecvStatus::TimedOut) => break,
                          Err(RecvError::BufferTooSmall { needed, got }) => {
                            error!(
                              needed,
                              got, "drain buffer undersized, this is a bug and should be reported!"
                            );
                            break;
                          }
                          Err(RecvError::Backend(e)) => {
                            warn!(?e, "Isotp drain failed, socket likely dead!!");
                            break;
                          }
                        }
                      }

                      debug!("Sending CAN 2 message to module: {module_id}...");
                      match socket
                        .send_to(0, &msg_bytes, Duration::from_millis(100))
                        .await
                      {
                        Ok(_) => {
                          debug!("Successfully sent CAN 2 message to module: {module_id}!");
                        }
                        Err(err) => {
                          error!("Unable to send message to CAN 2 socket due to:\n{err:#?}");
                        }
                      }
                    }
                    Err(err) => {
                      error!("Could not turn the BusMessage into a valid MSGPack byte array, this is a bug and should be reported! Due to:\n{err}");
                    }
                  },
                  Err(err) => {
                    // TODO: do we tell the querier that they fucked up?
                    let querier_str = match query.source_info() {
                      Some(source_info) => {
                        format!(" from source: {}", source_info.source_id().zid())
                      }
                      None => "".to_string(),
                    };

                    error!(
                      "Invalid query payload{querier_str}, due to:\n{err}\nPayload:\n{query_payload:#?}"
                    );
                  }
                }
              }
              Err(err) => {
                // TODO: do we tell the querier that they fucked up?
                let querier_str = match query.source_info() {
                  Some(source_info) => {
                    format!(" from source: {}", source_info.source_id().zid())
                  }
                  None => "".to_string(),
                };

                error!(
                  "Query is not string{querier_str}, due to:\n{err}\nPayload:\n{query_payload:#?}"
                );
              }
            },
            None => {
              let querier_str = match query.source_info() {
                Some(source_info) => {
                  format!(" talking to you, {}...", source_info.source_id().zid())
                }
                None => "".to_string(),
              };

              warn!("Empty queries are ignored...{querier_str}");
            }
          }
        }
      }
    }
    Err(err) => {
      error!("Unable to create a zenoh queryable at: {key_expr}, there's probably an error in your configuration. Due to:\n{err}");
    }
  };
}
