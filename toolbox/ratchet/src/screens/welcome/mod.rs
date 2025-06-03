use carbon_steel::connections::ConnectionConfiguration;
use iced::widget::{
  button,
  column,
  row,
  text,
};
use log::{
  debug,
  info,
};
use std::collections::HashMap;

use crate::screens::{
  MoveToScreen,
  wizard::WizardStep,
};

#[derive(Debug, Clone, Default)]
pub struct WelcomeScreen {}

#[derive(Debug, Clone, Default)]
pub enum Action {
  #[default]
  None,
  MoveToScreen(MoveToScreen),
}

impl WelcomeScreen {
  pub fn update(&mut self, message: crate::Message) -> Action {
    match message {
      crate::Message::MoveToScreen(target_screen) => match target_screen {
        super::MoveToScreen::Welcome => Action::None,
        super::MoveToScreen::Wizard(starting_point) => {
          Action::MoveToScreen(MoveToScreen::Wizard(starting_point))
        }
        super::MoveToScreen::Configurator(id) => {
          Action::MoveToScreen(MoveToScreen::Configurator(id))
        }
      },
      crate::Message::SetWizardStep(_wizard_step) => Action::None,
    }
  }

  pub fn view(&self, state: &crate::MainAppState) -> iced::Element<crate::Message> {
    let mut elements = vec![
      text("Hey there!").size(32).into(),
      text("Using Ratchet v0.0.1").into(),
    ];

    let connections = state.connections.clone();
    let num_connections = connections.len();

    info!("Loading pre-existing connections into UI state...");
    if num_connections > 0 {
      debug!("Connections found!");
      elements.push(text("Existing connections:").size(24).into());
      <HashMap<String, ConnectionConfiguration> as Clone>::clone(&connections)
        .into_iter()
        .map(|(id, configuration)| {
          debug!(
            "Found \"{}\", with config: {:?}",
            id.clone(),
            configuration.clone()
          );

          button("Connection")
            .on_press(crate::Message::MoveToScreen(
              super::MoveToScreen::Configurator(id.clone()),
            ))
            .into()
        })
        .collect_into(&mut elements);
    } else {
      debug!("No connections found...");
      elements.push(text("No existing connections!").into());
    }

    elements.push(text("Create a New Connection...").into());
    elements.push(
      row(vec![
        button("From Scratch")
          .on_press(crate::Message::MoveToScreen(MoveToScreen::Wizard(
            WizardStep::Intro,
          )))
          .into(),
        button("to an Existing Instance")
          .on_press(crate::Message::MoveToScreen(MoveToScreen::Wizard(
            WizardStep::ConnectionType,
          )))
          .into(),
      ])
      .into(),
    );

    column(elements).into()
  }
}

#[derive(Debug, Clone, Copy)]
pub struct Connection {}
