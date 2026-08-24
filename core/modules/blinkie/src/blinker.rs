use defmt::*;
use embassy_rp::{
  Peripherals,
  gpio::{
    Level,
    Output,
  },
};
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn blinker_thread(p: Peripherals) {
  let mut led = Output::new(p.PIN_13, Level::Low);

  loop {
    info!("led on!");
    led.set_high();
    Timer::after_millis(500).await;

    info!("led off!");
    led.set_low();
    Timer::after_millis(500).await;
  }
}
