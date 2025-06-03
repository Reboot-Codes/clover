use iced::{
  Task,
  widget::button,
};

use crate::screens::MoveToScreen;

#[derive(Debug, Clone)]
pub struct ConfiguratorScreen {
  pub instance_id: String,
}

#[derive(Debug, Clone, Default)]
pub enum Action {
  #[default]
  None,
  MoveToScreen(MoveToScreen),
}

impl ConfiguratorScreen {
  pub fn new(id: String) -> (Self, Task<crate::Message>) {
    (ConfiguratorScreen { instance_id: id }, Task::none())
  }

  pub fn view(&self, _state: &crate::MainAppState) -> iced::Element<crate::Message> {
    button("Hallo, Welt")
      .on_press(crate::Message::MoveToScreen(MoveToScreen::Welcome))
      .into()
  }

  pub fn update(&mut self, message: crate::Message) -> Action {
    match message {
      crate::Message::MoveToScreen(target_screen) => match target_screen {
        super::MoveToScreen::Welcome => Action::MoveToScreen(MoveToScreen::Welcome),
        super::MoveToScreen::Configurator(_id) => Action::None,
        super::MoveToScreen::Wizard(starting_point) => {
          Action::MoveToScreen(MoveToScreen::Wizard(starting_point))
        }
      },
    }
  }
}
