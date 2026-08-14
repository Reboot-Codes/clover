use std::{
  sync::Arc,
  time::Duration,
};

use can_isotp_interface::{
  IsoTpAsyncEndpoint,
  IsoTpEndpoint,
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
    Ok(publisher) => {}
    Err(err) => todo!(),
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
                        Err(err) => todo!(),
                      }
                    }
                    Err(err) => todo!(),
                  },
                  Err(err) => todo!(),
                }
              }
              Err(err) => todo!(),
            },
            None => {}
          }
        }
      }
    }
    Err(err) => todo!(),
  };
}
