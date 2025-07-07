pub mod steps;

use crate::{
  screens::{
    MoveToScreen,
    wizard::steps::{
      connection_type::connection_type,
      from_scratch_intro::from_scratch_intro,
      use_case::use_case,
    },
  },
  util::menu::gen_menu_bar,
};
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
  pub step: WizardStep,
  pub first_step: WizardStep,
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
        first_step: starting_point,
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
      WizardStep::FromScratchIntro => from_scratch_intro(self, &mut elements, &mut controls),
      WizardStep::UseCase => use_case(self, &mut elements, &mut controls),
      WizardStep::ConnectionType => connection_type(self, &mut elements, &mut controls),
      WizardStep::Finishing => {
        elements.push(text("Connecting to: ").into());

        controls.push(
          button("Back")
            .on_press(crate::Message::SetWizardStep(WizardStep::ConnectionType))
            .into(),
        );
      }
    }

    column(vec![
      gen_menu_bar(),
      column(elements)
        .padding(iced::Padding {
          top: 12.0,
          bottom: 0.0,
          right: 12.0,
          left: 12.0,
        })
        .spacing(12)
        .into(),
      vertical_space().into(),
      row(controls)
        .spacing(12)
        .padding(12)
        .width(iced::Length::Fill)
        .into(),
    ])
    .spacing(12)
    .into()
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
      crate::Message::None => Action::None,
    }
  }
}

#[derive(Default, Debug, Clone, Copy)]
pub enum WizardStep {
  #[default]
  FromScratchIntro,
  UseCase,
  ConnectionType,
  Finishing,
}
