//! MCP2515 SPI CAN controller driver implementation.

use core::cell::RefCell;
use core::time::Duration;
use embedded_can_interface::{
  AsyncRxFrameIo,
  AsyncTxFrameIo,
  RxFrameIo,
  TxFrameIo,
};
use embedded_hal::spi::SpiDevice;
use mcp2515::{
  CanSpeed,
  MCP2515,
  McpSpeed,
  Settings,
  frame::CanFrame,
  regs::OpMode,
};

use super::driver::CanDriver;

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

pub struct McpTx<'a, SPI>(pub &'a RefCell<MCP2515<SPI>>);

pub struct McpRx<'a, SPI, INT> {
  pub driver: &'a RefCell<MCP2515<SPI>>,
  pub int_pin: &'a mut INT,
  pub buffered: Option<CanFrame>,
}

#[allow(async_fn_in_trait)]
pub trait McpIntPin {
  fn is_active(&mut self) -> bool;
  async fn wait_active(&mut self);
}

#[derive(Debug, Clone, Copy, Default)]
pub struct NoIntPin;

impl McpIntPin for NoIntPin {
  fn is_active(&mut self) -> bool {
    true
  }

  async fn wait_active(&mut self) {
    #[cfg(feature = "embassy-time")]
    embassy_time::Timer::after_micros(250).await;
  }
}

impl<P: embedded_hal::digital::InputPin> McpIntPin for P {
  fn is_active(&mut self) -> bool {
    self.is_low().unwrap_or(false)
  }

  async fn wait_active(&mut self) {
    while !self.is_active() {
      #[cfg(feature = "embassy-time")]
      embassy_time::Timer::after_micros(250).await;
      #[cfg(not(feature = "embassy-time"))]
      break;
    }
  }
}

impl<'a, SPI: SpiDevice> TxFrameIo for McpTx<'a, SPI> {
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  fn try_send(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
    let mut driver = self.0.borrow_mut();
    let buf = match driver.find_free_tx_buf() {
      Ok(b) => b,
      Err(mcp2515::error::Error::TxBusy) => return Err(CanIoError::WouldBlock),
      Err(e) => return Err(CanIoError::Driver(e)),
    };

    if let Err(e) = driver.write_register_addr(&[buf.ctrl()], &[0]) {
      return Err(CanIoError::Driver(e));
    }

    match driver.send_message_via_buffer(buf, frame.clone()) {
      Ok(()) => Ok(()),
      Err(mcp2515::error::Error::TxBusy) | Err(mcp2515::error::Error::TxFailed) => {
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
        Err(CanIoError::WouldBlock) => {}
        Err(e) => return Err(e),
      }
    }
  }

  fn send_timeout(&mut self, frame: &Self::Frame, _timeout: Duration) -> Result<(), Self::Error> {
    #[cfg(feature = "embassy-time")]
    {
      let start = embassy_time::Instant::now();
      let timeout_micros = _timeout.as_micros() as u64;

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
    #[cfg(not(feature = "embassy-time"))]
    TxFrameIo::send(self, frame)
  }
}

impl<'a, SPI: SpiDevice> AsyncTxFrameIo for McpTx<'a, SPI> {
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  async fn send(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
    loop {
      match self.try_send(frame) {
        Ok(()) => return Ok(()),
        Err(CanIoError::WouldBlock) => {
          #[cfg(feature = "embassy-time")]
          embassy_time::Timer::after_micros(250).await;
        }
        Err(e) => return Err(e),
      }
    }
  }

  async fn send_timeout(
    &mut self,
    frame: &Self::Frame,
    _timeout: Duration,
  ) -> Result<(), Self::Error> {
    #[cfg(feature = "embassy-time")]
    {
      let dur = embassy_time::Duration::from_micros(_timeout.as_micros() as u64);
      embassy_time::with_timeout(dur, AsyncTxFrameIo::send(self, frame))
        .await
        .map_err(|_| CanIoError::Timeout)?
    }
    #[cfg(not(feature = "embassy-time"))]
    AsyncTxFrameIo::send(self, frame).await
  }
}

impl<'a, SPI: SpiDevice, INT: McpIntPin> RxFrameIo for McpRx<'a, SPI, INT> {
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
          while !self.int_pin.is_active() {}
        }
        Err(e) => return Err(e),
      }
    }
  }

  fn recv_timeout(&mut self, _timeout: Duration) -> Result<Self::Frame, Self::Error> {
    #[cfg(feature = "embassy-time")]
    {
      let start = embassy_time::Instant::now();
      let timeout_micros = _timeout.as_micros() as u64;

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
    #[cfg(not(feature = "embassy-time"))]
    RxFrameIo::recv(self)
  }

  fn wait_not_empty(&mut self) -> Result<(), Self::Error> {
    if self.buffered.is_some() {
      return Ok(());
    }

    while !self.int_pin.is_active() {}
    let frame = self.try_recv()?;
    self.buffered = Some(frame);
    Ok(())
  }
}

