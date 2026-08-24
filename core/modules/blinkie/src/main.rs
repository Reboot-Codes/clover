#![no_std]
#![no_main]

pub mod blinker;
pub mod comms;

use defmt::*;
use defmt_rtt as _;
use embassy_executor::Spawner;
use panic_probe as _;

use crate::{
  blinker::blinker_thread,
  comms::comms_thread,
};

#[embassy_executor::main(executor = "embassy_executor::Executor", entry = "cortex_m_rt::entry")]
async fn main(spawner: Spawner) {
  info!("Starting up com.reboot-codes.clover.CORE.blinky...");

  let p = embassy_rp::init(Default::default());

  match spawner.spawn(comms_thread()) {
    Ok(_) => match spawner.spawn(blinker_thread(p)) {
      Ok(_) => {
        info!("Module started!");
      }
      Err(err) => {
        error!("Failed to start blinker thread!");
      }
    },
    Err(_) => {}
  }
}
