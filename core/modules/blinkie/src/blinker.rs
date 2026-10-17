use defmt::*;
use embassy_rp::gpio::Output;
use embassy_time::Timer;

#[embassy_executor::task]
pub async fn blinker_thread(mut led: Output<'static>) {
  info!("Starting Blinker Thread.");

  loop {
    info!("led on!");
    defmt::flush();
    led.set_high();
    Timer::after_millis(500).await;

    info!("led off!");
    defmt::flush();
    led.set_low();
    Timer::after_millis(500).await;
  }
}
