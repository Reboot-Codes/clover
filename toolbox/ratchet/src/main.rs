pub mod screens;
// pub mod util;

use std::env;

use screens::TopLevelScreen;
use screens::welcome::WelcomeScreen;

use iced::{
  Center,
  Element,
  Task,
  Theme,
  widget::{
    Column,
    button,
    column,
    text,
  },
};

use crate::screens::{
  CurrentTopLevelScreen,
  MoveToScreen,
};

fn theme(state: &MainAppState) -> Theme {
  Theme::TokyoNight
}

pub const APP_NAME: &str = "Ratchet";

pub fn main() -> iced::Result {
  // TODO:: Create a logger that will send logs to a FIFO buffer to send over WS via EvtBuzz
  env_logger::Builder::new()
    .parse_filters(&env::var("RATCHET_LOG").unwrap_or("info".to_string()))
    .init();

  iced::application(APP_NAME, MainAppState::update, MainAppState::view)
    .theme(theme)
    .run()
}

#[derive(Default)]
pub struct MainAppState {
  pub screen: CurrentTopLevelScreen,
}

#[derive(Debug, Clone)]
pub enum Message {
  MoveToScreen(MoveToScreen),
}

impl MainAppState {
  fn update(&mut self, message: Message) -> Task<Message> {
    match message {
      Message::MoveToScreen(screen) => match screen {
        MoveToScreen::Welcome => Task::none(),
        MoveToScreen::Wizard(_) => todo!(),
        MoveToScreen::Configurator(instance_id) => todo!(),
      },
    }
  }

  fn view(&self) -> Element<Message> {
    match &self.screen {
      CurrentTopLevelScreen::Welcome(welcome) => welcome.view(self),
      CurrentTopLevelScreen::Wizard(wizard_screen) => todo!(),
      CurrentTopLevelScreen::Configurator(configurator_screen) => todo!(),
    }
  }
}
