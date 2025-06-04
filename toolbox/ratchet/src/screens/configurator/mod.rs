use std::collections::HashMap;

use anyhow::Context;
use carbon_steel::repos::{
  App,
  GesturePack,
  Module,
  Repo,
};
use iced::{
  Border,
  Element,
  Padding,
  Task,
  Theme,
  alignment,
  border::Radius,
  widget::{
    button,
    column,
    container,
    horizontal_space,
    row,
    scrollable,
    text,
    vertical_space,
  },
};
use iced_aw::badge;
use iced_fonts::{
  REQUIRED_FONT,
  required::{
    RequiredIcons,
    icon_to_string,
  },
};
use log::{
  debug,
  error,
};

use crate::{
  Message,
  screens::MoveToScreen,
  util::menu::gen_menu_bar,
};

#[derive(Debug, Clone)]
pub struct ConfiguratorScreen {
  pub instance_id: String,
  pub repos: HashMap<String, Repo>,
  pub tab: ConfiguratorTab,
  pub current_repo: Option<String>,
}

// TODO: Setup setting to save this in app state or use the default here.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfiguratorTab {
  #[default]
  Overview,
  Modules,
  Gestures,
  Apps,
  Repos,
  RepoDetail(String),
}

#[derive(Debug, Clone, Default)]
pub enum Action {
  #[default]
  None,
  MoveToScreen(MoveToScreen),
  SetTab(ConfiguratorTab),
}

impl ConfiguratorScreen {
  pub fn new(id: String) -> (Self, Task<crate::Message>) {
    let mut repos = HashMap::new();

    let mut gesture_packs = HashMap::new();

    gesture_packs.insert(
      "com.reboot-codes.clover.CORE.default".to_string(),
      GesturePack {
        name: "Default Gesture Pack".to_string(),
      },
    );

    let mut modules = HashMap::new();

    modules.insert(
      "com.reboot-codes.clover.CORE.three-phase-tail".to_string(),
      Module {
        name: "Three Phase Tail".to_string(),
      },
    );

    let mut apps = HashMap::new();

    apps.insert(
      "com.reboot-codes.clover.CORE.home".to_string(),
      App {
        name: "Home".to_string(),
      },
    );

    repos.insert(
      "com.reboot-codes.clover.CORE".to_string(),
      Repo {
        name: "Clover CORE (unstable)".to_string(),
        url: "https://codeberg.org/Reboot-Codes/clover".to_string(),
        gesture_packs,
        modules,
        apps,
      },
    );

    (
      ConfiguratorScreen {
        instance_id: id,
        // TODO: load repos from connection
        repos,
        tab: Default::default(),
        current_repo: None,
      },
      Task::none(),
    )
  }

