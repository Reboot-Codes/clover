use core::cell::RefCell;
use core::time::Duration;
use embassy_rp::gpio::Input;
use embassy_time::{
  Duration as EmbassyDuration,
  Instant,
  with_timeout,
};
use embedded_can_interface::{
  AsyncRxFrameIo,
  AsyncTxFrameIo,
  RxFrameIo,
  TxFrameIo,
};
use embedded_hal_1::spi::SpiDevice;
use mcp2515::{
  MCP2515,
  frame::CanFrame,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CanIoError<E> {
  WouldBlock,
  Timeout,
  Driver(E),
}

impl<E> From<E> for CanIoError<E> {
  fn from(err: E) -> Self {
    Self::Driver(err)
  }
}

pub struct McpTx<'a, SPI>(&'a RefCell<MCP2515<SPI>>);

pub struct McpRx<'a, SPI> {
  pub(crate) driver: &'a RefCell<MCP2515<SPI>>,
  pub(crate) int_pin: Input<'a>,
  pub(crate) buffered: Option<CanFrame>,
}

pub fn split_mcp<'a, SPI>(
  mcp: &'a RefCell<MCP2515<SPI>>,
  int_pin: Input<'a>,
) -> (McpTx<'a, SPI>, McpRx<'a, SPI>) {
  (
    McpTx(mcp),
    McpRx {
      driver: mcp,
      int_pin,
      buffered: None,
    },
  )
}

