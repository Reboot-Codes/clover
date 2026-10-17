#![no_std]
#![no_main]

pub mod blinker;
pub mod comms;

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
  let bus = CAN2::new(p.SPI_0);

  let led_comp = IndicatorComponent::new(0)
    .neopixel_pin(p.GPIO_3, NeopixelConfig { num_pixels: 1, })
    // Linear transitions
    .interpolate_linear();

  let stage1_comp = MovementComponent::new(1)
    .tentacle(TentacleConfig {
      length: 0.6096,
      ..default()
    })
    .degree1_servo(p.GPIO_4)
    .degree2_servo(p.GPIO_5)
    // Default cubic transition smoothing curve
    .interpolate_smooth();

  let stage2_comp = MovementComponent::new(2)
    .tentacle(TentacleConfig {
      length: 0.6096,
      ..default()
    })
    .degree1_servo(p.GPIO_4)
    .degree2_servo(p.GPIO_5)
    // Component ID of where to set kinematics off of for inverse kinematic calculations.
    .kinematic_of(1)
    // Custom smoothing calcuations
    .interpolate_custom(|prev, curr, progress| {
      prev.x - curr.y * progress
    });

  let (comms_thread, module) = Module::init(bus, [led_comp, stage1_comp, stage2_comp]);

  core1_spawner.spawn(| | { comms_thread().await; });

  module.start();
}
