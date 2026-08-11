use std::sync::Arc;

use serde::{
  Deserialize,
  Serialize,
};
use tokio::io::{
  AsyncWriteExt,
  WriteHalf,
};
use tokio_serial::SerialStream;
use tracing::{
  debug,
  error,
  instrument,
};

use crate::server::modman::busses::models::{
  BusMessage,
  ContentMessage,
};
use crate::server::modman::busses::proxies::uart::PortToBind;
use crate::server::modman::MODULE_EVT_ID;

#[derive(Serialize, Deserialize, Debug, Clone, strum_macros::Display)]
pub enum UARTTXError {
  #[strum(to_string = "missing-payload")]
  MissingPayload,
  #[strum(to_string = "payload-is-not-string")]
  PayloadIsNotString,
  #[strum(to_string = "malformed-payload")]
  MalformedPayload,
  #[strum(to_string = "malformed-payload-wrapper")]
  MalformedPayloadWrapper,
  #[strum(to_string = "tx-failed")]
  TXFailed,
}

#[instrument]
async fn report_error(error: UARTTXError, query: zenoh::query::Query, key_expr: &str) {
  match query.reply(key_expr, format!("error:{error}")).await {
    Ok(_) => {}
    Err(err) => {
      error!("Failed to reply to query with error message due to:\n{err}");
    }
  }
}

#[instrument(skip(port_session, port_write))]
pub async fn uart_tx_thread(
  tx_bind_info: PortToBind,
  port_session: Arc<zenoh::Session>,
  tx_port_ctx: (String, String),
  mut port_write: WriteHalf<SerialStream>,
) {
  let (module_id, _port_name) = tx_port_ctx;

  let key_expr = format!("{MODULE_EVT_ID}/modules/by-id/{module_id}/send");

  match port_session.declare_queryable(&key_expr).await {
    Ok(queryable) => {
      // If we don't fail out, this block is what should get put into that new thread.

      while let Ok(query) = queryable.recv_async().await {
        match query.payload() {
          Some(payload) => {
            match payload.try_to_string() {
              Ok(payload_str) => {
                debug!("Sending message: {payload_str}...");

                // TODO: Encrypt
                match rmp_serde::to_vec(&payload_str) {
                  Ok(msg_vec) => {
                    let wrapped_message = BusMessage::Content(ContentMessage {
                      nonce: vec![],
                      data: msg_vec,
                      hmac: vec![],
                    });

                    match rmp_serde::to_vec(&wrapped_message) {
                      Ok(wrapped_vec) => match port_write.write(wrapped_vec.as_slice()).await {
                        Ok(size) => match query.reply(&key_expr, format!("{size}")).await {
                          Ok(_) => {}
                          Err(err) => {
                            error!(
                              "Failed to reply to client that we were able to send the message, due to:\n{err}"
                            );
                          }
                        },
                        Err(err) => {
                          error!("Failed to write to UART port due to:\n{err}");
                          report_error(UARTTXError::TXFailed, query, &key_expr).await;
                        }
                      },
                      Err(err) => {
                        error!("Failed to wrap payload as a BusMessage; this is a bug and should be reported! This happened due to:\n{err}");
                        report_error(UARTTXError::MalformedPayloadWrapper, query, &key_expr).await;
                      }
                    }
                  }
                  Err(err) => {
                    error!("Failed to parse payload into msgpack, due to:\n{err}");
                    report_error(UARTTXError::MalformedPayload, query, &key_expr).await;
                  }
                }
              }
              Err(err) => {
                error!("Query's payload could not be decoded into a string, due to:\n{err}");
                report_error(UARTTXError::PayloadIsNotString, query, &key_expr).await;
              }
            }
          }
          None => {
            error!("Query was sent to the module endpoint without a payload!");
            report_error(UARTTXError::MissingPayload, query, &key_expr).await;
          }
        }
      }
    }
    Err(err) => {
      // TODO: implement retries, otherwise fail out the UART bridge creation.
      error!(
        "Failed to create queryable; this is a bug and should be reported! This was due to:\n{err}"
      );
    }
  }
}
