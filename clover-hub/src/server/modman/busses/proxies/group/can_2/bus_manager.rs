use std::sync::Arc;

use can_iso_tp::{
  self,
  IsoTpNode,
};
use embedded_can::Id;
use linux_socketcan_iso_tp::{
  self,
  IsoTpKernelOptions,
  TokioSocketCanIsoTp,
};
use tokio::sync::{
  broadcast::{
    channel as broadcast_channel,
    Receiver as BroadcastReceiver,
    Sender as BroadcastSender,
  },
  oneshot::{
    channel as oneshot_channel,
    Receiver as OneShotReceiver,
    Sender as OneShotSender,
  },
};
use tokio_util::sync::CancellationToken;
use tracing::instrument;

use crate::server::modman::busses::{
  models::BusMessage,
  proxies::group::can_2::CAN2Bus,
};

#[instrument(skip(ctx, cancellation_token))]
pub async fn can_bus_manager(
  ctx: Arc<CAN2Bus>,
  cancellation_token: CancellationToken,
  iface_details: (String, u32),
) {
  let (iface_name, iface_index) = iface_details;

  // Modules shouldn't be sending events over CAN before we introduce ourselves, but we have a buffer just in case.
  let (to_bus_tx, to_bus) = broadcast_channel(16);
  let (from_bus, from_bus_rx) = broadcast_channel(16);

  // I'd rather return the JoinHandle from the function, but due to using libc, we should avoid doing that.
  let (status_channel, status_channel_rx) = oneshot_channel();

  tokio::task::spawn(async move {
    can_bus_listener(
      to_bus,
      from_bus,
      cancellation_token.clone(),
      status_channel,
      // We recreate the iface_details tuple due to an error created by the instrument macro.
      (iface_name.clone(), iface_index),
    )
    .await
  });

  match status_channel_rx.await {
    Ok(_) => while !ctx.cancellation_token.is_cancelled() {},
    Err(err) => todo!(),
  }
}

#[instrument(skip(to_bus, from_bus, cancellation_token))]
pub async fn can_bus_listener(
  to_bus: BroadcastReceiver<BusMessage>,
  from_bus: BroadcastSender<BusMessage>,
  cancellation_token: CancellationToken,
  status_channel: OneShotSender<Result<(), anyhow::Error>>,
  iface_details: (String, u32),
) {
  let options = IsoTpKernelOptions::default();
  let (iface_name, _iface_index) = iface_details;

  match TokioSocketCanIsoTp::open(
    &iface_name,
    Id::Standard(embedded_can::StandardId::new(0x101).expect("0x101 is a valid standard CAN ID")),
    Id::Standard(embedded_can::StandardId::new(0x201).expect("0x201 is a valid standard CAN ID")),
    &options,
  ) {
    Ok(socket) => {
      
    }
    Err(err) => todo!(),
  }
}
