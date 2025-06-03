use crate::screens::MoveToScreen;
use iced::{
  Task,
  widget::{
    button,
    column,
    horizontal_space,
    row,
    text,
    vertical_space,
  },
};
use log::debug;

#[derive(Default, Debug, Clone)]
pub struct WizardScreen {
  step: WizardStep,
}

#[derive(Debug, Clone, Default)]
pub enum Action {
  #[default]
  None,
  MoveToScreen(MoveToScreen),
  SetStep(WizardStep),
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
    let mut elements = Vec::new();

    elements.push(text("Configuration Wizard").into());
    let mut controls = Vec::new();

    controls.push(
      button("Cancel")
        .on_press(crate::Message::MoveToScreen(MoveToScreen::Welcome))
        .into(),
    );
    controls.push(horizontal_space().into());

    match self.step {
      WizardStep::Intro => {
        elements.push(text("introduction").into());

        controls.push(button("Back").into());
        controls.push(
          button("Forward")
            .on_press(crate::Message::SetWizardStep(WizardStep::ConnectionType))
            .into(),
        );
      }
      WizardStep::ConnectionType => {
        elements.push(text("existing connection type").into());

        controls.push(
          button("Back")
            .on_press(crate::Message::SetWizardStep(WizardStep::Intro))
            .into(),
        );
        controls.push(
          button("Finish")
            .on_press(crate::Message::SetWizardStep(WizardStep::Finishing))
            .into(),
        );
      }
      WizardStep::Finishing => {
        elements.push(text("Connecting to: ").into());

        controls.push(
          button("Back")
            .on_press(crate::Message::SetWizardStep(WizardStep::ConnectionType))
            .into(),
        );
      }
    }

    elements.push(vertical_space().into());
    elements.push(
      row(controls)
        .spacing(12)
        .padding(12)
        .width(iced::Length::Fill)
        .into(),
    );
    column(elements).spacing(12).padding(12).into()
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
      crate::Message::SetWizardStep(wizard_step) => {
        debug!("Moving to wizard step: {:?}", wizard_step);

        // TODO: Do validation on step move
        self.step = wizard_step;
        Action::None
      }
      crate::Message::SetConfiguratorTab(_tab) => Action::None,
    }
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum WizardStep {
  #[default]
  Intro = 0,
  ConnectionType = 1,
  Finishing = 3,
}
