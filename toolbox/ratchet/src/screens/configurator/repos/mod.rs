pub mod detail;

use std::collections::HashMap;

use carbon_steel::repos::Repo;
use iced::{
  Element,
  widget::{
    button,
    column,
    horizontal_space,
    row,
    text,
  },
};
use iced_aw::badge;
use log::debug;

use crate::{
  Message,
  screens::configurator::{
    ConfiguratorScreen,
    ConfiguratorTab,
  },
};

pub fn repos_tab(configurator_screen: &ConfiguratorScreen, content: &mut Vec<Element<Message>>) {
  debug!("All Repos Tab");

  content.push(text("Repositories").size(24).into());

  let repos = &configurator_screen.repos.clone();

  content.push(
    column(
      <HashMap<String, Repo> as Clone>::clone(&repos)
        .into_iter()
        .map(|(id, repo_config)| {
          debug!(
            "Found \"{}\", with config: {:?}",
            id.clone(),
            repo_config.clone()
          );

          let mut badges = vec![];

          if !repo_config.modules.is_empty() {
            badges.push(badge("Modules").into());
          }

          if !repo_config.gesture_packs.is_empty() {
            badges.push(badge("Gesture Packs").into());
          }

          if !repo_config.apps.is_empty() {
            badges.push(badge("Apps").into());
          }

          button(
            row(vec![
              column(vec![
                column(vec![
                  text(repo_config.name).size(18).into(),
                  text(repo_config.url).size(14).into(),
                ])
                .into(),
                row(badges).spacing(3).into(),
              ])
              .spacing(3)
              .into(),
              horizontal_space().into(),
            ])
            .spacing(3),
          )
          // TODO: Repo management sub_screen
          .on_press(Message::SetConfiguratorTab(ConfiguratorTab::RepoDetail(
            id.clone(),
            Box::new(ConfiguratorTab::Repos),
          )))
          .into()
        })
        .collect::<Vec<Element<Message>>>(),
    )
    .into(),
  );
}
