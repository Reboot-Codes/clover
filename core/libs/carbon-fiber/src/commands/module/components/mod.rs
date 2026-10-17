pub mod indicators;
pub mod movement;

use crate::commands::models::components::{
  ComponentFaliure,
  ComponentTarget,
  GestureFlags,
  GestureTargetCommand,
};

#[derive(Debug, Clone, Copy)]
pub struct StagedTarget {
  pub transaction_id: [u8; 16],
  pub target: ComponentTarget,
  pub transition_ms: u16,
  pub flags: Option<GestureFlags>,
}

/// In memory state tracker for Command Components.
#[derive(Debug, Clone, Copy)]
pub struct CommandComponentPipeline {
  /// Used to backtrack to the previous state of the module.
  pub prev_target: Option<StagedTarget>,
  /// The current target,
  pub current_target: Option<StagedTarget>,
  pub next_target: Option<StagedTarget>,
  pub elapsed_ms: u32,
  pub paused: bool,
}

impl CommandComponentPipeline {
  pub const fn new() -> Self {
    Self {
      prev_target: None,
      current_target: None,
      next_target: None,
      elapsed_ms: 0,
      paused: false,
    }
  }

  /// Submit a new target from the CAN bus.
  /// Returns Ok(true) if queued into `next`, Ok(false) if loaded directly into `current`,
  /// or Err if both current and next are already occupied.
  pub fn push(
    &mut self,
    cmd: GestureTargetCommand,
    tx_id: [u8; 16],
  ) -> Result<bool, ComponentFaliure> {
    let staged = StagedTarget {
      transaction_id: tx_id,
      target: cmd.target,
      transition_ms: cmd.transition_ms,
      flags: cmd.flags,
    };

    if self.current_target.is_none() {
      // Cold start: directly engage target
      self.current_target = Some(staged);
      self.elapsed_ms = 0;
      Ok(false)
    } else if self.next_target.is_none() {
      // Buffer ahead for gapless transition
      self.next_target = Some(staged);
      Ok(true)
    } else {
      // Buffer overrun: Hub sent targets faster than transition window
      Err(ComponentFaliure::LookaheadFull)
    }
  }

  /// Advance the interpolation clock by `dt_ms`.
  /// When `current` reaches its `transition_ms` deadline, it shifts to `prev`
  /// and smoothly promotes `next` into `current`.
  pub fn tick(&mut self, dt_ms: u32) -> PipelineEvent {
    let Some(current) = self.current_target else {
      return PipelineEvent::Idle;
    };

    self.elapsed_ms += dt_ms;

    if self.elapsed_ms >= current.transition_ms as u32 {
      // Reached target pose
      let completed_tx = current.transaction_id;
      self.prev_target = self.current_target.take();

      if let Some(next) = self.next_target.take() {
        // Gapless handoff: promote next immediately
        self.current_target = Some(next);
        self.elapsed_ms = 0;
        PipelineEvent::PromotedNext { completed_tx }
      } else {
        // Held at target waiting for commands
        self.elapsed_ms = current.transition_ms as u32;
        PipelineEvent::ReachedTarget { completed_tx }
      }
    } else {
      PipelineEvent::Interpolating
    }
  }

  /// Cancels forward lookahead and initiates an immediate backtrack to the previous state, or holds exact position if there was no previous state.
  /// Returns the transaction ID of the canceled state to report back over CAN.
  pub fn backtrack(&mut self, return_ms: u16) -> Option<[u8; 16]> {
    // Drop preloaded lookahead
    self.next_target = None;

    if let Some(prev) = self.prev_target {
      let abort_tx = self.current_target.map(|c| c.transaction_id);

      // Invert target: make `prev` the new `current` setpoint
      self.current_target = Some(StagedTarget {
        transaction_id: prev.transaction_id,
        target: prev.target,
        transition_ms: return_ms,
        flags: Some(GestureFlags::Normal),
      });
      self.elapsed_ms = 0;

      abort_tx
    } else {
      // No prior position known; hold current pose
      // TODO: is this a safe behaviour?
      None
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineEvent {
  Idle,
  Interpolating,
  ReachedTarget { completed_tx: [u8; 16] },
  PromotedNext { completed_tx: [u8; 16] },
}
