pub mod detail;

use carbon_steel::repos::App;
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
use log::debug;
use std::collections::HashMap;

use crate::{
  Message,
  screens::configurator::{
    ConfiguratorScreen,
    ConfiguratorTab,
  },
};

pub fn apps_tab(configurator_screen: &ConfiguratorScreen, content: &mut Vec<Element<Message>>) {
  debug!("Apps Tab");

  content.push(text("Apps").size(24).into());

  let mut apps = HashMap::new();

  for (repo_id, repo_config) in configurator_screen.repos.clone().iter() {
    for (app_id, app_config) in repo_config.apps.clone().iter() {
      apps.insert((repo_id.clone(), app_id.clone()), app_config.clone());
    }
  }

  content.push(
    column(
      <HashMap<(String, String), App> as Clone>::clone(&apps)
        .into_iter()
        .map(|(ids, app_config)| {
          let (repo_id, app_id) = ids;
          let repos = &configurator_screen.repos.clone();
          let repo_name = repos.get(&repo_id).unwrap().name.clone();

          debug!(
            "Found \"{}\", with config: {:?}",
            app_id.clone(),
            app_config.clone()
          );

          // TODO: App badges
          let badges = vec![];

          button(
            row(vec![
              column(vec![
                column(vec![
                  text(app_config.name).size(18).into(),
                  text(repo_name.clone()).size(14).into(),
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
          .on_press(Message::SetConfiguratorTab(ConfiguratorTab::AppDetail(
            repo_id.clone(),
            app_id.clone(),
            Box::new(ConfiguratorTab::Apps),
          )))
          .into()
        })
        .collect::<Vec<Element<Message>>>(),
    )
    .into(),
  );
}
