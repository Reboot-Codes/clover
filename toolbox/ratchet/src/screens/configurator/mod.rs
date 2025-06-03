use iced::{
  Element,
  Task,
  Theme,
  widget::{
    button,
    column,
    container,
    row,
    scrollable,
    text,
    vertical_space,
  },
};

use crate::{
  Message,
  screens::MoveToScreen,
  util::menu::gen_menu_bar,
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
    let mut elements: Vec<Element<Message>> = vec![];
    let sidebar = vec![
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
      vertical_space().into(),
      button("Instances")
        .on_press(Message::MoveToScreen(MoveToScreen::Welcome))
        .width(iced::Length::Fill)
        .into(),
    ];
    let mut content: Vec<iced::Element<crate::Message>> = Vec::new();

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
      container(
        column(sidebar)
          .height(iced::Length::Fill)
          .width(128)
          .spacing(12)
          .padding(12),
      )
      .style(|theme: &Theme| {
        let palette = theme.extended_palette();

        iced::widget::container::Style::default().background(palette.primary.base.text)
      })
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
    column(vec![
      gen_menu_bar(),
      row(elements)
        .height(iced::Length::Fill)
        .width(iced::Length::Fill)
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
      Message::SetConfiguratorTab(tab) => {
        self.tab = tab;
        Action::None
      }
      Message::None => Action::None,
    }
  }
}
