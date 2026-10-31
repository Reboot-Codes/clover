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
    None
  }

  fn step(&mut self, _dt_ms: u32) -> Option<AlertMessage> {
    todo!("Camera streaming pipeline deferred to future sprint")
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
