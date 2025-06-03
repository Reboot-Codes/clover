#![feature(iter_collect_into)]

pub mod screens;
// pub mod util;

use std::{
  collections::HashMap,
  env,
};

use carbon_steel::connections::ConnectionConfiguration;

use iced::{
  Element,
  Task,
  Theme,
};
use log::{
  debug,
  warn,
};

use crate::screens::{
  CurrentTopLevelScreen,
  MoveToScreen,
  configurator::ConfiguratorScreen,
  wizard::{
    WizardScreen,
    WizardStep,
  },
};

fn theme(_state: &MainAppState) -> Theme {
  Theme::TokyoNight
}

pub const APP_NAME: &str = "Ratchet";

pub fn main() -> iced::Result {
  env_logger::Builder::new()
    .parse_filters(&env::var("RATCHET_LOG").unwrap_or("info".to_string()))
    .init();

  iced::application(APP_NAME, MainAppState::update, MainAppState::view)
    .theme(theme)
    .run()
}

pub struct MainAppState {
  pub screen: CurrentTopLevelScreen,
  pub current_connection: Option<String>,
  pub connections: HashMap<String, ConnectionConfiguration>,
}

impl Default for MainAppState {
  fn default() -> Self {
    // Load state when the app starts up...
    let mut connections = HashMap::new();

    connections.insert("main".to_string(), ConnectionConfiguration {});

    MainAppState {
      screen: Default::default(),
      current_connection: None,
      connections,
    }
  }
}

#[derive(Debug, Clone)]
pub enum Message {
  MoveToScreen(MoveToScreen),
  SetWizardStep(WizardStep),
}