// ---------------------------------------------------------------------------
// Synchronous TX Adapter
// ---------------------------------------------------------------------------
impl<'a, SPI> TxFrameIo for McpTx<'a, SPI>
where
  SPI: SpiDevice,
{
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  fn try_send(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
    let mut driver = self.0.borrow_mut();
    let buf = match driver.find_free_tx_buf() {
      Ok(b) => b,
      Err(mcp2515::error::Error::TxBusy) => return Err(CanIoError::WouldBlock),
      Err(e) => return Err(CanIoError::Driver(e)),
    };

    // Clear stale error/abort flags in TXBnCTRL so previous ACK/bus errors don't poison subsequent sends
    if let Err(e) = driver.write_register_addr(&[buf.ctrl()], &[0]) {
      return Err(CanIoError::Driver(e));
    }

    match driver.send_message_via_buffer(buf, frame.clone()) {
      Ok(()) => Ok(()),
      Err(mcp2515::error::Error::TxBusy) | Err(mcp2515::error::Error::TxFailed) => {
        // Abort/clear the failed buffer so it doesn't permanently wedge or leak TXREQ=1
        let _ = driver.write_register_addr(&[buf.ctrl()], &[0]);
        Err(CanIoError::WouldBlock)
      }
      Err(e) => Err(CanIoError::Driver(e)),
    }
  }

  fn send(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
    loop {
      match self.try_send(frame) {
        Ok(()) => return Ok(()),
        Err(CanIoError::WouldBlock) => {} // Synchronous spin
        Err(e) => return Err(e),
      }
    }
  }

  fn send_timeout(&mut self, frame: &Self::Frame, timeout: Duration) -> Result<(), Self::Error> {
    let start = Instant::now();
    let timeout_micros = timeout.as_micros() as u64;

    loop {
      match self.try_send(frame) {
        Ok(()) => return Ok(()),
        Err(CanIoError::WouldBlock) => {
          if start.elapsed().as_micros() >= timeout_micros {
            return Err(CanIoError::Timeout);
          }
        }
        Err(e) => return Err(e),
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Asynchronous TX Adapter
// ---------------------------------------------------------------------------
impl<'a, SPI> AsyncTxFrameIo for McpTx<'a, SPI>
where
  SPI: SpiDevice,
{
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  async fn send(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
    loop {
      match self.try_send(frame) {
        Ok(()) => return Ok(()),
        // Yield briefly back to the Embassy executor so other tasks run without hammering SPI
        Err(CanIoError::WouldBlock) => embassy_time::Timer::after_micros(500).await,
        Err(e) => return Err(e),
      }
    }
  }

  async fn send_timeout(
    &mut self,
    frame: &Self::Frame,
    timeout: Duration,
  ) -> Result<(), Self::Error> {
    let embassy_dur = EmbassyDuration::from_micros(timeout.as_micros() as u64);
    with_timeout(embassy_dur, AsyncTxFrameIo::send(self, frame))
      .await
      .map_err(|_| CanIoError::Timeout)?
  }
}

// ---------------------------------------------------------------------------
// Synchronous RX Adapter
// ---------------------------------------------------------------------------
impl<'a, SPI> RxFrameIo for McpRx<'a, SPI>
where
  SPI: SpiDevice,
{
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  fn try_recv(&mut self) -> Result<Self::Frame, Self::Error> {
    if let Some(frame) = self.buffered.take() {
      return Ok(frame);
    }

    let mut driver = self.driver.borrow_mut();
    match driver.read_message() {
      Ok(frame) => Ok(frame),
      Err(mcp2515::error::Error::NoMessage) => Err(CanIoError::WouldBlock),
      Err(e) => Err(CanIoError::Driver(e)),
    }
  }

  fn recv(&mut self) -> Result<Self::Frame, Self::Error> {
    loop {
      match self.try_recv() {
        Ok(frame) => return Ok(frame),
        Err(CanIoError::WouldBlock) => {
          // Poll local GPIO line to avoid flooding the shared SPI bus
          while self.int_pin.is_high() {}
        }
        Err(e) => return Err(e),
      }
    }
  }

  fn recv_timeout(&mut self, timeout: Duration) -> Result<Self::Frame, Self::Error> {
    let start = Instant::now();
    let timeout_micros = timeout.as_micros() as u64;

    loop {
      match self.try_recv() {
        Ok(frame) => return Ok(frame),
        Err(CanIoError::WouldBlock) => {
          if start.elapsed().as_micros() >= timeout_micros {
            return Err(CanIoError::Timeout);
          }
        }
        Err(e) => return Err(e),
      }
    }
  }

  fn wait_not_empty(&mut self) -> Result<(), Self::Error> {
    if self.buffered.is_some() {
      return Ok(());
    }

    while self.int_pin.is_high() {}
    let frame = self.try_recv()?;
    self.buffered = Some(frame);
    Ok(())
  }
}

// ---------------------------------------------------------------------------
// Asynchronous RX Adapter
// ---------------------------------------------------------------------------
impl<'a, SPI> AsyncRxFrameIo for McpRx<'a, SPI>
where
  SPI: SpiDevice,
{
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  async fn recv(&mut self) -> Result<Self::Frame, Self::Error> {
    loop {
      // Always attempt reception first (clears buffered frame or any unread HW buffer)
      match self.try_recv() {
        Ok(frame) => return Ok(frame),
        Err(CanIoError::WouldBlock) => {
          if self.int_pin.is_low() {
            // !INT is active low, but no frame was in RXB0/RXB1 (likely bus/message error).
            // Clear interrupt flags so !INT releases and yield to avoid hard-spinning on SPI.
            let _ = self
              .driver
              .borrow_mut()
              .write_register(mcp2515::regs::CanIntf::new());
            embassy_time::Timer::after_micros(500).await;
          } else {
            // Hardware RX buffers are empty; suspend until the MCP2515 !INT line fires low
            self.int_pin.wait_for_low().await;
          }
        }
        Err(e) => return Err(e),
      }
    }
  }

  async fn recv_timeout(&mut self, timeout: Duration) -> Result<Self::Frame, Self::Error> {
    let embassy_dur = EmbassyDuration::from_micros(timeout.as_micros() as u64);
    with_timeout(embassy_dur, AsyncRxFrameIo::recv(self))
      .await
      .map_err(|_| CanIoError::Timeout)?
  }

  async fn wait_not_empty(&mut self) -> Result<(), Self::Error> {
    if self.buffered.is_some() {
      return Ok(());
    }

    // Suspends execution until at least one frame arrives
    let frame = AsyncRxFrameIo::recv(self).await?;
    self.buffered = Some(frame);
    Ok(())
  }
}
