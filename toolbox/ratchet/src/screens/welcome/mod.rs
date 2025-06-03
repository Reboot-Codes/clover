use iced::widget::text;
use log::debug;
use std::collections::HashMap;

use crate::screens::TopLevelScreen;

#[derive(Debug, Clone, Default)]
pub struct WelcomeScreen {
  connections: HashMap<String, Connection>,
}

impl TopLevelScreen for WelcomeScreen {
  fn update(&mut self, message: crate::Message) {
    debug!("{:?}", &message);
  }

  fn view(&self, state: &crate::MainAppState) -> iced::Element<crate::Message> {
    text("I am 300px tall!").height(300).into()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Connection {}