impl MainAppState {
  fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::MoveToScreen(ref screen) => match screen {
        MoveToScreen::Welcome => match &mut self.screen {
          CurrentTopLevelScreen::Welcome(_welcome_screen) => Task::none(),
          CurrentTopLevelScreen::Wizard(wizard_screen) => {
            let action = wizard_screen.update(message.clone());

            match action {
              screens::wizard::Action::None => Task::none(),
              screens::wizard::Action::MoveToScreen(move_to_screen) => match move_to_screen {
                MoveToScreen::Welcome => {
                  self.screen = CurrentTopLevelScreen::Welcome(Default::default());
                  Task::none()
                }
                MoveToScreen::Wizard(_starting_point) => Task::none(),
                MoveToScreen::Configurator(id) => {
                  let (screen, task) = ConfiguratorScreen::new(id);

                  self.screen = CurrentTopLevelScreen::Configurator(screen);
                  task
                }
              },
              screens::wizard::Action::SetStep(_wizard_step) => Task::none(),
            }
          }
          CurrentTopLevelScreen::Configurator(configurator_screen) => {
            let action = configurator_screen.update(message.clone());

            match action {
              screens::configurator::Action::None => Task::none(),
              screens::configurator::Action::MoveToScreen(move_to_screen) => match move_to_screen {
                MoveToScreen::Welcome => {
                  self.screen = CurrentTopLevelScreen::Welcome(Default::default());
                  Task::none()
                }
                MoveToScreen::Configurator(_id) => Task::none(),
                MoveToScreen::Wizard(starting_point) => {
                  let (screen, task) = WizardScreen::new(starting_point);

                  self.screen = CurrentTopLevelScreen::Wizard(screen);
                  task
                }
              },
            }
          }
        },
        MoveToScreen::Wizard(_starting_point) => match &mut self.screen {
          CurrentTopLevelScreen::Welcome(welcome_screen) => {
            let action = welcome_screen.update(message.clone());

            match action {
              screens::welcome::Action::None => Task::none(),
              screens::welcome::Action::MoveToScreen(move_to_screen) => match move_to_screen {
                MoveToScreen::Welcome => Task::none(),
                MoveToScreen::Wizard(starting_point) => {
                  let (screen, task) = WizardScreen::new(starting_point);

                  self.screen = CurrentTopLevelScreen::Wizard(screen);
                  task
                }
                MoveToScreen::Configurator(id) => {
                  let (screen, task) = ConfiguratorScreen::new(id);

                  self.screen = CurrentTopLevelScreen::Configurator(screen);
                  task
                }
              },
            }
          }
          CurrentTopLevelScreen::Wizard(_wizard_screen) => Task::none(),
          CurrentTopLevelScreen::Configurator(configurator_screen) => {
            let action = configurator_screen.update(message.clone());

            match action {
              screens::configurator::Action::None => Task::none(),
              screens::configurator::Action::MoveToScreen(move_to_screen) => match move_to_screen {
                MoveToScreen::Welcome => {
                  self.screen = CurrentTopLevelScreen::Welcome(Default::default());
                  Task::none()
                }
                MoveToScreen::Configurator(_id) => Task::none(),
                MoveToScreen::Wizard(starting_point) => {
                  let (screen, task) = WizardScreen::new(starting_point);

                  self.screen = CurrentTopLevelScreen::Wizard(screen);
                  task
                }
              },
            }
          }
        },
        MoveToScreen::Configurator(_instance_id) => match &mut self.screen {
          CurrentTopLevelScreen::Welcome(welcome_screen) => {
            let action = welcome_screen.update(message.clone());

            match action {
              screens::welcome::Action::None => Task::none(),
              screens::welcome::Action::MoveToScreen(move_to_screen) => match move_to_screen {
                MoveToScreen::Welcome => Task::none(),
                MoveToScreen::Wizard(starting_point) => {
                  let (screen, task) = WizardScreen::new(starting_point);

                  self.screen = CurrentTopLevelScreen::Wizard(screen);
                  task
                }
                MoveToScreen::Configurator(id) => {
                  let (screen, task) = ConfiguratorScreen::new(id);

                  self.screen = CurrentTopLevelScreen::Configurator(screen);
                  task
                }
              },
            }
          }
          CurrentTopLevelScreen::Wizard(wizard_screen) => {
            let action = wizard_screen.update(message.clone());

            match action {
              screens::wizard::Action::None => Task::none(),
              screens::wizard::Action::MoveToScreen(move_to_screen) => match move_to_screen {
                MoveToScreen::Welcome => {
                  self.screen = CurrentTopLevelScreen::Welcome(Default::default());
                  Task::none()
                }
                MoveToScreen::Wizard(_starting_point) => Task::none(),
                MoveToScreen::Configurator(id) => {
                  let (screen, task) = ConfiguratorScreen::new(id);

                  self.screen = CurrentTopLevelScreen::Configurator(screen);
                  task
                }
              },
              screens::wizard::Action::SetStep(_wizard_step) => Task::none(),
            }
          }
          CurrentTopLevelScreen::Configurator(_configurator_screen) => Task::none(),
        },
      },
      Message::SetWizardStep(_wizard_step) => match &mut self.screen {
        CurrentTopLevelScreen::Welcome(_welcome_screen) => Task::none(),
        CurrentTopLevelScreen::Wizard(wizard_screen) => {
          let action = wizard_screen.update(message.clone());

          match action {
            screens::wizard::Action::None => Task::none(),
            screens::wizard::Action::MoveToScreen(_move_to_screen) => {
              warn!("Not moving to screen since this is not the message that should cause that!");
              Task::none()
            }

            screens::wizard::Action::SetStep(_wizard_step) => Task::none(),
          }
        }
        CurrentTopLevelScreen::Configurator(_configurator_screen) => Task::none(),
      },
    }
  }

  fn view(&self) -> Element<Message> {
    match &self.screen {
      CurrentTopLevelScreen::Welcome(welcome_screen) => {
        debug!("Showing: Welcome Screen...");
        welcome_screen.view(self)
      }
      CurrentTopLevelScreen::Wizard(wizard_screen) => {
        debug!("Showing: Configuration Generation Wizard...");
        wizard_screen.view(self)
      }
      CurrentTopLevelScreen::Configurator(configurator_screen) => {
        debug!("Showing: Instance Configurator");
        configurator_screen.view(self)
      }
    }
  }
}
