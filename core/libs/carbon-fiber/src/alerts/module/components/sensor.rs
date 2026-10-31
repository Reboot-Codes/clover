use crate::alerts::{
  models::components::{
    AlertMessage,
    sensors::{
      SensorComponent,
      SensorDriver,
    },
  },
  module::{
    AlertComponent,
    AlertComponentBuilder,
    components::AlertComponentPipeline,
    sensitivity::Sensitivity,
  },
};

impl<Dr: SensorDriver> AlertComponent for SensorComponent<Dr> {
  fn id(&self) -> u8 {
    self.id
  }

  fn get_status(&self) -> Option<AlertMessage> {
    None
  }

  fn step(&mut self, dt_ms: u32) -> Option<AlertMessage> {
    self.pipeline.debounce_ms = self.sensitivity.debounce_ms();
    self
      .driver
      .poll_payload(&mut self.pipeline, dt_ms)
      .map(|payload| AlertMessage {
        component_id: self.id,
        severity: crate::alerts::models::components::AlertSeverity::Info,
        timestamp_ms: (self.pipeline.uptime_ms % 65536) as u16,
        payload,
      })
  }
}

impl<Dr: SensorDriver> SensorComponent<Dr> {
  pub fn new(id: u8, driver: Dr, sensitivity: Sensitivity) -> Self {
    let mut pipeline = AlertComponentPipeline::new();
    pipeline.debounce_ms = sensitivity.debounce_ms();
    Self {
      id,
      pipeline,
      sensitivity,
      driver,
    }
  }
}

impl<Dr: SensorDriver> AlertComponentBuilder for SensorComponent<Dr> {}
