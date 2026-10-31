use crate::commands::{
  models::components::movement::{
    ActuatorDriver,
    MovementComponent,
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

impl MovementComponent<(), (), (), (), (), (), SmoothInterpolation> {
  pub const fn new(id: u8) -> Self {
    Self {
      id,
      actuator1: None,
      actuator2: None,
      actuator3: None,
      actuator4: None,
      actuator5: None,
      actuator6: None,
      interpolator: SmoothInterpolation::new(),
      pipeline: CommandComponentPipeline::new(),
      positions: [0; 6],
    }
  }
}

impl<A1, A2, A3, A4, A5, A6, I: Interpolation> MovementComponent<A1, A2, A3, A4, A5, A6, I> {
  /// Set the Actuator for the 1st degree of freedom (x).
  pub fn set_degree1<NewP1>(self, pin: NewP1) -> MovementComponent<NewP1, A2, A3, A4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: Some(pin),
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  /// Set the Actuator for the 2nd degree of freedom (y).
  pub fn set_degree2<NewP2>(self, pin: NewP2) -> MovementComponent<A1, NewP2, A3, A4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: Some(pin),
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  /// Set the Actuator for the 3rd degree of freedom (z).
  pub fn set_degree3<NewP3>(self, pin: NewP3) -> MovementComponent<A1, A2, NewP3, A4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: Some(pin),
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  /// Set the Actuator for the 4th degree of freedom (u).
  pub fn set_degree4<NewP4>(self, pin: NewP4) -> MovementComponent<A1, A2, A3, NewP4, A5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: Some(pin),
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  /// Set the Actuator for the 5th degree of freedom (v).
  pub fn set_degree5<NewP5>(self, pin: NewP5) -> MovementComponent<A1, A2, A3, A4, NewP5, A6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: Some(pin),
      actuator6: self.actuator6,
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  /// Set the Actuator for the 6th degree of freedom (w).
  pub fn set_degree6<NewP6>(self, pin: NewP6) -> MovementComponent<A1, A2, A3, A4, A5, NewP6, I> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: Some(pin),
      interpolator: self.interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }
}

impl<P1, P2, P3, P4, P5, P6, I: Interpolation> CommandComponent
  for MovementComponent<P1, P2, P3, P4, P5, P6, I>
where
  P1: ActuatorDriver,
  P2: ActuatorDriver,
  P3: ActuatorDriver,
  P4: ActuatorDriver,
  P5: ActuatorDriver,
  P6: ActuatorDriver,
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
      crate::commands::models::components::ComponentTarget::Movement(_) => {
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

      if let crate::commands::models::components::ComponentTarget::Movement(target_dofs) =
        current.target
      {
        let prev_dofs = if is_completed {
          self.positions
        } else {
          match self.pipeline.prev_target.map(|p| p.target) {
            Some(crate::commands::models::components::ComponentTarget::Movement(pt)) => match pt {
              crate::commands::models::components::movement::MovementTarget::OneDof(a) => {
                [a, 0, 0, 0, 0, 0]
              }
              crate::commands::models::components::movement::MovementTarget::TwoDof(a, b) => {
                [a, b, 0, 0, 0, 0]
              }
              crate::commands::models::components::movement::MovementTarget::ThreeDof(a, b, c) => {
                [a, b, c, 0, 0, 0]
              }
              crate::commands::models::components::movement::MovementTarget::FourDof(a, b, c, d) => {
                [a, b, c, d, 0, 0]
              }
              crate::commands::models::components::movement::MovementTarget::FiveDof(
                a,
                b,
                c,
                d,
                e,
              ) => [a, b, c, d, e, 0],
              crate::commands::models::components::movement::MovementTarget::SixDof(
                a,
                b,
                c,
                d,
                e,
                f,
              ) => [a, b, c, d, e, f],
            },
            _ => self.positions,
          }
        };

        let target_arr = match target_dofs {
          crate::commands::models::components::movement::MovementTarget::OneDof(a) => {
            [a, 0, 0, 0, 0, 0]
          }
          crate::commands::models::components::movement::MovementTarget::TwoDof(a, b) => {
            [a, b, 0, 0, 0, 0]
          }
          crate::commands::models::components::movement::MovementTarget::ThreeDof(a, b, c) => {
            [a, b, c, 0, 0, 0]
          }
          crate::commands::models::components::movement::MovementTarget::FourDof(a, b, c, d) => {
            [a, b, c, d, 0, 0]
          }
          crate::commands::models::components::movement::MovementTarget::FiveDof(a, b, c, d, e) => {
            [a, b, c, d, e, 0]
          }
          crate::commands::models::components::movement::MovementTarget::SixDof(
            a,
            b,
            c,
            d,
            e,
            f,
          ) => [a, b, c, d, e, f],
        };

        for i in 0..6 {
          self.positions[i] =
            self.interpolator.interpolate_u32(prev_dofs[i], target_arr[i], progress);
        }

        if let Some(ref mut a) = self.actuator1 {
          let _ = a.drive(self.positions[0]);
        }
        if let Some(ref mut a) = self.actuator2 {
          let _ = a.drive(self.positions[1]);
        }
        if let Some(ref mut a) = self.actuator3 {
          let _ = a.drive(self.positions[2]);
        }
        if let Some(ref mut a) = self.actuator4 {
          let _ = a.drive(self.positions[3]);
        }
        if let Some(ref mut a) = self.actuator5 {
          let _ = a.drive(self.positions[4]);
        }
        if let Some(ref mut a) = self.actuator6 {
          let _ = a.drive(self.positions[5]);
        }
      }
    }

    match event {
      crate::commands::module::components::PipelineEvent::ReachedTarget { completed_tx }
      | crate::commands::module::components::PipelineEvent::PromotedNext { completed_tx } => {
        Some((
          completed_tx,
          crate::commands::models::components::ComponentTelemetry::Movement {
            position: self.positions,
          },
        ))
      }
      _ => None,
    }
  }

  fn backtrack(&mut self, return_ms: u16) {
    self.pipeline.backtrack(return_ms);
  }
}

impl<P1, P2, P3, P4, P5, P6, I: Interpolation> CommandComponentBuilder
  for MovementComponent<P1, P2, P3, P4, P5, P6, I>
where
  P1: ActuatorDriver,
  P2: ActuatorDriver,
  P3: ActuatorDriver,
  P4: ActuatorDriver,
  P5: ActuatorDriver,
  P6: ActuatorDriver,
{
  type WithInterpolator<NewI: Interpolation> = MovementComponent<P1, P2, P3, P4, P5, P6, NewI>;

  fn interpolate_smooth(self) -> Self::WithInterpolator<SmoothInterpolation> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: SmoothInterpolation::new(),
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  fn interpolate_linear(self) -> Self::WithInterpolator<LinearInterpolation> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: LinearInterpolation::new(),
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  fn interpolate_with<NewI: Interpolation>(
    self,
    interpolator: NewI,
  ) -> Self::WithInterpolator<NewI> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator,
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }

  fn interpolate_cubic(
    self,
    x1: u16,
    y1: u16,
    x2: u16,
    y2: u16,
  ) -> Self::WithInterpolator<CubicInterpolation> {
    MovementComponent {
      id: self.id,
      actuator1: self.actuator1,
      actuator2: self.actuator2,
      actuator3: self.actuator3,
      actuator4: self.actuator4,
      actuator5: self.actuator5,
      actuator6: self.actuator6,
      interpolator: CubicInterpolation::new(x1, y1, x2, y2),
      pipeline: self.pipeline,
      positions: self.positions,
    }
  }
}
