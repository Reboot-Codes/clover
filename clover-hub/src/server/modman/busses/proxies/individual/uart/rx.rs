use std::sync::Arc;

use crate::server::modman::{
  busses::models::BusMessage,
  MODULE_EVT_ID,
};
use tokio::sync::mpsc::UnboundedReceiver;
use tracing::{
  debug,
  error,
  instrument,
};
use zenoh_ext::{
  AdvancedPublisherBuilderExt,
  CacheConfig,
};

#[instrument(skip(port_session))]
pub async fn uart_rx_thread(
  rx_port_ctx: (String, String),
  mut rx_channel: UnboundedReceiver<BusMessage>,
  port_session: Arc<zenoh::Session>,
) {
  let (module_id, _port_name) = rx_port_ctx;

  let key_expr = format!("{MODULE_EVT_ID}/modules/by-id/{}/recv", &module_id);

  match port_session
    .declare_publisher(&key_expr)
    .cache(CacheConfig::default().max_samples(1))
    .await
  {
    Ok(publisher) => {
      // If we don't fail out, this block should be the one that gets put in that new thread.

      while let Some(msg) = rx_channel.recv().await {
        match serde_json::to_string(&msg) {
          Ok(msg_str) => match publisher.put(&msg_str).await {
            Ok(_) => {
              debug!(
                "Sent message as {}, with len: {}",
                &module_id,
                msg_str.len()
              )
            }
            Err(err) => {
              error!("Failed to broadcast message contents, due to:\n:{err}");
            }
          },
          Err(err) => {
            error!("Failed to parse data from module into a BusMessage. Is the module connected properly? Due to:\n{err}");
          }
        }
      }
    }
    Err(err) => {
      // TODO: implement retries, otherwise fail out the UART bridge creation.
      error!(
        "Failed to create queryable; this is a bug and should be reported! This was due to:\n{err}"
      );
    } // see: rustc E0597 for why the semicolon is there??????
  };
}
