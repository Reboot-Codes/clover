use anyhow::Context;
use iced::{
  Element,
  alignment,
  widget::{
    button,
    horizontal_space,
    row,
    text,
  },
};
use iced_fonts::{
  REQUIRED_FONT,
  required::{
    RequiredIcons,
    icon_to_string,
  },
};

use crate::{
  Message,
  screens::configurator::{
    ConfiguratorScreen,
    ConfiguratorTab,
  },
};

pub fn app_detail_tab(
  configurator_screen: &ConfiguratorScreen,
  content: &mut Vec<Element<Message>>,
  repo_id: &String,
  app_id: &String,
  prev_screen: &Box<ConfiguratorTab>,
) {
  let repo = configurator_screen
    .repos
    .get(repo_id)
    .with_context(|| {
      format!(
        "ConfiguratorScreen::update() should've ensured that repo: \"{}\" existed in the repo cache",
        repo_id
      )
    })
    .unwrap();

  let app = repo
    .apps
    .get(app_id)
    .with_context(|| {
      format!(
        "ConfiguratorScreen::update() should've ensured that app: \"{}\" existed in the repo cache",
        app_id
      )
    })
    .unwrap();

  content.push(
    row(vec![
      button(row(vec![
        text(icon_to_string(RequiredIcons::CaretLeftFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
        text("Back").into(),
      ]))
      .on_press(Message::SetConfiguratorTab(*prev_screen.clone()))
      .into(),
      horizontal_space().into(),
      button("Settings").into(),
    ])
    .into(),
  );

  content.push(text(app.name.clone()).size(24).into());
}
