pub mod apps;
pub mod gestures;
pub mod modules;
pub mod overview;
pub mod repos;
pub mod sidebar;

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
  border::Radius,
  widget::{
    column,
    container,
    row,
    scrollable,
  },
};
use log::error;
use std::collections::HashMap;

use crate::{
  Message,
  screens::{
    ConfiguratorFocus,
    MoveToScreen,
    configurator::{
      apps::{
        apps_tab,
        detail::app_detail_tab,
      },
      gestures::{
        gestures_tab,
        pack_detail::gesture_pack_detail_tab,
      },
      modules::{
        detail::module_detail_tab,
        modules_tab,
      },
      overview::overview_tab,
      repos::{
        detail::repo_detail_tab,
        repos_tab,
      },
      sidebar::gen_sidebar,
    },
  },
  util::menu::gen_menu_bar,
};

#[derive(Debug, Clone)]
pub struct ConfiguratorScreen {
  pub instance_id: Option<String>,
  pub repos: HashMap<String, Repo>,
  pub tab: ConfiguratorTab,
  pub current_repo: Option<String>,
  pub focused_on_repo: bool,
}

// TODO: Setup setting to save this in app state or use the default here.
#[derive(Debug, Clone, Default, PartialEq)]
pub enum ConfiguratorTab {
  #[default]
  Overview,
  None,
  Modules,
  ModuleDetail(String, String, Box<ConfiguratorTab>),
  Gestures,
  GesturePackDetail(String, String, Box<ConfiguratorTab>),
  Apps,
  AppDetail(String, String, Box<ConfiguratorTab>),
  Repos,
  RepoDetail(String, Box<ConfiguratorTab>),
}

#[derive(Debug, Clone, Default)]
pub enum Action {
  #[default]
  None,
  MoveToScreen(MoveToScreen),
  SetTab(ConfiguratorTab),
}

impl ConfiguratorScreen {
  pub fn new(id: ConfiguratorFocus) -> (Self, Task<crate::Message>) {
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
        installed: true,
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
      (match id {
        ConfiguratorFocus::Instance(instance_id) => ConfiguratorScreen {
          instance_id: Some(instance_id),
          // TODO: load repos from connection instead of hardcoding them, obviously.
          repos,
          tab: Default::default(),
          current_repo: None,
          focused_on_repo: false,
        },
        ConfiguratorFocus::Repo(repo_id) => ConfiguratorScreen {
          instance_id: None,
          // TODO: load repos from connection instead of hardcoding them, obviously.
          repos,
          tab: ConfiguratorTab::RepoDetail(repo_id.clone(), Box::new(ConfiguratorTab::None)),
          current_repo: Some(repo_id.clone()),
          focused_on_repo: true,
        },
      }),
      Task::none(),
    )
  }

  pub fn view(&self, _state: &crate::MainAppState) -> iced::Element<crate::Message> {
    let mut elements: Vec<Element<Message>> = vec![];
    let sidebar = gen_sidebar(&self);
    let mut content: Vec<iced::Element<crate::Message>> = Vec::new();

    match &self.tab {
      ConfiguratorTab::Overview => overview_tab(self, &mut content),
      ConfiguratorTab::Modules => modules_tab(self, &mut content),
      ConfiguratorTab::Gestures => gestures_tab(self, &mut content),
      ConfiguratorTab::Apps => apps_tab(self, &mut content),
      ConfiguratorTab::Repos => repos_tab(self, &mut content),
      ConfiguratorTab::RepoDetail(repo_id, prev_screen) => {
        repo_detail_tab(self, &mut content, repo_id, prev_screen)
      }
      ConfiguratorTab::AppDetail(repo_id, app_id, prev_screen) => {
        app_detail_tab(self, &mut content, repo_id, app_id, prev_screen)
      }
      ConfiguratorTab::ModuleDetail(repo_id, module_id, prev_screen) => {
        module_detail_tab(self, &mut content, repo_id, module_id, prev_screen)
      }
      ConfiguratorTab::GesturePackDetail(repo_id, module_id, prev_screen) => {
        gesture_pack_detail_tab(self, &mut content, repo_id, module_id, prev_screen)
      }
      ConfiguratorTab::None => {}
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
        ConfiguratorTab::RepoDetail(ref repo_id, _) => match self.repos.get(repo_id) {
          Some(_) => {
            self.focused_on_repo = true;
            self.current_repo = Some(repo_id.to_string());

            self.tab = tab.clone();
            Action::None
          }
          None => {
            error!("Repo ID set, but it does not exist!");
            // TODO: show pop-up!
            Action::SetTab(ConfiguratorTab::Repos)
          }
        },
        ConfiguratorTab::AppDetail(ref repo_id, ref app_id, _) => match self.repos.get(repo_id) {
          Some(repo) => {
            match repo.apps.get(app_id) {
              Some(_) => {
                self.tab = tab.clone();
                Action::None
              }
              None => {
                error!("Found repo, but the referenced app does not exist!");
                // TODO: show pop-up!
                Action::SetTab(ConfiguratorTab::Repos)
              }
            }
          }
          None => {
            error!("Repo ID set, but it does not exist!");
            // TODO: show pop-up!
            Action::SetTab(ConfiguratorTab::Repos)
          }
        },
        ConfiguratorTab::Overview => {
          // Reset the focus
          self.focused_on_repo = false;
          self.current_repo = None;

          self.tab = tab;
          Action::None
        }
        ConfiguratorTab::Repos => {
          // Reset the focus
          self.focused_on_repo = false;
          self.current_repo = None;

          self.tab = tab;
          Action::None
        }
        _ => {
          self.tab = tab;
          Action::None
        }
      },
      Message::None => Action::None,
    }
  }
}
