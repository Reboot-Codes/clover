use crate::commands::{
  models::components::speaker::{
    I2cDacDriver,
    PwmPinDriver,
    SpeakerComponent,
    SpeakerDriver,
    SpiDacDriver,
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

impl<Dr: SpeakerDriver, I: Interpolation> CommandComponent for SpeakerComponent<Dr, I> {
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
      crate::commands::models::components::ComponentTarget::SpeakerTone { .. }
      | crate::commands::models::components::ComponentTarget::SpeakerSilence => {
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
        crate::commands::models::components::ComponentTarget::SpeakerTone {
          frequency_hz,
          volume_percent,
        } => {
          let (prev_freq, prev_vol) = match self.pipeline.prev_target.map(|p| p.target) {
            Some(crate::commands::models::components::ComponentTarget::SpeakerTone {
              frequency_hz: pf,
              volume_percent: pv,
            }) => (pf, pv),
            _ => (frequency_hz, volume_percent),
          };
          self.current_freq_hz =
            self.interpolator.interpolate_u16(prev_freq, frequency_hz, progress);
          self.current_volume =
            self.interpolator.interpolate_u8(prev_vol, volume_percent, progress);
          self.active = self.current_volume > 0 && self.current_freq_hz > 0;
          let _ = self.driver.set_tone(self.current_freq_hz, self.current_volume);
        }
        crate::commands::models::components::ComponentTarget::SpeakerSilence => {
          self.current_freq_hz = 0;
          self.current_volume = 0;
          self.active = false;
          let _ = self.driver.silence();
        }
        _ => {}
      }
    }

    match event {
      crate::commands::module::components::PipelineEvent::ReachedTarget { completed_tx }
      | crate::commands::module::components::PipelineEvent::PromotedNext { completed_tx } => {
        Some((
          completed_tx,
          crate::commands::models::components::ComponentTelemetry::SpeakerReady,
        ))
      }
      _ => None,
    }
  }

  fn backtrack(&mut self, return_ms: u16) {
    self.pipeline.backtrack(return_ms);
  }
}

impl<Dr: SpeakerDriver + Clone, I: Interpolation> CommandComponentBuilder for SpeakerComponent<Dr, I> {
  type WithInterpolator<NewI: Interpolation> = SpeakerComponent<Dr, NewI>;

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    SpeakerComponent {
      id: self.id,
      driver: self.driver,
      interpolator,
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    SpeakerComponent {
      id: self.id,
      driver: self.driver,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    SpeakerComponent {
      id: self.id,
      driver: self.driver,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<crate::commands::module::interpolation::CubicInterpolation> {
    SpeakerComponent {
      id: self.id,
      driver: self.driver,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }
}

impl SpeakerComponent<(), LinearInterpolation> {
  /// Create a new SpeakerComponent with the given ID.
  pub fn new(id: u8) -> Self {
    SpeakerComponent {
      id,
      driver: (),
      interpolator: LinearInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
      ohms: None,
      current_freq_hz: 0,
      current_volume: 0,
      active: false,
    }
  }
}

impl<Dr: SpeakerDriver, I: Interpolation> SpeakerComponent<Dr, I> {
  /// Sets the expected resistance of the connected speaker enabling:
  /// - power draw estimates,
  /// - more accurate volume control,
  /// - and self-tests.
  pub fn ohms(mut self, ohms: u16) -> Self {
    self.ohms = Some(ohms);
    self
  }

  /// Attach a connected I2C DAC/AMP on the given I2C bus and 7-bit address.
  pub fn i2c_dac<I2C: embedded_hal::i2c::I2c>(
    self,
    i2c: I2C,
    address: u8,
  ) -> SpeakerComponent<I2cDacDriver<I2C>, I> {
    SpeakerComponent {
      id: self.id,
      driver: I2cDacDriver { i2c, address },
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }

  /// Attach a connected SPI DAC/AMP on the given SPI device.
  pub fn spi_dac<SPI: embedded_hal::spi::SpiDevice>(
    self,
    spi: SPI,
  ) -> SpeakerComponent<SpiDacDriver<SPI>, I> {
    SpeakerComponent {
      id: self.id,
      driver: SpiDacDriver { spi },
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }

  /// Modulate samples directly to a PWM pin on the MCU.
  pub fn pin<P: embedded_hal::pwm::SetDutyCycle>(
    self,
    pwm: P,
  ) -> SpeakerComponent<PwmPinDriver<P>, I> {
    SpeakerComponent {
      id: self.id,
      driver: PwmPinDriver { pwm },
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      ohms: self.ohms,
      current_freq_hz: self.current_freq_hz,
      current_volume: self.current_volume,
      active: self.active,
    }
  }
}