impl<'a, SPI: SpiDevice, INT: McpIntPin> AsyncRxFrameIo for McpRx<'a, SPI, INT> {
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;

  async fn recv(&mut self) -> Result<Self::Frame, Self::Error> {
    loop {
      match self.try_recv() {
        Ok(frame) => return Ok(frame),
        Err(CanIoError::WouldBlock) => {
          if self.int_pin.is_active() {
            let _ = self
              .driver
              .borrow_mut()
              .write_register(mcp2515::regs::CanIntf::new());
            #[cfg(feature = "embassy-time")]
            embassy_time::Timer::after_micros(250).await;
          } else {
            self.int_pin.wait_active().await;
          }
        }
        Err(e) => return Err(e),
      }
    }
  }

  async fn recv_timeout(&mut self, _timeout: Duration) -> Result<Self::Frame, Self::Error> {
    #[cfg(feature = "embassy-time")]
    {
      let dur = embassy_time::Duration::from_micros(_timeout.as_micros() as u64);
      embassy_time::with_timeout(dur, AsyncRxFrameIo::recv(self))
        .await
        .map_err(|_| CanIoError::Timeout)?
    }
    #[cfg(not(feature = "embassy-time"))]
    AsyncRxFrameIo::recv(self).await
  }

  async fn wait_not_empty(&mut self) -> Result<(), Self::Error> {
    if self.buffered.is_some() {
      return Ok(());
    }

    let frame = AsyncRxFrameIo::recv(self).await?;
    self.buffered = Some(frame);
    Ok(())
  }
}

pub struct Mcp2515Driver<SPI, INT = ()> {
  pub driver: RefCell<MCP2515<SPI>>,
  pub int_pin: INT,
}

impl<SPI: SpiDevice, INT: McpIntPin> Mcp2515Driver<SPI, INT> {
  pub fn new(spi: SPI, int_pin: INT) -> Result<Self, mcp2515::error::Error<SPI::Error>> {
    let settings = Settings {
      mode: OpMode::Normal,
      can_speed: CanSpeed::Kbps125,
      mcp_speed: McpSpeed::MHz8,
      clkout_en: false,
    };
    Self::with_settings(spi, int_pin, settings)
  }

  pub fn with_settings(
    spi: SPI,
    int_pin: INT,
    settings: Settings,
  ) -> Result<Self, mcp2515::error::Error<SPI::Error>> {
    #[cfg(feature = "embassy-time")]
    let mcp = {
      let mut mcp = MCP2515::new(spi);
      let mut delay = embassy_time::Delay;
      mcp.init(&mut delay, settings)?;
      mcp
    };
    #[cfg(not(feature = "embassy-time"))]
    let mcp = {
      let _ = settings;
      MCP2515::new(spi)
    };
    Ok(Self {
      driver: RefCell::new(mcp),
      int_pin,
    })
  }

  pub fn from_mcp(mcp: MCP2515<SPI>, int_pin: INT) -> Self {
    Self {
      driver: RefCell::new(mcp),
      int_pin,
    }
  }
}

impl<SPI: SpiDevice, INT: McpIntPin> CanDriver for Mcp2515Driver<SPI, INT> {
  type Frame = CanFrame;
  type Error = CanIoError<mcp2515::error::Error<SPI::Error>>;
  type Tx<'a>
    = McpTx<'a, SPI>
  where
    Self: 'a;
  type Rx<'a>
    = McpRx<'a, SPI, INT>
  where
    Self: 'a;

  fn split(&mut self) -> (Self::Tx<'_>, Self::Rx<'_>) {
    (
      McpTx(&self.driver),
      McpRx {
        driver: &self.driver,
        int_pin: &mut self.int_pin,
        buffered: None,
      },
    )
  }
}
