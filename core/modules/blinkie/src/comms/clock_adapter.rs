use can_iso_tp::Clock;
use core::time::Duration;
use embassy_time::Instant;

#[derive(Clone, Copy, Default)]
pub struct EmbassyClock;

impl Clock for EmbassyClock {
  type Instant = Instant;

  fn now(&self) -> Self::Instant {
    Instant::now()
  }

  fn elapsed(&self, earlier: Self::Instant) -> Duration {
    let now_us = Instant::now().as_micros();
    let earlier_us = earlier.as_micros();
    // saturating_sub prevents underflow panics if called right at boundary edges
    Duration::from_micros(now_us.saturating_sub(earlier_us))
  }

  fn add(&self, instant: Self::Instant, dur: Duration) -> Self::Instant {
    instant + embassy_time::Duration::from_micros(dur.as_micros() as u64)
  }
}
