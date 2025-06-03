use iced::{
  Task,
  widget::{
    button,
    column,
    row,
    scrollable,
    text,
    vertical_space,
  },
};

use crate::{
  Message,
  screens::MoveToScreen,
};

#[derive(Debug, Clone)]
pub struct ConfiguratorScreen {
  pub instance_id: String,
  pub tab: ConfiguratorTab,
}

// TODO: Setup setting to save this in app state or use the default here.
#[derive(Debug, Clone, Default, Copy, PartialEq)]
pub enum ConfiguratorTab {
  #[default]
  Overview,
  Modules,
  Apps,
  Repos,
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
    (
      ConfiguratorScreen {
        instance_id: id,
        tab: Default::default(),
      },
      Task::none(),
    )
  }

  pub fn view(&self, _state: &crate::MainAppState) -> iced::Element<crate::Message> {
    let mut elements = Vec::new();
    let mut sidebar = Vec::new();
    let mut content: Vec<iced::Element<crate::Message>> = Vec::new();

    sidebar.push(
      button("Overview")
        .on_press_maybe({
          if self.tab != ConfiguratorTab::Overview {
            Some(Message::SetConfiguratorTab(ConfiguratorTab::Overview))
          } else {
            None
          }
        })
        .width(iced::Length::Fill)
        .into(),
    );
    sidebar.push(
      button("Modules")
        .on_press_maybe({
          if self.tab != ConfiguratorTab::Modules {
            Some(Message::SetConfiguratorTab(ConfiguratorTab::Modules))
          } else {
            None
          }
        })
        .width(iced::Length::Fill)
        .into(),
    );
    sidebar.push(
      button("Apps")
        .on_press_maybe({
          if self.tab != ConfiguratorTab::Apps {
            Some(Message::SetConfiguratorTab(ConfiguratorTab::Apps))
          } else {
            None
          }
        })
        .width(iced::Length::Fill)
        .into(),
    );
    sidebar.push(
      button("Repos")
        .on_press_maybe({
          if self.tab != ConfiguratorTab::Repos {
            Some(Message::SetConfiguratorTab(ConfiguratorTab::Repos))
          } else {
            None
          }
        })
        .width(iced::Length::Fill)
        .into(),
    );
    sidebar.push(vertical_space().into());
    sidebar.push(
      button("Instances")
        .on_press(Message::MoveToScreen(MoveToScreen::Welcome))
        .width(iced::Length::Fill)
        .into(),
    );

    match self.tab {
      ConfiguratorTab::Overview => {
        content.push(text("Overview").into());
      }
      ConfiguratorTab::Modules => {
        content.push(text("Modules").into());
      }
      ConfiguratorTab::Apps => {
        content.push(text("Apps").into());
      }
      ConfiguratorTab::Repos => {
        content.push(text("Repositories").into());
      }
    }

    elements.push(
      column(sidebar)
        .height(iced::Length::Fill)
        .width(256)
        .spacing(12)
        .padding(12)
        .into(),
    );
    elements.push(
      scrollable(
        column(content)
          .width(iced::Length::Fill)
          .spacing(12)
          .padding(12),
      )
      .width(iced::Length::Fill)
      .into(),
    );
    row(elements)
      .height(iced::Length::Fill)
      .width(iced::Length::Fill)
      .into()
  }

  pub fn update(&mut self, message: crate::Message) -> Action {
    match message {
      crate::Message::MoveToScreen(target_screen) => match target_screen {
        super::MoveToScreen::Welcome => Action::MoveToScreen(MoveToScreen::Welcome),
        super::MoveToScreen::Configurator(_id) => Action::None,
        super::MoveToScreen::Wizard(starting_point) => {
          Action::MoveToScreen(MoveToScreen::Wizard(starting_point))
        }
      },
      crate::Message::SetWizardStep(_wizard_step) => Action::None,
      crate::Message::SetConfiguratorTab(tab) => {
        self.tab = tab;
        Action::None
      }
    }
  }
}
