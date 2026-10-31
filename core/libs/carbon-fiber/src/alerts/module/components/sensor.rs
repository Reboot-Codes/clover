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
    todo!()
  }
}

impl<Dr: SensorDriver> SensorComponent<Dr> {
  pub fn new(id: u8, driver: Dr, sensitivity: Sensitivity) -> Self {
    Self {
      id,
      pipeline: AlertComponentPipeline::new(),
      sensitivity,
      driver,
    }
  }
}

impl<Dr: SensorDriver> AlertComponentBuilder for SensorComponent<Dr> {}
