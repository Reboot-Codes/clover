use core::fmt::Debug;
use embedded_hal::pwm::{
  Error as PwmError,
  ErrorKind,
};
use embedded_hal::{
  digital::OutputPin,
  pwm::SetDutyCycle,
};
use smart_leds::{
  RGB,
  SmartLedsWriteAsync,
};
use snafu::Snafu;

use crate::commands::models::components::indicators::NeoPixelSet;
use crate::commands::{
  models::components::indicators::{
    BinaryIndicator,
    DimmerIndicator,
    IndicatorComponent,
    IndicatorTarget,
    RGBIndicator,
  },
  module::{
    CommandComponent,
    CommandComponentBuilder,
    components::CommandComponentPipeline,
    interpolation::{
      CubicInterpolation,
      Interpolation,
      LinearInterpolation,
      SmoothInterpolation,
    },
  },
};

impl<D> IndicatorComponent<D, LinearInterpolation>
where
  D: for<'a> IndicatorDriver<'a>,
{
  /// Create a new Indicator with the given ID and [IndicatorDriver].
  pub fn new(id: u8, driver: D) -> Self {
    IndicatorComponent {
      id,
      interpolator: LinearInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
      driver,
      brightness: 255,
    }
  }

  /// Sets global brightness scaling (0..255).
  pub fn with_brightness(mut self, brightness: u8) -> Self {
    self.brightness = brightness;
    self
  }
}

