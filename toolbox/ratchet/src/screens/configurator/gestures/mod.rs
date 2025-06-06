use carbon_steel::repos::GesturePack;
use iced::{
  Element,
  widget::text,
};
use log::debug;

use crate::{
  Message,
  screens::configurator::ConfiguratorScreen,
};

pub fn gestures_tab(
  _configurator_screen: &ConfiguratorScreen,
  content: &mut Vec<Element<Message>>,
) {
  debug!("Gestures Tab");

  content.push(text("Gestures").into());
}
