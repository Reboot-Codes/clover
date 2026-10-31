#![no_std]
#![no_main]

use core::cell::RefCell;
use core::ptr::addr_of_mut;
use defmt::*;
use defmt_rtt as _;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_executor::{
  Executor,
  Spawner,
};
use embassy_rp::{
  gpio::{
    Input,
    Level,
    Output,
    Pull,
  },
  multicore::Stack,
  peripherals::SPI0,
  spi::{
    Config as SpiConfig,
    Spi,
  },
};
use embassy_sync::blocking_mutex::{
  raw::CriticalSectionRawMutex,
  Mutex,
};
use panic_probe as _;
use static_cell::StaticCell;

use carbon_fiber::{
  commands::models::components::indicators::{
    BinaryIndicator,
    IndicatorComponent,
  },
  module::{
    comms::{
      can2::{
        Mcp2515Driver,
        CAN2,
      },
      CommsWorker,
    },
    Module,
  },
};

type SpiBus =
  Mutex<CriticalSectionRawMutex, RefCell<Spi<'static, SPI0, embassy_rp::spi::Blocking>>>;

type BlinkieSpiDevice = SpiDevice<
  'static,
  CriticalSectionRawMutex,
  Spi<'static, SPI0, embassy_rp::spi::Blocking>,
  Output<'static>,
>;

type BlinkieComms =
  CommsWorker<'static, CAN2<Mcp2515Driver<BlinkieSpiDevice, Input<'static>>>>;

static mut CORE1_STACK: Stack<16384> = Stack::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();
static SPI_BUS: StaticCell<SpiBus> = StaticCell::new();

#[embassy_executor::task]
async fn comms_task(mut comms: BlinkieComms) {
  comms.run().await;
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
  let p = embassy_rp::init(Default::default());

  info!("Starting up com.reboot-codes.clover.CORE.blinky...");

  let spi = Spi::new_blocking(p.SPI0, p.PIN_18, p.PIN_19, p.PIN_16, SpiConfig::default());
  let spi_bus = SPI_BUS.init(Mutex::new(RefCell::new(spi)));
  let mcp_cs = Output::new(p.PIN_9, Level::High);
  let mcp_spi = SpiDevice::new(spi_bus, mcp_cs);
  let mcp_int = Input::new(p.PIN_6, Pull::Up);

  let can2 = CAN2::mcp2515(mcp_spi, mcp_int).expect("Failed to initialize CAN2 MCP2515");

  let mut light_comp = IndicatorComponent::new(
    0,
    BinaryIndicator::new(Output::new(p.PIN_25, Level::Low), false),
  );

  let (comms, mut module) = Module::new("blinky", "com.reboot-codes.clover.CORE.blinky")
    .add_command_component(&mut light_comp)
    .build(can2);

  embassy_rp::multicore::spawn_core1(
    p.CORE1,
    unsafe { &mut *addr_of_mut!(CORE1_STACK) },
    move || {
      let executor1 = EXECUTOR1.init(Executor::new());
      executor1.run(|spawner| {
        spawner.spawn(comms_task(comms)).unwrap();
      });
    },
  );

  module.run().await;
}
