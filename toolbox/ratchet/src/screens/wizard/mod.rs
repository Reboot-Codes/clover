use crate::screens::TopLevelScreen;

#[derive(Debug, Clone)]
pub struct WizardScreen {
  step: WizardStep,
}

impl TopLevelScreen for WizardScreen {
  fn view(&self, state: &crate::MainAppState) -> iced::Element<crate::Message> {
    todo!()
  }

  fn update(&mut self, message: crate::Message) {
    todo!()
  }
}

#[derive(Debug, Clone, Copy)]
pub enum WizardStartingPoints {}

#[derive(Debug, Clone, Copy)]
pub enum WizardStep {}
