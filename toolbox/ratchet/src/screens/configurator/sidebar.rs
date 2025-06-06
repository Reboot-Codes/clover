use iced::{
  Element,
  Padding,
  alignment,
  widget::{
    button,
    column,
    horizontal_space,
    row,
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

use crate::{
  Message,
  screens::{
    MoveToScreen,
    configurator::{
      ConfiguratorScreen,
      ConfiguratorTab,
    },
  },
};

// TODO: Add button to sidebar when a main repo is set, then toggle sidebar mode between repo and instance focuses
pub fn gen_sidebar(configurator_screen: &ConfiguratorScreen) -> Vec<Element<Message>> {
  vec![
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
      if configurator_screen.tab != ConfiguratorTab::Overview {
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
      if configurator_screen.tab != ConfiguratorTab::Modules {
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
      if configurator_screen.tab != ConfiguratorTab::Gestures {
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
      if configurator_screen.tab != ConfiguratorTab::Apps {
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
      if configurator_screen.tab != ConfiguratorTab::Repos {
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
  ]
}
