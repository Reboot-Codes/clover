use crate::screens::{
  configurator::ConfiguratorScreen,
  welcome::WelcomeScreen,
  wizard::{
    WizardScreen,
    WizardStep,
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
  Wizard(WizardStep),
  Configurator(ConfiguratorFocus),
}

#[derive(Debug, Clone)]
pub enum ConfiguratorFocus {
  Instance(String),
  Repo(String),
}