impl<D: for<'a> IndicatorDriver<'a>, I: Interpolation> CommandComponent
  for IndicatorComponent<D, I>
{
  fn id(&self) -> u8 {
    self.id
  }

  fn on_target(
    &mut self,
    target: crate::commands::models::components::ComponentTarget,
    transition_ms: u16,
    flags: Option<crate::commands::models::components::GestureFlags>,
    tx_id: [u8; 16],
  ) -> Result<(), ()> {
    match target {
      crate::commands::models::components::ComponentTarget::IndicatorBinary(_)
      | crate::commands::models::components::ComponentTarget::IndicatorDimmer(_)
      | crate::commands::models::components::ComponentTarget::IndicatorRGB(_, _, _)
      | crate::commands::models::components::ComponentTarget::IndicatorNeoPixels { .. } => {
        let cmd = crate::commands::models::components::GestureTargetCommand {
          component_id: self.id,
          transition_ms,
          flags,
          target,
        };
        self.pipeline.push(cmd, tx_id).map(|_| ()).map_err(|_| ())
      }
      _ => Err(()),
    }
  }

  fn step(
    &mut self,
    dt_ms: u32,
  ) -> Option<(
    [u8; 16],
    crate::commands::models::components::ComponentTelemetry,
  )> {
    let event = self.pipeline.tick(dt_ms);

    let active_target = match self.pipeline.current_target {
      Some(c) => Some((c, false)),
      None => {
        if matches!(
          event,
          crate::commands::module::components::PipelineEvent::ReachedTarget { .. }
        ) {
          self.pipeline.prev_target.map(|p| (p, true))
        } else {
          None
        }
      }
    };

    if let Some((current, is_completed)) = active_target {
      let progress: u16 = if is_completed || current.transition_ms == 0 {
        u16::MAX
      } else {
        let elapsed = ((self.pipeline.elapsed_ms as u64 * 65535)
          + (current.transition_ms as u64 / 2))
          / current.transition_ms as u64;
        elapsed.min(65535) as u16
      };

      match current.target {
        crate::commands::models::components::ComponentTarget::IndicatorBinary(b) => {
          let scaled = if self.brightness == 0 { false } else { b };
          let waker = core::task::Waker::noop();
          let mut cx = core::task::Context::from_waker(&waker);
          let mut fut = core::pin::pin!(self.driver.set_output(IndicatorTarget::Binary(scaled)));
          let _ = fut.as_mut().poll(&mut cx);
        }
        crate::commands::models::components::ComponentTarget::IndicatorDimmer(val) => {
          let prev_val = match self.pipeline.prev_target.map(|p| p.target) {
            Some(crate::commands::models::components::ComponentTarget::IndicatorDimmer(pv)) => pv,
            _ => 0,
          };
          let interp = self.interpolator.interpolate_u8(prev_val, val, progress);
          let scaled = ((interp as u16 * self.brightness as u16) / 255) as u8;
          let waker = core::task::Waker::noop();
          let mut cx = core::task::Context::from_waker(&waker);
          let mut fut = core::pin::pin!(self.driver.set_output(IndicatorTarget::Dimmer(scaled)));
          let _ = fut.as_mut().poll(&mut cx);
        }
        crate::commands::models::components::ComponentTarget::IndicatorRGB(r, g, b) => {
          let (pr, pg, pb) = match self.pipeline.prev_target.map(|p| p.target) {
            Some(crate::commands::models::components::ComponentTarget::IndicatorRGB(
              pr,
              pg,
              pb,
            )) => (pr, pg, pb),
            _ => (0, 0, 0),
          };
          let ir = self.interpolator.interpolate_u8(pr, r, progress);
          let ig = self.interpolator.interpolate_u8(pg, g, progress);
          let ib = self.interpolator.interpolate_u8(pb, b, progress);
          let sr = ((ir as u16 * self.brightness as u16) / 255) as u8;
          let sg = ((ig as u16 * self.brightness as u16) / 255) as u8;
          let sb = ((ib as u16 * self.brightness as u16) / 255) as u8;
          let waker = core::task::Waker::noop();
          let mut cx = core::task::Context::from_waker(&waker);
          let mut fut = core::pin::pin!(self.driver.set_output(IndicatorTarget::RGB(sr, sg, sb)));
          let _ = fut.as_mut().poll(&mut cx);
        }
        crate::commands::models::components::ComponentTarget::IndicatorNeoPixels {
          count,
          colors,
        } => {
          let mut scaled_pixels = [smart_leds::RGB8::new(0, 0, 0); 8];
          let valid_count = (count as usize).min(8);
          let prev_colors = match self.pipeline.prev_target.map(|p| p.target) {
            Some(crate::commands::models::components::ComponentTarget::IndicatorNeoPixels {
              count: pc,
              colors: pcols,
            }) => Some((pc as usize, pcols)),
            _ => None,
          };
          for i in 0..valid_count {
            let (target_r, target_g, target_b) = colors[i];
            let (pr, pg, pb) = match prev_colors {
              Some((pc, pcols)) if i < pc => pcols[i],
              _ => (0, 0, 0),
            };
            let ir = self.interpolator.interpolate_u8(pr, target_r, progress);
            let ig = self.interpolator.interpolate_u8(pg, target_g, progress);
            let ib = self.interpolator.interpolate_u8(pb, target_b, progress);
            let sr = ((ir as u16 * self.brightness as u16) / 255) as u8;
            let sg = ((ig as u16 * self.brightness as u16) / 255) as u8;
            let sb = ((ib as u16 * self.brightness as u16) / 255) as u8;
            scaled_pixels[i] = smart_leds::RGB8::new(sr, sg, sb);
          }
          let waker = core::task::Waker::noop();
          let mut cx = core::task::Context::from_waker(&waker);
          let mut fut = core::pin::pin!(
            self
              .driver
              .set_output(IndicatorTarget::NeoPixels(&scaled_pixels[..valid_count]))
          );
          let _ = fut.as_mut().poll(&mut cx);
        }
        _ => {}
      }
    }

    match event {
      crate::commands::module::components::PipelineEvent::ReachedTarget { completed_tx }
      | crate::commands::module::components::PipelineEvent::PromotedNext { completed_tx } => {
        Some((
          completed_tx,
          crate::commands::models::components::ComponentTelemetry::IndicatorReady,
        ))
      }
      _ => None,
    }
  }

  fn backtrack(&mut self, return_ms: u16) {
    self.pipeline.backtrack(return_ms);
  }
}

