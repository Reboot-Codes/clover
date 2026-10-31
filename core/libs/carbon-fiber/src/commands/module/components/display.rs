use embedded_graphics::prelude::DrawTarget;

use crate::commands::{
  models::components::display::DisplayComponent,
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

impl<I: Interpolation, Dt: DrawTarget> CommandComponent for DisplayComponent<I, Dt>
where
  Dt::Color: From<embedded_graphics::pixelcolor::raw::RawU16>,
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
      crate::commands::models::components::ComponentTarget::DisplayClear => {
        let _ = self.display.clear(Dt::Color::from(
          embedded_graphics::pixelcolor::raw::RawU16::new(0),
        ));
        self.dirty = true;
      }
      crate::commands::models::components::ComponentTarget::DisplayBlitChunk {
        offset: _,
        x,
        y,
        width,
        height,
        len,
        data,
      } => {
        let rect = embedded_graphics::primitives::Rectangle::new(
          embedded_graphics::geometry::Point::new(x as i32, y as i32),
          embedded_graphics::geometry::Size::new(width as u32, height as u32),
        );
        let valid_len = (len as usize).min(data.len());
        let colors = data[..valid_len].chunks_exact(2).map(|chunk| {
          Dt::Color::from(embedded_graphics::pixelcolor::raw::RawU16::new(
            u16::from_le_bytes([chunk[0], chunk[1]]),
          ))
        });
        let _ = self.display.fill_contiguous(&rect, colors);
        self.dirty = true;
      }
      _ => return Err(()),
    }

    let cmd = crate::commands::models::components::GestureTargetCommand {
      component_id: self.id,
      transition_ms,
      flags,
      target,
    };
    self.pipeline.push(cmd, tx_id).map(|_| ()).map_err(|_| ())
  }

  fn step(
    &mut self,
    dt_ms: u32,
  ) -> Option<(
    [u8; 16],
    crate::commands::models::components::ComponentTelemetry,
  )> {
    let event = self.pipeline.tick(dt_ms);
    if self.dirty {
      self.dirty = false;
    }

    match event {
      crate::commands::module::components::PipelineEvent::ReachedTarget { completed_tx }
      | crate::commands::module::components::PipelineEvent::PromotedNext { completed_tx } => {
        Some((
          completed_tx,
          crate::commands::models::components::ComponentTelemetry::DisplayReady,
        ))
      }
      _ => None,
    }
  }

  fn backtrack(&mut self, return_ms: u16) {
    self.pipeline.backtrack(return_ms);
  }
}

impl<I: Interpolation, Dt: DrawTarget> CommandComponentBuilder for DisplayComponent<I, Dt>
where
  Dt::Color: From<embedded_graphics::pixelcolor::raw::RawU16>,
{
  type WithInterpolator<NewI: Interpolation> = DisplayComponent<NewI, Dt>;

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    DisplayComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    DisplayComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    DisplayComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<crate::commands::module::interpolation::CubicInterpolation> {
    DisplayComponent {
      id: self.id,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }
}

impl<Dt: DrawTarget> DisplayComponent<LinearInterpolation, Dt> {
  /// Create a new Display Component with the given ID and display driver.
  pub fn new(id: u8, display: Dt) -> Self {
    DisplayComponent {
      id,
      interpolator: LinearInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
      display,
      dirty: false,
    }
  }
}

impl<I: Interpolation, Dt: DrawTarget> DisplayComponent<I, Dt> {
  pub fn interpolate_smooth(self) -> DisplayComponent<SmoothInterpolation, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }

  pub fn interpolate_linear(self) -> DisplayComponent<LinearInterpolation, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }

  pub fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> DisplayComponent<CubicInterpolation, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }

  pub fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> DisplayComponent<NewI, Dt> {
    DisplayComponent {
      id: self.id,
      interpolator,
      pipeline: self.pipeline,
      display: self.display,
      dirty: self.dirty,
    }
  }
}
