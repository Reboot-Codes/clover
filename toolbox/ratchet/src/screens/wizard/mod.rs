use iced::Task;

use crate::screens::MoveToScreen;

#[derive(Default, Debug, Clone)]
pub struct WizardScreen {
  step: WizardStep,
}

#[derive(Debug, Clone, Default)]
pub enum Action {
  #[default]
  None,
  MoveToScreen(MoveToScreen),
}

impl WizardScreen {
  pub fn new(starting_point: WizardStep) -> (Self, Task<crate::Message>) {
    (
      WizardScreen {
        step: starting_point,
      },
      Task::none(),
    )
  }

  pub fn view(&self, _state: &crate::MainAppState) -> iced::Element<crate::Message> {
    todo!()
  }

  pub fn update(&mut self, message: crate::Message) -> Action {
    match message {
      crate::Message::MoveToScreen(target_screen) => match target_screen {
        super::MoveToScreen::Welcome => Action::MoveToScreen(MoveToScreen::Welcome),
        super::MoveToScreen::Wizard(_starting_point) => Action::None,
        super::MoveToScreen::Configurator(id) => {
          Action::MoveToScreen(MoveToScreen::Configurator(id))
        }
      },
    }
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum WizardStep {
  #[default]
  ConnectionType,
}
