//! Embassy runtime adapter for can-iso-tp.

#[cfg(feature = "embassy-time")]
mod embassy_impl {
  use can_iso_tp::{
    AsyncRuntime,
    Clock,
  };
  use core::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{
      Context,
      Poll,
    },
    time::Duration,
  };
  use embassy_time::{
    Duration as EmbassyDuration,
    Instant,
    TimeoutError,
    Timer,
  };

  pub struct EmbassyTimeout<'a, F> {
    future: F,
    timer: Timer,
    _phantom: PhantomData<&'a ()>,
  }

  impl<'a, F: Future> Future for EmbassyTimeout<'a, F> {
    type Output = Result<F::Output, TimeoutError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
      let this = unsafe { self.get_unchecked_mut() };
      let future = unsafe { Pin::new_unchecked(&mut this.future) };
      if let Poll::Ready(val) = future.poll(cx) {
        return Poll::Ready(Ok(val));
      }
      let timer = unsafe { Pin::new_unchecked(&mut this.timer) };
      if let Poll::Ready(()) = timer.poll(cx) {
        return Poll::Ready(Err(TimeoutError));
      }
      Poll::Pending
    }
  }

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
      Duration::from_micros(now_us.saturating_sub(earlier_us))
    }

    fn add(&self, instant: Self::Instant, dur: Duration) -> Self::Instant {
      instant + EmbassyDuration::from_micros(dur.as_micros() as u64)
    }
  }

  #[derive(Clone, Copy, Default)]
  pub struct EmbassyRuntime;

  impl AsyncRuntime for EmbassyRuntime {
    type TimeoutError = TimeoutError;

    type Sleep<'a>
      = Timer
    where
      Self: 'a;

    type Timeout<'a, F>
      = EmbassyTimeout<'a, F>
    where
      Self: 'a,
      F: Future + 'a;

    fn sleep(&self, duration: Duration) -> Self::Sleep<'_> {
      let micros = duration.as_micros() as u64;
      Timer::after(EmbassyDuration::from_micros(micros))
    }

    fn timeout<'a, F>(&'a self, duration: Duration, future: F) -> Self::Timeout<'a, F>
    where
      F: Future + 'a,
    {
      let dur = EmbassyDuration::from_micros(duration.as_micros() as u64);
      EmbassyTimeout {
        future,
        timer: Timer::after(dur),
        _phantom: PhantomData,
      }
    }
  }
}

#[cfg(feature = "embassy-time")]
pub use embassy_impl::*;
