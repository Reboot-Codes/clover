use iced::{
  Element,
  widget::{
    button,
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

pub fn connection_type(
  wizard_screen: &WizardScreen,
  elements: &mut Vec<Element<Message>>,
  controls: &mut Vec<Element<Message>>,
) {
  elements.push(text("existing connection type").into());

  controls.push(
    button("Back")
      .on_press(crate::Message::SetWizardStep(WizardStep::UseCase))
      .into(),
  );
  controls.push(
    button("Finish")
      .on_press(crate::Message::SetWizardStep(WizardStep::Finishing))
      .into(),
  );
}
