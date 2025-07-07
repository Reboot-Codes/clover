pub mod pack_detail;

use carbon_steel::repos::GesturePack;
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

pub fn gestures_tab(configurator_screen: &ConfiguratorScreen, content: &mut Vec<Element<Message>>) {
  debug!("Gestures Tab");

  content.push(text("Gestures").size(24).into());

  let mut gesture_packs = HashMap::new();

  if configurator_screen.focused_on_repo {
    let repo_id = configurator_screen.current_repo.clone().unwrap();
    for (gesture_pack_id, module_config) in configurator_screen
      .repos
      .get(&repo_id)
      .unwrap()
      .gesture_packs
      .clone()
      .iter()
    {
      gesture_packs.insert(
        (repo_id.clone(), gesture_pack_id.clone()),
        module_config.clone(),
      );
    }
  } else {
    for (repo_id, repo_config) in configurator_screen.repos.clone().iter() {
      for (gesture_pack_id, gesture_pack_config) in repo_config.gesture_packs.clone().iter() {
        gesture_packs.insert(
          (repo_id.clone(), gesture_pack_id.clone()),
          gesture_pack_config.clone(),
        );
      }
    }
  }

  content.push(
    column(
      <HashMap<(String, String), GesturePack> as Clone>::clone(&gesture_packs)
        .into_iter()
        .map(|(ids, gesture_pack_config)| {
          let (repo_id, gesture_pack_id) = ids;
          let repos = &configurator_screen.repos.clone();
          let repo_name = repos.get(&repo_id).unwrap().name.clone();

          debug!(
            "Found \"{}\", with config: {:?}",
            gesture_pack_id.clone(),
            gesture_pack_config.clone()
          );

          // TODO: App badges
          let badges = vec![];

          button(
            row(vec![
              column(vec![
                column(vec![
                  text(gesture_pack_config.name).size(18).into(),
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
          .on_press(Message::SetConfiguratorTab(
            ConfiguratorTab::GesturePackDetail(
              repo_id.clone(),
              gesture_pack_id.clone(),
              Box::new(ConfiguratorTab::Gestures),
            ),
          ))
          .into()
        })
        .collect::<Vec<Element<Message>>>(),
    )
    .into(),
  );
}
