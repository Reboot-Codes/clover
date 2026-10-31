//! # Indicator Command Targets
//!
//! Indicators control arbitrary GPIOs and basic sets of Smart LEDs.
//! For anything more complicated, you probably want a [Display Component](super::display::DisplayComponent) backed by a (SmartLED Matrix)[smart_led_matrix::SmartLedMatrix].
//!
//! When composing the component for a module, pick one of the driver types (or write one that implements [IndicatorDriver]):
//!
//! - [BinaryIndicator] - On or off with support for inversion.
//! - [DimmerIndicator] - PWM duty-cycle dimming of a specific pin.
//! - [RGBIndicator] - 3 Dimmers assigned to the u8 RGB color space.
//! - [NeoPixelSet] - An array of NeoPixels which accept u8 RGB color space commands (1 per pixel).
//!
//! Then pass that into the [IndicatorComponent::new] method:
//!
//! ```rust,ignore
//! let mut component = IndicatorComponent::new(0,
//!   // Should bind to the default LED on a Pi Pico
//!   BinaryIndicator::new(Output::new(p.PIN_25, Level::Low), false)
//! );
//! ```
//!

use embedded_hal::{
  digital::OutputPin,
  pwm::SetDutyCycle,
};
use smart_leds::{
  RGB8,
  SmartLedsWriteAsync,
};

use crate::commands::module::{
  components::{
    CommandComponentPipeline,
    indicators::IndicatorDriver,
  },
  interpolation::Interpolation,
};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IndicatorTarget<'a, const MAXPIXELS: usize = 1> {
  /// If the line should be high or low.
  Binary(bool),
  /// Brightness percentage (adjusted for human visual estimates)
  Dimmer(u8),
  /// Parameters in RGB order
  RGB(u8, u8, u8),
  /// New NeoPixel Set state
  NeoPixels(&'a [RGB8]),
}

/// This component type is used to control arbitrary GPIO pins, and to control sets of smart leds.
///
/// ```rust,ignore
/// let mut component = IndicatorComponent::new(0,
///   BinaryIndicator::new(Output::new(p.PIN_25, Level::Low), false)
/// );
/// ```
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct IndicatorComponent<D: for<'a> IndicatorDriver<'a>, I: Interpolation> {
  pub id: u8,
  pub interpolator: I,
  pub pipeline: CommandComponentPipeline,
  pub driver: D,
  pub brightness: u8,
}

/// Indicator type which is either on or off.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BinaryIndicator<P: OutputPin> {
  pub pin: P,
  pub inverted: bool,
}

/// PWM dimmer type.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct DimmerIndicator<P: SetDutyCycle> {
  pub pin: P,
}

/// Basically 3 dimmers in one type that accepts a [Color](smart_leds::RGB).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RGBIndicator<RP: SetDutyCycle, GP: SetDutyCycle, BP: SetDutyCycle> {
  pub r_pin: RP,
  pub g_pin: GP,
  pub b_pin: BP,
}

/// A set of smart LEDs
#[derive(Debug)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct NeoPixelSet<Dr: SmartLedsWriteAsync, const MAXPIXELS: usize = 1> {
  pub pixels: [RGB8; MAXPIXELS],
  pub driver: Dr,
}
