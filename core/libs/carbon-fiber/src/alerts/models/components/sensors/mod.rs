use embedded_hal::digital::InputPin;

use crate::{
  alerts::models::components::AlertPayload,
  alerts::module::components::AlertComponentPipeline,
  alerts::module::sensitivity::{
    EdgeTrigger,
    Sensitivity,
  },
};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SensorComponent<Dr: SensorDriver> {
  pub id: u8,
  pub sensitivity: Sensitivity,
  pub pipeline: AlertComponentPipeline,
  pub driver: Dr,
}

pub trait SensorDriver {
  fn poll_payload(
    &mut self,
    pipeline: &mut AlertComponentPipeline,
    dt_ms: u32,
  ) -> Option<AlertPayload>;
}

/// Momentary switch / contact binary sensor.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BinarySensor<P> {
  pub pin: P,
  pub pin_index: u8,
  pub active_low: bool,
  pub edge: EdgeTrigger,
}

impl<P: InputPin> BinarySensor<P> {
  pub fn new(pin: P, pin_index: u8, active_low: bool, edge: EdgeTrigger) -> Self {
    Self {
      pin,
      pin_index,
      active_low,
      edge,
    }
  }
}

impl<P: InputPin> SensorDriver for BinarySensor<P> {
  fn poll_payload(
    &mut self,
    pipeline: &mut AlertComponentPipeline,
    dt_ms: u32,
  ) -> Option<AlertPayload> {
    pipeline.edge_trigger = self.edge;
    let is_high = self.pin.is_high().unwrap_or(false);
    let raw_active = if self.active_low { !is_high } else { is_high };
    pipeline
      .tick_digital(dt_ms, raw_active, pipeline.debounce_ms)
      .map(|active| AlertPayload::DigitalContact {
        pin_index: self.pin_index,
        active,
      })
  }
}

/// Capacitive touch sensor.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct CapacitiveSensor<P> {
  pub pin: P,
  pub pin_index: u8,
  pub active_low: bool,
  pub raw_proximity: Option<i16>,
  pub proximity_threshold: i16,
}

impl<P: InputPin> CapacitiveSensor<P> {
  pub fn new(pin: P, pin_index: u8) -> Self {
    Self {
      pin,
      pin_index,
      active_low: false,
      raw_proximity: None,
      proximity_threshold: 0,
    }
  }

  pub fn with_threshold(mut self, threshold: i16) -> Self {
    self.proximity_threshold = threshold;
    self
  }

  pub fn set_proximity(&mut self, prox: i16) {
    self.raw_proximity = Some(prox);
  }
}

impl<P: InputPin> SensorDriver for CapacitiveSensor<P> {
  fn poll_payload(
    &mut self,
    pipeline: &mut AlertComponentPipeline,
    dt_ms: u32,
  ) -> Option<AlertPayload> {
    if let Some(proximity) = self.raw_proximity {
      if proximity >= self.proximity_threshold {
        return Some(AlertPayload::ThresholdCrossed {
          reading_raw: proximity,
          threshold: self.proximity_threshold,
        });
      }
    }
    let is_high = self.pin.is_high().unwrap_or(false);
    let raw_active = if self.active_low { !is_high } else { is_high };
    pipeline
      .tick_digital(dt_ms, raw_active, pipeline.debounce_ms)
      .map(|active| AlertPayload::DigitalContact {
        pin_index: self.pin_index,
        active,
      })
  }
}

/// Rotary encoder sensor.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RotaryEncoderSensor<PinA, PinB> {
  pub pin_a: PinA,
  pub pin_b: PinB,
  pub last_state: u8,
  pub step_count: i32,
}

impl<PinA: InputPin, PinB: InputPin> RotaryEncoderSensor<PinA, PinB> {
  pub fn new(pin_a: PinA, pin_b: PinB) -> Self {
    Self {
      pin_a,
      pin_b,
      last_state: 0,
      step_count: 0,
    }
  }
}

impl<PinA: InputPin, PinB: InputPin> SensorDriver for RotaryEncoderSensor<PinA, PinB> {
  fn poll_payload(
    &mut self,
    _pipeline: &mut AlertComponentPipeline,
    _dt_ms: u32,
  ) -> Option<AlertPayload> {
    let a = self.pin_a.is_high().unwrap_or(false) as u8;
    let b = self.pin_b.is_high().unwrap_or(false) as u8;
    let current_state = (a << 1) | b;
    if current_state != self.last_state {
      let dir = match (self.last_state, current_state) {
        (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => 1,
        (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => -1,
        _ => 0,
      };
      self.last_state = current_state;
      if dir != 0 {
        self.step_count += dir;
        return Some(AlertPayload::SpatialTrigger {
          event_flags: if dir > 0 { 1 } else { 2 },
          magnitude: self.step_count.unsigned_abs() as u16,
        });
      }
    }
    None
  }
}

/// Variable resistor / Potentiometer sensor.
#[derive(Debug, Clone, Copy)]
pub struct AnalogResistorSensor<R> {
  pub reader: R,
  pub deadband: i16,
  pub low_trip: i16,
  pub high_trip: i16,
  pub last_reading: i16,
}

impl<R: FnMut() -> i16> AnalogResistorSensor<R> {
  pub fn new(reader: R, deadband: i16, low_trip: i16, high_trip: i16) -> Self {
    Self {
      reader,
      deadband,
      low_trip,
      high_trip,
      last_reading: 0,
    }
  }
}

impl<R: FnMut() -> i16> SensorDriver for AnalogResistorSensor<R> {
  fn poll_payload(
    &mut self,
    _pipeline: &mut AlertComponentPipeline,
    _dt_ms: u32,
  ) -> Option<AlertPayload> {
    let reading = (self.reader)();
    let delta = (reading - self.last_reading).abs();
    if delta >= self.deadband {
      self.last_reading = reading;
      if reading <= self.low_trip {
        return Some(AlertPayload::ThresholdCrossed {
          reading_raw: reading,
          threshold: self.low_trip,
        });
      } else if reading >= self.high_trip {
        return Some(AlertPayload::ThresholdCrossed {
          reading_raw: reading,
          threshold: self.high_trip,
        });
      }
    }
    None
  }
}
