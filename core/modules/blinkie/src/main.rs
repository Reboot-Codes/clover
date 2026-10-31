#![no_std]
#![no_main]

pub mod blinker;
pub mod comms;

use carbon_fiber::commands::models::components::indicators::{
  BinaryIndicator,
  IndicatorComponent,
};
use carbon_fiber::module::Module;
use carbon_fiber::module::comms::ModuleChannels;
use core::cell::RefCell;
use core::ptr::addr_of_mut;
use defmt::*;
use defmt_rtt as _;
use embassy_executor::{
  Executor,
  Spawner,
};
use embassy_rp::gpio::{
  Input,
  Level,
  Output,
};
use embassy_rp::multicore::Stack;
use embassy_rp::{
  peripherals::SPI0,
  spi::Spi,
};
use embassy_sync::blocking_mutex::{
  Mutex,
  raw::CriticalSectionRawMutex,
};
use panic_probe as _;
use static_cell::StaticCell;

static mut CORE1_STACK: Stack<16384> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

use crate::{
  blinker::blinker_thread,
  comms::comms_thread,
};

pub type SpiBus =
  Mutex<CriticalSectionRawMutex, RefCell<Spi<'static, SPI0, embassy_rp::spi::Blocking>>>;

static CHANNELS: StaticCell<ModuleChannels<8, 16>> = StaticCell::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  let p = embassy_rp::init(Default::default());

  info!("Starting up com.reboot-codes.clover.CORE.blinky...");

  let spi_config = embassy_rp::spi::Config::default();
  let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_16, spi_config);
  static SPI_BUS: StaticCell<SpiBus> = StaticCell::new();
  let spi_bus = SPI_BUS.init(Mutex::new(RefCell::new(spi)));
  let mcp_int = Input::new(p.PIN_6, embassy_rp::gpio::Pull::Up);

  info!("Initing CORE1...");
  embassy_rp::multicore::spawn_core1(
    p.CORE1,
    unsafe { &mut *addr_of_mut!(CORE1_STACK) },
    move || {
      info!("Core 1 started! Creating EXECUTOR1...");
      let executor1 = EXECUTOR1.init(Executor::new());
      executor1.run(|spawner| {
        info!("Core 1 executor running! Spawning comms thread...");

        match spawner.spawn(comms_thread(spi_bus, p.PIN_9.into(), mcp_int)) {
          Ok(_) => {
            info!("Comms thread spawned successfully on Core 1!");
          }
          Err(err) => {
            error!(
              "Failed to spawn comms thread due to:\n{:?}",
              Debug2Format(&err)
            );
          }
        }
      })
    },
  );

  let channels = CHANNELS.init(ModuleChannels::new());

  let mut light_comp = IndicatorComponent::new(
    0,
    BinaryIndicator::new(Output::new(p.PIN_25, Level::low), false),
  );

  let (comms, module) = Module::new()
    .add_command_component(&mut light_comp)
    .build(None, channels);

  spawner
    .spawn(blinker_thread(Output::new(p.PIN_25, Level::Low)))
    .unwrap();
}
