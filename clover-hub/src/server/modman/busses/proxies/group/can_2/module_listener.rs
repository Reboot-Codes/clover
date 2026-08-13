use std::sync::Arc;

use linux_socketcan_iso_tp::TokioSocketCanIsoTp;
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::server::modman::busses::proxies::group::can_2::CAN2Bus;

// JK You thought this function actually did something lmaoooooo
#[instrument(skip(cancellation_token, raw_socket))]
pub async fn can_bus_listener(
  ctx: Arc<CAN2Bus>,
  cancellation_token: CancellationToken,
  module_id: String,
  raw_socket: TokioSocketCanIsoTp,
) {
  let socket = Arc::new(raw_socket);

  let rx_session = ctx.session.clone();
  let rx_token = cancellation_token.clone();
  let rx_socket = socket.clone();
  let rx_id = module_id.clone();
  tokio::task::spawn(async move {
    can_module_rx(rx_session, rx_token, rx_socket, rx_id).await;
  });

  let tx_session = ctx.session.clone();
  let tx_token = cancellation_token.clone();
  let tx_socket = socket.clone();
  let tx_id = module_id.clone();
  tokio::task::spawn(async move {
    can_module_tx(tx_session, tx_token, tx_socket, tx_id).await;
  });
}

#[instrument(skip(session, cancellation_token, socket))]
pub async fn can_module_rx(
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
  socket: Arc<TokioSocketCanIsoTp>,
  module_id: String,
) {
}

#[instrument(skip(session, cancellation_token, socket))]
pub async fn can_module_tx(
  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
  socket: Arc<TokioSocketCanIsoTp>,
  module_id: String,
) {
}
