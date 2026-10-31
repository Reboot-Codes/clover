use crate::{
  alerts::module::components::AlertComponentPipeline,
  alerts::{
    models::components::{
      AlertMessage,
      microphone::{
        MicrophoneComponent,
        MicrophoneDriver,
      },
    },
    module::{
      AlertComponent,
      AlertComponentBuilder,
    },
  },
};

impl<Dr: MicrophoneDriver> AlertComponent for MicrophoneComponent<Dr> {
  fn id(&self) -> u8 {
    self.id
  }

  fn get_status(&self) -> Option<AlertMessage> {
    todo!()
  }
}

impl<Dr: MicrophoneDriver> MicrophoneComponent<Dr> {
  pub fn new(id: u8, driver: Dr) -> Self {
    Self {
      id,
      sensitivity: crate::alerts::module::sensitivity::Sensitivity::Polled,
      pipeline: AlertComponentPipeline::new(),
      driver,
    }
  }
}

impl<Dr: MicrophoneDriver> AlertComponentBuilder for MicrophoneComponent<Dr> {}
