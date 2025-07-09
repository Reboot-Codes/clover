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
use log::debug;

use crate::{
  Message,
  screens::configurator::{
    ConfiguratorScreen,
    ConfiguratorTab,
  },
};

pub fn gesture_detail_tab(
  configurator_screen: &ConfiguratorScreen,
  content: &mut Vec<Element<Message>>,
  repo_id: &String,
  gesture_pack_id: &String,
  gesture_id: &String,
  prev_screen: &Box<ConfiguratorTab>,
) {
  debug!("Gesture Detail Tab");

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

  let gesture = repo
    .gesture_packs
    .get(gesture_pack_id)
    .with_context(|| {
      format!(
        "ConfiguratorScreen::update() should've ensured that gesture pack: \"{}\" existed in the repo cache",
        gesture_pack_id
      )
    })
    .unwrap()
    .gestures
    .get(gesture_id)
    .with_context(|| {
      format!(
        "ConfiguratorScreen::update() should've ensured that gesture: \"{}\" existed in the repo cache",
        gesture_id
      )
    })
    .unwrap();

  content.push(
    row(vec![
      (match &**prev_screen {
        ConfiguratorTab::None => horizontal_space().into(),
        _ => button(row(vec![
          text(icon_to_string(RequiredIcons::CaretLeftFill))
            .font(REQUIRED_FONT)
            .width(iced::Length::Shrink)
            .align_y(alignment::Vertical::Center)
            .into(),
          text("Back").into(),
        ]))
        .on_press(Message::SetConfiguratorTab(*prev_screen.clone()))
        .into(),
      }),
      horizontal_space().into(),
      button("Settings").into(),
    ])
    .into(),
  );

  content.push(text(gesture.name.clone()).size(24).into());
}
