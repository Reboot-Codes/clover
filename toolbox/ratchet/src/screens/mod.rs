use iced::Element;

use crate::{
  MainAppState,
  Message,
  screens::{
    configurator::ConfiguratorScreen,
    welcome::WelcomeScreen,
    wizard::{
      WizardScreen,
      WizardStartingPoints,
    },
  },
};

pub mod configurator;
pub mod welcome;
pub mod wizard;

#[derive(Debug, Clone)]
pub enum CurrentTopLevelScreen {
  Welcome(WelcomeScreen),
  Wizard(WizardScreen),
  Configurator(ConfiguratorScreen),
}

impl Default for CurrentTopLevelScreen {
  fn default() -> Self {
    CurrentTopLevelScreen::Welcome(WelcomeScreen::default())
  }
}

#[derive(Debug, Clone)]
pub enum MoveToScreen {
  Welcome,
  Wizard(WizardStartingPoints),
  Configurator(String),
}

pub trait TopLevelScreen {
  fn view(&self, state: &MainAppState) -> Element<Message>;
  fn update(&mut self, message: Message);
}
