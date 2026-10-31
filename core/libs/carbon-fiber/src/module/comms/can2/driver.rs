//! Extensible CAN controller driver abstraction.

use embedded_can::Frame;
use embedded_can_interface::{
  AsyncRxFrameIo,
  AsyncTxFrameIo,
};

/// Trait implemented by CAN controller hardware drivers (e.g. MCP2515, on-chip RP2040 PIO, STM32 bxCAN).
pub trait CanDriver {
  type Frame: Frame;
  type Error: core::fmt::Debug;
  type Tx<'a>: AsyncTxFrameIo<Frame = Self::Frame, Error = Self::Error>
  where
    Self: 'a;
  type Rx<'a>: AsyncRxFrameIo<Frame = Self::Frame, Error = Self::Error>
  where
    Self: 'a;

  /// Split the driver into asynchronous TX and RX halves.
  fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>);
}