  pub fn view(&self, _state: &crate::MainAppState) -> iced::Element<crate::Message> {
    let mut elements: Vec<Element<Message>> = vec![];
    let sidebar = vec![
      column(vec![
        text("$ConnectionName").size(24).into(),
        badge("Connected")
          .style(iced_aw::style::badge::success)
          .into(),
      ])
      .spacing(12)
      .padding(Padding {
        top: 0.0,
        bottom: 12.0,
        left: 0.0,
        right: 0.0,
      })
      .into(),
      button(row([
        text("Overview").into(),
        horizontal_space().into(),
        // TODO: Replace with actual icon
        text(icon_to_string(RequiredIcons::CaretRightFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
      ]))
      .on_press_maybe({
        if self.tab != ConfiguratorTab::Overview {
          Some(Message::SetConfiguratorTab(ConfiguratorTab::Overview))
        } else {
          None
        }
      })
      .width(iced::Length::Fill)
      .into(),
      button(row([
        text("Modules").into(),
        horizontal_space().into(),
        // TODO: Replace with actual icon
        text(icon_to_string(RequiredIcons::CaretRightFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
      ]))
      .on_press_maybe({
        if self.tab != ConfiguratorTab::Modules {
          Some(Message::SetConfiguratorTab(ConfiguratorTab::Modules))
        } else {
          None
        }
      })
      .width(iced::Length::Fill)
      .into(),
      button(row([
        text("Gestures").into(),
        horizontal_space().into(),
        // TODO: Replace with actual icon
        text(icon_to_string(RequiredIcons::CaretRightFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
      ]))
      .on_press_maybe({
        if self.tab != ConfiguratorTab::Gestures {
          Some(Message::SetConfiguratorTab(ConfiguratorTab::Gestures))
        } else {
          None
        }
      })
      .width(iced::Length::Fill)
      .into(),
      button(row([
        text("Apps").into(),
        horizontal_space().into(),
        // TODO: Replace with actual icon
        text(icon_to_string(RequiredIcons::CaretRightFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
      ]))
      .on_press_maybe({
        if self.tab != ConfiguratorTab::Apps {
          Some(Message::SetConfiguratorTab(ConfiguratorTab::Apps))
        } else {
          None
        }
      })
      .width(iced::Length::Fill)
      .into(),
      button(row([
        text("Repos").into(),
        horizontal_space().into(),
        // TODO: Replace with actual icon
        text(icon_to_string(RequiredIcons::CaretRightFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
      ]))
      .on_press_maybe({
        if self.tab != ConfiguratorTab::Repos {
          Some(Message::SetConfiguratorTab(ConfiguratorTab::Repos))
        } else {
          None
        }
      })
      .width(iced::Length::Fill)
      .into(),
      vertical_space().into(),
      button(row([
        text("Instances").into(),
        horizontal_space().into(),
        // TODO: Replace with actual icon
        text(icon_to_string(RequiredIcons::CaretRightFill))
          .font(REQUIRED_FONT)
          .width(iced::Length::Shrink)
          .align_y(alignment::Vertical::Center)
          .into(),
      ]))
      .on_press(Message::MoveToScreen(MoveToScreen::Welcome))
      .width(iced::Length::Fill)
      .into(),
    ];
    let mut content: Vec<iced::Element<crate::Message>> = Vec::new();

    match &self.tab {
      ConfiguratorTab::Overview => {
        debug!("Overview Tab");
        content.push(text("Overview").into());
      }
      ConfiguratorTab::Modules => {
        debug!("Modules Tab");
        content.push(text("Modules").into());
      }
      ConfiguratorTab::Gestures => {
        debug!("Gestures Tab");

        content.push(text("Gestures").into());
      }
      ConfiguratorTab::Apps => {
        debug!("Apps Tab");
        content.push(text("Apps").into());
      }
      ConfiguratorTab::Repos => {
        debug!("All Repos Tab");

        content.push(text("Repositories").size(24).into());

        let repos = &self.repos.clone();

        content.push(
          column(
            <HashMap<String, Repo> as Clone>::clone(&repos)
              .into_iter()
              .map(|(id, repo)| {
                debug!("Found \"{}\", with config: {:?}", id.clone(), repo.clone());

                let mut badges = vec![];

                if !repo.modules.is_empty() {
                  badges.push(badge("Modules").into());
                }

                if !repo.gesture_packs.is_empty() {
                  badges.push(badge("Gesture Packs").into());
                }

                if !repo.apps.is_empty() {
                  badges.push(badge("Apps").into());
                }

                button(
                  row(vec![
                    column(vec![
                      column(vec![
                        text(repo.name).size(18).into(),
                        text(repo.url).size(14).into(),
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
                )))
                .into()
              })
              .collect::<Vec<Element<Message>>>(),
          )
          .into(),
        );
      }
      ConfiguratorTab::RepoDetail(repo_id) => {
        let repo = self.repos.get(repo_id).with_context(|| format!("ConfiguratorScreen::update() should've ensured that \"{}\" existed in the repo cache", repo_id)).unwrap();

        content.push(
          button(row(vec![
            text(icon_to_string(RequiredIcons::CaretLeftFill))
              .font(REQUIRED_FONT)
              .width(iced::Length::Shrink)
              .align_y(alignment::Vertical::Center)
              .into(),
            text("Back to All Repos").into(),
          ]))
          .on_press(Message::SetConfiguratorTab(ConfiguratorTab::Repos))
          .into(),
        );

        content.push(text(repo.name.clone()).size(24).into());
      }
    }

    elements.push(
      container(
        column(sidebar)
          .height(iced::Length::Fill)
          .width(175)
          .spacing(12)
          .padding(12),
      )
      .style(|theme: &Theme| {
        let palette = theme.extended_palette();

        iced::widget::container::Style::default()
          .background(palette.primary.base.text)
          .border(Border {
            radius: Radius::from(12),
            ..Default::default()
          })
      })
      .into(),
    );
    elements.push(
      scrollable(
        column(content)
          .width(iced::Length::Fill)
          .padding(Padding {
            left: 6.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
          })
          .spacing(12),
      )
      .width(iced::Length::Fill)
      .into(),
    );
    column(vec![
      gen_menu_bar(),
      row(elements)
        .height(iced::Length::Fill)
        .width(iced::Length::Fill)
        .padding(6)
        .into(),
    ])
    .into()
  }

  pub fn update(&mut self, message: Message) -> Action {
    match message {
      Message::MoveToScreen(target_screen) => match target_screen {
        MoveToScreen::Welcome => Action::MoveToScreen(MoveToScreen::Welcome),
        MoveToScreen::Wizard(starting_point) => {
          Action::MoveToScreen(MoveToScreen::Wizard(starting_point))
        }
        _ => Action::None,
      },
      Message::SetWizardStep(_wizard_step) => Action::None,
      Message::SetConfiguratorTab(tab) => match tab {
        ConfiguratorTab::RepoDetail(ref repo_id) => match self.repos.get(repo_id) {
          Some(_) => {
            self.tab = tab.clone();
            Action::None
          }
          None => {
            error!("Repo ID set, but it does not exist!");
            // TODO: show pop-up!
            Action::SetTab(ConfiguratorTab::Repos)
          }
        },
        _ => {
          self.tab = tab;
          Action::None
        }
      },
      Message::None => Action::None,
    }
  }
}
