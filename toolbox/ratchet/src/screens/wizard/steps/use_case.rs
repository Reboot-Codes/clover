use iced::{
  Element,
  widget::{
    button,
    column,
    row,
    text,
  },
};
use iced_fonts::{
  REQUIRED_FONT,
  required::{
    RequiredIcons,
    icon_to_string,
  },
};

use crate::{
  Message,
  screens::wizard::{
    WizardScreen,
    WizardStep,
  },
};

#[derive(Debug, Clone)]
pub enum UseCase {
  Fursuit,
  Cosplay,
  Cyborg,
}

pub fn use_case(
  _wizard_screen: &WizardScreen,
  elements: &mut Vec<Element<Message>>,
  controls: &mut Vec<Element<Message>>,
) {
  elements.push(text("Choose a Use Case").size(20).into());

  elements.push(text("To begin planning your setup, you'll want to figure out what you'll primarily be using Clover for.").into());

  let bullets: Vec<(&str, Element<Message>, UseCase)> = vec![
    ("Fursuit", text("In this context, the fursuit can have many modules and features added to enable prolonged use of the fursuit; including expression replication, utility applications, and hardware regulation. You don't have to wear your suit all day, however, that is 100% possible with this option").into(), UseCase::Fursuit),
    ("Cosplay", text("Clover will have multiple utility gestures configured for cosplaying at short, and longer intervals").into(), UseCase::Cosplay)
  ];
  elements.push(
    column(
      bullets
        .into_iter()
        .map(|(label, element, chosen_use_case)| {
          row(vec![
            text(icon_to_string(RequiredIcons::CaretRightFill))
              .font(REQUIRED_FONT)
              .width(iced::Length::Shrink)
              .align_y(iced::alignment::Vertical::Center)
              .into(),
            text(label).into(),
            column(vec![element, button("Use!").into()]).into(),
          ])
          .into()
        })
        .collect::<Vec<Element<Message>>>(),
    )
    .spacing(6)
    .into(),
  );

  controls.push(button("Back").into());
  controls.push(
    button("Forward")
      .on_press(crate::Message::SetWizardStep(WizardStep::ConnectionType))
      .into(),
  );
}
