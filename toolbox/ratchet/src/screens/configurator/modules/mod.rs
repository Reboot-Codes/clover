pub mod detail;

use carbon_steel::repos::Module;
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

pub fn modules_tab(configurator_screen: &ConfiguratorScreen, content: &mut Vec<Element<Message>>) {
  debug!("Modules Tab");

  content.push(text("Modules").size(24).into());

  let mut modules = HashMap::new();

  if configurator_screen.focused_on_repo {
    let repo_id = configurator_screen.current_repo.clone().unwrap();
    for (app_id, module_config) in configurator_screen
      .repos
      .get(&repo_id)
      .unwrap()
      .modules
      .clone()
      .iter()
    {
      modules.insert((repo_id.clone(), app_id.clone()), module_config.clone());
    }
  } else {
    for (repo_id, repo_config) in configurator_screen.repos.clone().iter() {
      for (app_id, module_config) in repo_config.modules.clone().iter() {
        modules.insert((repo_id.clone(), app_id.clone()), module_config.clone());
      }
    }
  }

  content.push(
    column(
      <HashMap<(String, String), Module> as Clone>::clone(&modules)
        .into_iter()
        .map(|(ids, module_config)| {
          let (repo_id, module_id) = ids;
          let repos = &configurator_screen.repos.clone();
          let repo_name = repos.get(&repo_id).unwrap().name.clone();

          debug!(
            "Found \"{}\", with config: {:?}",
            module_id.clone(),
            module_config.clone()
          );

          // TODO: App badges
          let badges = vec![];

          button(
            row(vec![
              column(vec![
                column(vec![
                  text(module_config.name).size(18).into(),
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
          .on_press(Message::SetConfiguratorTab(ConfiguratorTab::ModuleDetail(
            repo_id.clone(),
            module_id.clone(),
            Box::new(ConfiguratorTab::Modules),
          )))
          .into()
        })
        .collect::<Vec<Element<Message>>>(),
    )
    .into(),
  );
}
