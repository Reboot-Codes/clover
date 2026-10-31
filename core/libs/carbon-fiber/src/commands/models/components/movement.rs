use crate::commands::module::{
  components::CommandComponentPipeline,
  interpolation::Interpolation,
};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum MovementTarget {
  OneDof(u32),
  /// x and y
  TwoDof(u32, u32),
  /// x, y, and z
  ThreeDof(u32, u32, u32),
  /// x, y, z, and u
  FourDof(u32, u32, u32, u32),
  /// x, y, z, u, and v
  FiveDof(u32, u32, u32, u32, u32),
  /// x, y, z, u, v, and w
  SixDof(u32, u32, u32, u32, u32, u32),
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MovementComponent<A1, A2, A3, A4, A5, A6, I>
where
  I: Interpolation,
{
  pub id: u8,
  pub actuator1: Option<A1>,
  pub actuator2: Option<A2>,
  pub actuator3: Option<A3>,
  pub actuator4: Option<A4>,
  pub actuator5: Option<A5>,
  pub actuator6: Option<A6>,
  pub interpolator: I,
  pub pipeline: CommandComponentPipeline,
}

pub struct PwmServo<P> {
  pub pwm_pin: P,
  pub min_pulse_us: u16,
  pub max_pulse_us: u16,
  pub pange_mdeg: u32,
}

pub struct StepperMotor<StepPin, DirPin> {
  pub step_pin: StepPin,
  pub dir_pin: DirPin,
  pub microsteps: u16,
  pub gear_ratio: u16,
  pub current_step: i32,
}

pub struct SerialBusServo<Uart> {
  pub uart: Uart,
  pub servo_id: u8,
}
