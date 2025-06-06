use anyhow::Context;
use iced::{
  Element,
  alignment,
  widget::{
    button,
    column,
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

pub fn repo_detail_tab(
  configurator_screen: &ConfiguratorScreen,
  content: &mut Vec<Element<Message>>,
  repo_id: &String,
  prev_screen: &Box<ConfiguratorTab>,
) {
  let repo_config = configurator_screen
    .repos
    .get(repo_id)
    .with_context(|| {
      format!(
        "ConfiguratorScreen::update() should've ensured that \"{}\" existed in the repo cache",
        repo_id
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
    ])
    .into(),
  );

  content.push(text(repo_config.name.clone()).size(24).into());

  if !repo_config.modules.is_empty() {
    let mut section = vec![text("Modules").size(20).into()];

    for (_module_id, module_config) in repo_config.modules.iter() {
      // TODO: Go to module detail tab
      section.push(button(column(vec![text(module_config.name.clone()).into()])).into());
    }

    content.push(column(section).spacing(6).into());
  }

  if !repo_config.gesture_packs.is_empty() {
    let mut section = vec![text("Gesture Packs").size(20).into()];

    for (_pack_id, pack_config) in repo_config.gesture_packs.iter() {
      // TODO: Go to gesture pack detail tab
      section.push(button(column(vec![text(pack_config.name.clone()).into()])).into());
    }

    content.push(column(section).spacing(6).into());
  }

  if !repo_config.apps.is_empty() {
    let mut section = vec![text("Apps").size(20).into()];

    for (app_id, app_config) in repo_config.apps.iter() {
      section.push(
        button(column(vec![text(app_config.name.clone()).into()]))
          .on_press(Message::SetConfiguratorTab(ConfiguratorTab::AppDetail(
            repo_id.clone(),
            app_id.clone(),
            Box::new(ConfiguratorTab::RepoDetail(
              repo_id.clone(),
              prev_screen.clone(),
            )),
          )))
          .into(),
      );
    }

    content.push(column(section).spacing(6).into());
  }
}
