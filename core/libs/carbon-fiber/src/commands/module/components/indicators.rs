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
    }
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
    todo!()
  }

  fn step(
    &mut self,
    dt_ms: u32,
  ) -> Option<(
    [u8; 16],
    crate::commands::models::components::ComponentTelemetry,
  )> {
    todo!()
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
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      driver: self.driver,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    IndicatorComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      driver: self.driver,
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
