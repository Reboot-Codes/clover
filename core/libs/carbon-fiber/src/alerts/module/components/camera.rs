use crate::{
  alerts::module::components::AlertComponentPipeline,
  alerts::{
    models::components::{
      AlertMessage,
      camera::{
        CameraComponent,
        CameraDriver,
      },
    },
    module::{
      AlertComponent,
      AlertComponentBuilder,
    },
  },
};

impl<Dr: CameraDriver> AlertComponent for CameraComponent<Dr> {
  fn id(&self) -> u8 {
    self.id
  }

  fn get_status(&self) -> Option<AlertMessage> {
    todo!()
  }
}

impl<Dr: CameraDriver> CameraComponent<Dr> {
  pub fn new<NewDr: CameraDriver>(id: u8, driver: NewDr) -> CameraComponent<NewDr> {
    CameraComponent {
      id,
      sensitivity: crate::alerts::module::sensitivity::Sensitivity::Polled,
      pipeline: AlertComponentPipeline::new(),
      driver,
    }
  }
}

impl<Dr: CameraDriver> AlertComponentBuilder for CameraComponent<Dr> {}
