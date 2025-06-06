use carbon_steel::repos::Module;
use iced::{
  Element,
  widget::text,
};
use log::debug;

use crate::{
  Message,
  screens::configurator::ConfiguratorScreen,
};

pub fn modules_tab(_configurator_screen: &ConfiguratorScreen, content: &mut Vec<Element<Message>>) {
  debug!("Modules Tab");

  content.push(text("Modules").into());
}
