use crate::commands::{
  models::components::speaker::SpeakerComponent,
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

impl<I: Interpolation> CommandComponent for SpeakerComponent<I> {
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

impl<I: Interpolation> CommandComponentBuilder for SpeakerComponent<I> {
  type WithInterpolator<NewI: Interpolation> = SpeakerComponent<NewI>;

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    SpeakerComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
      ohms: self.ohms,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    SpeakerComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      ohms: self.ohms,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    SpeakerComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      ohms: self.ohms,
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
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      ohms: self.ohms,
    }
  }
}

impl SpeakerComponent<LinearInterpolation> {
  /// Create a new SpeakerComponent with the given ID.
  pub fn new(id: u8) -> Self {
    SpeakerComponent {
      id,
      interpolator: LinearInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
      ohms: None,
    }
  }
}

impl<I: Interpolation> SpeakerComponent<I> {
  /// Sets the expected resistance of the connected speaker enabling:
  /// - power draw estimates,
  /// - more accurate volume control,
  /// - and self-tests.
  pub fn ohms(self, ohms: u16) -> Self {
    SpeakerComponent {
      id: self.id,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      ohms: Some(ohms),
    }
  }

  /// Send samples to a connected I2C DAC/AMP.
  pub fn i2c_dac(self) -> Self {
    todo!()
  }

  /// Send samples to a connected SPI DAC/AMP.
  pub fn spi_dac(self) -> Self {
    todo!()
  }

  /// Modulate samples directly to a pin on the MCU.
  ///
  /// ***Note:*** this will be lower quality and take more processing time on the MCU.
  pub fn pin(self) -> Self {
    todo!()
  }
}
