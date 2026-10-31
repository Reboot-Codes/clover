use crate::commands::module::{
  components::CommandComponentPipeline,
  interpolation::Interpolation,
};

/// Play audio embeded in the module firmware/fs.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioCueTarget {}

/// Raw audio chunk from the comms bus.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AudioChunkTarget {}

/// Driver trait for speaker audio output backends.
pub trait SpeakerDriver {
  type Error: core::fmt::Debug;

  /// Update the output frequency (Hz) and volume percentage (0..100).
  fn set_tone(&mut self, frequency_hz: u16, volume_percent: u8) -> Result<(), Self::Error>;

  /// Silence audio output.
  fn silence(&mut self) -> Result<(), Self::Error>;
}

impl SpeakerDriver for () {
  type Error = core::convert::Infallible;

  fn set_tone(&mut self, _frequency_hz: u16, _volume_percent: u8) -> Result<(), Self::Error> {
    Ok(())
  }

  fn silence(&mut self) -> Result<(), Self::Error> {
    Ok(())
  }
}

/// Speaker driver sending tone & volume commands to an I2C DAC/AMP.
pub struct I2cDacDriver<I2C> {
  pub i2c: I2C,
  pub address: u8,
}

impl<I2C: embedded_hal::i2c::I2c> SpeakerDriver for I2cDacDriver<I2C> {
  type Error = I2C::Error;

  fn set_tone(&mut self, frequency_hz: u16, volume_percent: u8) -> Result<(), Self::Error> {
    let freq_bytes = frequency_hz.to_be_bytes();
    let payload = [0x01, volume_percent, freq_bytes[0], freq_bytes[1]];
    self.i2c.write(self.address, &payload)
  }

  fn silence(&mut self) -> Result<(), Self::Error> {
    let payload = [0x01, 0, 0, 0];
    self.i2c.write(self.address, &payload)
  }
}

/// Speaker driver sending tone & volume commands to an SPI DAC/AMP.
pub struct SpiDacDriver<SPI> {
  pub spi: SPI,
}

impl<SPI: embedded_hal::spi::SpiDevice> SpeakerDriver for SpiDacDriver<SPI> {
  type Error = SPI::Error;

  fn set_tone(&mut self, frequency_hz: u16, volume_percent: u8) -> Result<(), Self::Error> {
    let freq_bytes = frequency_hz.to_be_bytes();
    let payload = [0x01, volume_percent, freq_bytes[0], freq_bytes[1]];
    self.spi.write(&payload)
  }

  fn silence(&mut self) -> Result<(), Self::Error> {
    let payload = [0x01, 0, 0, 0];
    self.spi.write(&payload)
  }
}

/// Speaker driver driving PWM duty cycle on a microcontroller pin.
pub struct PwmPinDriver<P> {
  pub pwm: P,
}

impl<P: embedded_hal::pwm::SetDutyCycle> SpeakerDriver for PwmPinDriver<P> {
  type Error = P::Error;

  fn set_tone(&mut self, _frequency_hz: u16, volume_percent: u8) -> Result<(), Self::Error> {
    let max_duty = self.pwm.max_duty_cycle();
    let duty = ((max_duty as u32 * volume_percent as u32) / 100) as u16;
    self.pwm.set_duty_cycle(duty)
  }

  fn silence(&mut self) -> Result<(), Self::Error> {
    self.pwm.set_duty_cycle_fully_off()
  }
}

/// Component to output audio anywhere.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SpeakerComponent<Dr, I: Interpolation> {
  pub id: u8,
  pub driver: Dr,
  pub interpolator: I,
  pub pipeline: CommandComponentPipeline,
  pub ohms: Option<u16>,
  pub current_freq_hz: u16,
  pub current_volume: u8,
  pub active: bool,
}