impl<D: for<'a> IndicatorDriver<'a>, I: Interpolation> CommandComponentBuilder
  for IndicatorComponent<D, I>
{
  type WithInterpolator<NewI: Interpolation> = IndicatorComponent<D, NewI>;

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    IndicatorComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
      driver: self.driver,
      brightness: self.brightness,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      driver: self.driver,
      brightness: self.brightness,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      driver: self.driver,
      brightness: self.brightness,
    }
  }

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<crate::commands::module::interpolation::CubicInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      driver: self.driver,
      brightness: self.brightness,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IndicatorKind {
  Binary,
  Dimmer,
  RGB,
  NeoPixel,
  Other,
}

#[derive(Debug, Snafu, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IndicatorError<PE> {
  #[snafu(display("Invalid target variant {actual:?} for indicator type {expected:?}."))]
  MismatchedTarget {
    expected: IndicatorKind,
    actual: IndicatorKind,
  },

  #[snafu(display("Underlying driver write failed."))]
  DriverWriteFault { err: PE },

  #[snafu(display("Invalid command length of {cmd_len} for NeoPixel set length of {set_len}."))]
  PixelCountExceeded { cmd_len: usize, set_len: usize },
}

pub trait IndicatorDriver<'a> {
  type PError;

  /// Apply the interpolated color/intensity to the physical hardware
  fn set_output(
    &mut self,
    target: IndicatorTarget<'a>,
  ) -> impl Future<Output = Result<(), IndicatorError<Self::PError>>>;
}

impl<P: OutputPin> BinaryIndicator<P> {
  /// Create a new Binary Indicator, and if it's state should be inverted, for use in an [IndicatorComponent]
  pub fn new(pin: P, inverted: bool) -> Self {
    BinaryIndicator { pin, inverted }
  }
}

impl<'a, P: OutputPin> IndicatorDriver<'a> for BinaryIndicator<P> {
  type PError = P::Error;

  async fn set_output(
    &mut self,
    target: IndicatorTarget<'a>,
  ) -> Result<(), IndicatorError<Self::PError>> {
    match target {
      IndicatorTarget::Binary(state) => match state {
        true => match self.pin.set_high() {
          Ok(_) => Ok(()),
          Err(err) => Err(IndicatorError::DriverWriteFault { err }),
        },
        false => match self.pin.set_low() {
          Ok(_) => Ok(()),
          Err(err) => Err(IndicatorError::DriverWriteFault { err }),
        },
      },
      IndicatorTarget::Dimmer(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Binary,
        actual: IndicatorKind::Dimmer,
      }),
      IndicatorTarget::RGB(_, _, _) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Binary,
        actual: IndicatorKind::RGB,
      }),
      IndicatorTarget::NeoPixels(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Binary,
        actual: IndicatorKind::NeoPixel,
      }),
    }
  }
}

impl<P: SetDutyCycle> DimmerIndicator<P> {
  /// Create a new Dimmer Indicator for use in an [IndicatorComponent].
  pub fn new(pin: P) -> Self {
    DimmerIndicator { pin }
  }
}

impl<'a, P: SetDutyCycle> IndicatorDriver<'a> for DimmerIndicator<P> {
  type PError = P::Error;

  async fn set_output(
    &mut self,
    target: IndicatorTarget<'a>,
  ) -> Result<(), IndicatorError<Self::PError>> {
    match target {
      IndicatorTarget::Binary(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Dimmer,
        actual: IndicatorKind::Binary,
      }),
      IndicatorTarget::Dimmer(state) => match self.pin.set_duty_cycle_percent(state) {
        Ok(_) => Ok(()),
        Err(err) => Err(IndicatorError::DriverWriteFault { err }),
      },
      IndicatorTarget::RGB(_, _, _) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Dimmer,
        actual: IndicatorKind::RGB,
      }),
      IndicatorTarget::NeoPixels(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Binary,
        actual: IndicatorKind::NeoPixel,
      }),
    }
  }
}

