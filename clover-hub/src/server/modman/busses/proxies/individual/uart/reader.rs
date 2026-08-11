use serde::de::DeserializeOwned;
use tokio::{
  io::ReadHalf,
  sync::mpsc::UnboundedSender,
};
use tokio_serial::SerialStream;
use tokio_util::io::SyncIoBridge;

pub fn uart_reader<Msg>(
  mut bridge: SyncIoBridge<ReadHalf<SerialStream>>,
  channel: UnboundedSender<Msg>,
) where
  Msg: DeserializeOwned,
{
  let mut should_read = true;

  while should_read {
    match rmp_serde::from_read::<_, Msg>(&mut bridge) {
      Ok(msg) => {}
      Err(err) => todo!(),
    }
  }
}
