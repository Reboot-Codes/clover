use crate::screens::TopLevelScreen;

#[derive(Debug, Clone)]
pub struct ConfiguratorScreen {
  current_instance: String,
}

impl TopLevelScreen for ConfiguratorScreen {
  fn view(&self, state: &crate::MainAppState) -> iced::Element<crate::Message> {
    todo!()
  }

  fn update(&mut self, message: crate::Message) {
    todo!()
  }
}
