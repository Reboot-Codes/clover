use crate::commands::module::{
  components::CommandComponentPipeline,
  interpolation::Interpolation,
};

use serde::{
  Deserialize,
  Serialize,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
  pub positions: [u32; 6],
}

pub trait ActuatorDriver {
  fn drive(&mut self, setpoint: u32) -> Result<(), ()>;
}

impl ActuatorDriver for () {
  fn drive(&mut self, _setpoint: u32) -> Result<(), ()> {
    Ok(())
  }
}

pub struct PwmServo<P> {
  pub pwm_pin: P,
  pub min_pulse_us: u16,
  pub max_pulse_us: u16,
  pub pange_mdeg: u32,
}

impl<P: embedded_hal::pwm::SetDutyCycle> ActuatorDriver for PwmServo<P> {
  fn drive(&mut self, setpoint: u32) -> Result<(), ()> {
    let max = if self.pange_mdeg == 0 { 1 } else { self.pange_mdeg };
    let clamped = setpoint.min(max);
    let pulse_range = self.max_pulse_us.saturating_sub(self.min_pulse_us) as u32;
    let pulse_us = self.min_pulse_us as u32 + (pulse_range * clamped) / max;
    let max_duty = self.pwm_pin.max_duty_cycle();
    let duty = ((pulse_us as u64 * max_duty as u64) / 20000) as u16;
    self.pwm_pin.set_duty_cycle(duty).map_err(|_| ())
  }
}

pub struct StepperMotor<StepPin, DirPin> {
  pub step_pin: StepPin,
  pub dir_pin: DirPin,
  pub microsteps: u16,
  pub gear_ratio: u16,
  pub current_step: i32,
}

impl<StepPin: embedded_hal::digital::OutputPin, DirPin: embedded_hal::digital::OutputPin> ActuatorDriver
  for StepperMotor<StepPin, DirPin>
{
  fn drive(&mut self, setpoint: u32) -> Result<(), ()> {
    let target_step = setpoint as i32;
    if target_step > self.current_step {
      let _ = self.dir_pin.set_high();
      let _ = self.step_pin.set_high();
      let _ = self.step_pin.set_low();
      self.current_step += 1;
    } else if target_step < self.current_step {
      let _ = self.dir_pin.set_low();
      let _ = self.step_pin.set_high();
      let _ = self.step_pin.set_low();
      self.current_step -= 1;
    }
    Ok(())
  }
}

pub struct SerialBusServo<Uart> {
  pub uart: Uart,
  pub servo_id: u8,
}

impl<Uart> ActuatorDriver for SerialBusServo<Uart> {
  fn drive(&mut self, _setpoint: u32) -> Result<(), ()> {
    Ok(())
  }
}
