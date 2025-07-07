use iced::{
  Element,
  widget::{
    button,
    row,
    text,
  },
};

use crate::{
  Message,
  screens::wizard::{
    WizardScreen,
    WizardStep,
  },
};

pub fn from_scratch_intro(
  _wizard_screen: &WizardScreen,
  elements: &mut Vec<Element<Message>>,
  controls: &mut Vec<Element<Message>>,
) {
  elements.push(text("Creating an Instance from Scratch").size(20).into());
  elements.push(text("Thanks for choosing to use Clover for your bodily extensions! This wizard will guide you in creating a Clover instance from scratch using commonly available, hobbist parts and open source software; then constructing, and flashing your setup with Clover compatible firmware.").into());
  elements.push(
    text("This copy of Ratchet has the full Clover documentation embeded for your convinience! Having the documentation open while you plan, setup, and use Clover for the first time is highly recommended.")
      .into(),
  );
  elements.push(
    row(vec![
      button("Click here to view the embeded docs (new window), or...").into(),
      button("Here to view the online version.").into(),
    ])
    .spacing(6)
    .into(),
  );
  elements.push(text("Press \"Forward\" to continue.").into());

  controls.push(button("Back").into());
  controls.push(
    button("Forward")
      .on_press(crate::Message::SetWizardStep(WizardStep::UseCase))
      .into(),
  );
}
