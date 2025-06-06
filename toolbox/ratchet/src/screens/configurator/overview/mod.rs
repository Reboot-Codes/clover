use iced::{
  Element,
  widget::text,
};
use log::debug;

use crate::{
  Message,
  screens::configurator::ConfiguratorScreen,
};

pub fn overview_tab(
  _configurator_screen: &ConfiguratorScreen,
  content: &mut Vec<Element<Message>>,
) {
  debug!("Overview Tab");

  content.push(text("Overview").into());
}
