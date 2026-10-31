pub mod camera;
pub mod microphone;
pub mod sensor;

use crate::alerts::module::sensitivity::EdgeTrigger;

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlertComponentPipeline {
  pub elapsed_since_sample: u32,
  pub debounce_timer: u32,
  pub debounce_ms: u32,
  pub uptime_ms: u32,
  pub last_raw: bool,
  pub stable_state: bool,
  pub last_emitted_state: Option<bool>,
  pub edge_trigger: EdgeTrigger,
}

impl AlertComponentPipeline {
  pub const fn new() -> Self {
    Self {
      elapsed_since_sample: 0,
      debounce_timer: 0,
      debounce_ms: 20,
      uptime_ms: 0,
      last_raw: false,
      stable_state: false,
      last_emitted_state: None,
      edge_trigger: EdgeTrigger::Both,
    }
  }

  pub fn with_edge(mut self, edge: EdgeTrigger) -> Self {
    self.edge_trigger = edge;
    self
  }

  pub fn with_debounce(mut self, debounce_ms: u32) -> Self {
    self.debounce_ms = debounce_ms;
    self
  }

  pub fn tick_digital(
    &mut self,
    dt_ms: u32,
    raw_active: bool,
    debounce_ms: u32,
  ) -> Option<bool> {
    self.uptime_ms = self.uptime_ms.wrapping_add(dt_ms);

    if raw_active != self.last_raw {
      self.last_raw = raw_active;
      self.debounce_timer = 0;
    }

    self.debounce_timer += dt_ms;
    if (self.debounce_timer >= debounce_ms || debounce_ms == 0) && raw_active != self.stable_state {
      self.stable_state = raw_active;
      let should_trigger = match self.edge_trigger {
        EdgeTrigger::Rising => self.stable_state,
        EdgeTrigger::Falling => !self.stable_state,
        EdgeTrigger::Both => true,
      };

      if should_trigger {
        self.last_emitted_state = Some(self.stable_state);
        return Some(self.stable_state);
      }
    }
    None
  }
}