impl<RP: SetDutyCycle, GP: SetDutyCycle, BP: SetDutyCycle> RGBIndicator<RP, GP, BP> {
  /// Create a new RGB Indicator for use in an [IndicatorComponent].
  pub fn new(r_pin: RP, g_pin: GP, b_pin: BP) -> Self {
    RGBIndicator {
      r_pin,
      g_pin,
      b_pin,
    }
  }
}

impl<'a, RP: SetDutyCycle, GP: SetDutyCycle, BP: SetDutyCycle> IndicatorDriver<'a>
  for RGBIndicator<RP, GP, BP>
{
  type PError = ErrorKind;

  async fn set_output(
    &mut self,
    target: IndicatorTarget<'a>,
  ) -> Result<(), IndicatorError<Self::PError>> {
    match target {
      IndicatorTarget::Binary(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::RGB,
        actual: IndicatorKind::Binary,
      }),
      IndicatorTarget::Dimmer(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::RGB,
        actual: IndicatorKind::Dimmer,
      }),
      IndicatorTarget::RGB(r, g, b) => {
        self
          .r_pin
          .set_duty_cycle_percent(r)
          .map_err(|e| IndicatorError::DriverWriteFault { err: e.kind() })?;

        self
          .g_pin
          .set_duty_cycle_percent(g)
          .map_err(|e| IndicatorError::DriverWriteFault { err: e.kind() })?;

        self
          .b_pin
          .set_duty_cycle_percent(b)
          .map_err(|e| IndicatorError::DriverWriteFault { err: e.kind() })?;

        Ok(())
      }
      IndicatorTarget::NeoPixels(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::Binary,
        actual: IndicatorKind::NeoPixel,
      }),
    }
  }
}

impl<Dr: SmartLedsWriteAsync, const MAXNEOPIXELS: usize> NeoPixelSet<Dr, MAXNEOPIXELS>
where
  for<'b> Dr::Color: From<&'b RGB<u8>>,
{
  /// Create a new NeoPixel Set for use in an [IndicatorComponent].
  pub fn new(driver: Dr) -> Self {
    NeoPixelSet {
      pixels: [RGB::new(0, 0, 0); MAXNEOPIXELS],
      driver,
    }
  }
}

impl<'a, Dr, const MAXNEOPIXELS: usize> IndicatorDriver<'a> for NeoPixelSet<Dr, MAXNEOPIXELS>
where
  Dr: SmartLedsWriteAsync,
  for<'b> Dr::Color: From<&'b RGB<u8>>,
{
  type PError = Dr::Error;

  async fn set_output(
    &mut self,
    target: IndicatorTarget<'a>,
  ) -> Result<(), IndicatorError<Self::PError>> {
    match target {
      IndicatorTarget::Binary(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::NeoPixel,
        actual: IndicatorKind::Binary,
      }),
      IndicatorTarget::Dimmer(_) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::NeoPixel,
        actual: IndicatorKind::Dimmer,
      }),
      IndicatorTarget::RGB(_, _, _) => Err(IndicatorError::MismatchedTarget {
        expected: IndicatorKind::NeoPixel,
        actual: IndicatorKind::RGB,
      }),
      IndicatorTarget::NeoPixels(state) => {
        if state.len() > MAXNEOPIXELS {
          Err(IndicatorError::PixelCountExceeded {
            set_len: MAXNEOPIXELS,
            cmd_len: state.len(),
          })
        } else {
          self
            .driver
            .write(state.iter())
            .await
            .map_err(|err| IndicatorError::DriverWriteFault { err })
        }
      }
    }
  }
}
