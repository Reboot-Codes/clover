//! # MCP2515 CAN 2.0 Bus and ISO-TP Implementation

pub mod driver;
pub mod mcp2515;
pub mod runtime;

use embedded_can::{
  Frame,
  Id,
  StandardId,
};

pub use driver::CanDriver;
pub use mcp2515::{
  CanIoError,
  Mcp2515Driver,
  McpIntPin,
  McpRx,
  McpTx,
};
#[cfg(feature = "embassy-time")]
pub use runtime::{
  EmbassyClock,
  EmbassyRuntime,
};

pub const CAN_ID_CRITICAL_BASE: u16 = 0x010;
pub const CAN_ID_SELF_TEST: u16 = 0x080;
pub const CAN_ID_ALERT_INFO_BASE: u16 = 0x100;
pub const CAN_ID_FEEDBACK_BASE: u16 = 0x200;
pub const CAN_ID_TELEMETRY: u16 = 0x400;

pub const DEFAULT_ISOTP_TX_ID: u16 = 0x7E8;
pub const DEFAULT_ISOTP_RX_ID: u16 = 0x7E0;

/// Standard concrete CAN 2.0 frame (<= 8 data bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Can2Frame {
  id: Id,
  dlc: usize,
  data: [u8; 8],
}

#[cfg(feature = "defmt")]
impl defmt::Format for Can2Frame {
  fn format(&self, fmt: defmt::Formatter) {
    defmt::write!(
      fmt,
      "Can2Frame {{ dlc: {}, data: {=[u8]} }}",
      self.dlc,
      self.data()
    );
  }
}

impl Can2Frame {
  pub const fn empty() -> Self {
    Self {
      id: Id::Standard(StandardId::ZERO),
      dlc: 0,
      data: [0; 8],
    }
  }
}

impl Frame for Can2Frame {
  fn new(id: impl Into<Id>, data: &[u8]) -> Option<Self> {
    if data.len() > 8 {
      return None;
    }
    let mut buf = [0u8; 8];
    buf[..data.len()].copy_from_slice(data);
    Some(Self {
      id: id.into(),
      dlc: data.len(),
      data: buf,
    })
  }

  fn new_remote(id: impl Into<Id>, dlc: usize) -> Option<Self> {
    if dlc > 8 {
      return None;
    }
    Some(Self {
      id: id.into(),
      dlc,
      data: [0; 8],
    })
  }

  fn is_extended(&self) -> bool {
    matches!(self.id, Id::Extended(_))
  }

  fn is_remote_frame(&self) -> bool {
    false
  }

  fn id(&self) -> Id {
    self.id
  }

  fn dlc(&self) -> usize {
    self.dlc
  }

  fn data(&self) -> &[u8] {
    &self.data[..self.dlc]
  }
}

/// Pre-allocated ISO-TP reassembly state machine.
#[derive(Debug, Clone, Copy)]
pub struct IsoTpReassembler<const BUFFER_SIZE: usize = 256> {
  buffer: [u8; BUFFER_SIZE],
  total_len: usize,
  received_len: usize,
  expected_seq: u8,
  active: bool,
}

impl<const BUFFER_SIZE: usize> IsoTpReassembler<BUFFER_SIZE> {
  pub const fn new() -> Self {
    Self {
      buffer: [0u8; BUFFER_SIZE],
      total_len: 0,
      received_len: 0,
      expected_seq: 0,
      active: false,
    }
  }

  /// Ingest an incoming 8-byte CAN frame. Returns a reference to the reassembled
  /// buffer slice if this frame completes a packet.
  pub fn ingest<'a>(&'a mut self, frame_data: &[u8]) -> Option<&'a [u8]> {
    if frame_data.is_empty() {
      return None;
    }

    let pci_type = (frame_data[0] >> 4) & 0x0F;
    match pci_type {
      // 0x0: Single Frame (SF)
      0x0 => {
        let len = (frame_data[0] & 0x0F) as usize;
        if len == 0 || len > 7 || len >= frame_data.len() {
          return None;
        }
        self.buffer[..len].copy_from_slice(&frame_data[1..1 + len]);
        self.active = false;
        Some(&self.buffer[..len])
      }
      // 0x1: First Frame (FF)
      0x1 => {
        if frame_data.len() < 8 {
          return None;
        }
        let total = (((frame_data[0] & 0x0F) as usize) << 8) | (frame_data[1] as usize);
        if total > BUFFER_SIZE {
          return None;
        }
        self.total_len = total;
        self.buffer[..6].copy_from_slice(&frame_data[2..8]);
        self.received_len = 6;
        self.expected_seq = 1;
        self.active = true;

        if self.received_len >= self.total_len {
          self.active = false;
          Some(&self.buffer[..self.total_len])
        } else {
          None
        }
      }
      // 0x2: Consecutive Frame (CF)
      0x2 => {
        if !self.active || frame_data.len() < 2 {
          return None;
        }
        let seq = frame_data[0] & 0x0F;
        if seq != (self.expected_seq & 0x0F) {
          self.active = false;
          return None;
        }

        let chunk = &frame_data[1..];
        let bytes_to_copy = chunk.len().min(self.total_len.saturating_sub(self.received_len));
        self.buffer[self.received_len..self.received_len + bytes_to_copy]
          .copy_from_slice(&chunk[..bytes_to_copy]);
        self.received_len += bytes_to_copy;
        self.expected_seq = self.expected_seq.wrapping_add(1);

        if self.received_len >= self.total_len {
          self.active = false;
          Some(&self.buffer[..self.total_len])
        } else {
          None
        }
      }
      _ => None,
    }
  }
}

/// CAN 2.0 communication engine with ISO-TP support and extensible controller backends.
pub struct CAN2<D = ()> {
  pub driver: D,
  pub tx_id: u16,
  pub rx_id: u16,
}

impl<D> CAN2<D> {
  pub fn new(driver: D) -> Self {
    Self {
      driver,
      tx_id: DEFAULT_ISOTP_TX_ID,
      rx_id: DEFAULT_ISOTP_RX_ID,
    }
  }

  pub fn with_iso_tp_ids(mut self, tx_id: u16, rx_id: u16) -> Self {
    self.tx_id = tx_id;
    self.rx_id = rx_id;
    self
  }
}

impl<SPI: embedded_hal::spi::SpiDevice, INT: McpIntPin> CAN2<Mcp2515Driver<SPI, INT>> {
  /// Create a CAN2 instance with the standard MCP2515 controller backend.
  pub fn mcp2515(
    spi: SPI,
    int_pin: INT,
  ) -> Result<Self, ::mcp2515::error::Error<SPI::Error>> {
    let driver = Mcp2515Driver::new(spi, int_pin)?;
    Ok(Self::new(driver))
  }

  /// Create a CAN2 instance with the MCP2515 controller and custom settings.
  pub fn mcp2515_with_settings(
    spi: SPI,
    int_pin: INT,
    settings: ::mcp2515::Settings,
  ) -> Result<Self, ::mcp2515::error::Error<SPI::Error>> {
    let driver = Mcp2515Driver::with_settings(spi, int_pin, settings)?;
    Ok(Self::new(driver))
  }
}

#[cfg(feature = "embassy-time")]
impl<D: CanDriver> CAN2<D> {
  /// Run the CAN 2.0 engine: waits for adoption, then proxies commands and telemetry.
  pub async fn run<const RX_SIZE: usize, const TX_SIZE: usize>(
    &mut self,
    adoption: &mut crate::adoption::module::ModuleAdoptionContext<'_>,
    rx_from_module: embassy_sync::channel::Receiver<
      'static,
      embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
      crate::module::comms::ModuleToComms,
      RX_SIZE,
    >,
    tx_to_module: embassy_sync::channel::Sender<
      'static,
      embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
      crate::module::comms::CommsToModule,
      TX_SIZE,
    >,
  ) -> ! {
    let (tx, rx) = self.driver.split();
    let iso_tp_cfg = can_iso_tp::IsoTpConfig {
      tx_id: embedded_can_interface::Id::Standard(
        embedded_can::StandardId::new(self.tx_id).unwrap_or(embedded_can::StandardId::ZERO),
      ),
      rx_id: embedded_can_interface::Id::Standard(
        embedded_can::StandardId::new(self.rx_id).unwrap_or(embedded_can::StandardId::ZERO),
      ),
      tx_addr: ((self.tx_id & 0xFF) as u8).into(),
      rx_addr: ((self.rx_id & 0xFF) as u8).into(),
      ..Default::default()
    };

    let mut rx_buf = [0u8; 1024];
    let mut node = can_iso_tp::IsoTpAsyncNode::with_clock(
      tx,
      rx,
      iso_tp_cfg,
      runtime::EmbassyClock,
      &mut rx_buf,
    )
    .expect("Failed to initialize IsoTpAsyncNode");

    let rt = runtime::EmbassyRuntime;

    enum PendingAdoptionReply {
      None,
      SendHello,
      SendKeyHimo,
      Confirmed,
      Aborted(crate::adoption::models::AbortionReason),
    }

    // 1. Adoption Handshake Phase
    while adoption.state != crate::adoption::models::AdoptionState::Confirmed {
      let mut pending = PendingAdoptionReply::None;

      let _ = node
        .recv(&rt, core::time::Duration::from_millis(50), &mut |payload| {
          if let Ok(bus_msg) = postcard::from_bytes::<crate::models::BusMessage>(payload) {
            match bus_msg {
              crate::models::BusMessage::Adoption(adopt) => match adopt {
                crate::adoption::models::AdoptionMessage::Probe(probe) => {
                  if adoption.parse_probe(&probe).is_ok() {
                    pending = PendingAdoptionReply::SendHello;
                  }
                }
                crate::adoption::models::AdoptionMessage::KeyHOMI(homi) => {
                  if adoption.parse_key_homi(&homi).is_ok() {
                    pending = PendingAdoptionReply::SendKeyHimo;
                  }
                }
                crate::adoption::models::AdoptionMessage::Confirm(confirm) => {
                  let _ = adoption.parse_confirm(&confirm);
                  pending = PendingAdoptionReply::Confirmed;
                }
                crate::adoption::models::AdoptionMessage::Abortion(abort) => {
                  pending = PendingAdoptionReply::Aborted(abort.reason);
                }
                _ => {}
              },
              _ => {}
            }
          }
        })
        .await;

      match pending {
        PendingAdoptionReply::SendHello => {
          let hello = adoption.gen_hello();
          let msg = crate::models::BusMessage::Adoption(
            crate::adoption::models::AdoptionMessage::Hello(hello),
          );
          let mut send_buf = [0u8; 256];
          if let Ok(bytes) = postcard::to_slice(&msg, &mut send_buf) {
            let _ = node
              .send(&rt, bytes, core::time::Duration::from_millis(250))
              .await;
          }
        }
        PendingAdoptionReply::SendKeyHimo => {
          let himo = adoption.gen_key_himo();
          let msg = crate::models::BusMessage::Adoption(
            crate::adoption::models::AdoptionMessage::KeyHIMO(himo),
          );
          let mut send_buf = [0u8; 256];
          if let Ok(bytes) = postcard::to_slice(&msg, &mut send_buf) {
            let _ = node
              .send(&rt, bytes, core::time::Duration::from_millis(250))
              .await;
          }
        }
        PendingAdoptionReply::Aborted(reason) => {
          adoption.state = crate::adoption::models::AdoptionState::Aborted(reason);
        }
        _ => {}
      }
    }

    // 2. Operational Phase: proxy messages between Core 0 and CloverHub
    loop {
      // Drain messages from Core 0 and transmit to CloverHub over ISO-TP
      while let Ok(msg) = rx_from_module.try_receive() {
        let mut raw_buf = [0u8; 64];
        if let Ok(raw_slice) = postcard::to_slice(&msg, &mut raw_buf) {
          let bus_msg = crate::models::BusMessage::Content(crate::models::RawContentMessage {
            nonce: [0u8; 12],
            data: raw_slice,
            hmac: [0u8; 32],
          });
          let mut send_buf = [0u8; 128];
          if let Ok(bytes) = postcard::to_slice(&bus_msg, &mut send_buf) {
            let _ = node
              .send(&rt, bytes, core::time::Duration::from_millis(100))
              .await;
          }
        }
      }

      // Receive incoming ISO-TP messages from CloverHub
      let mut dispatch_to_core0: Option<crate::module::comms::CommsToModule> = None;
      let _ = node
        .recv(&rt, core::time::Duration::from_millis(20), &mut |payload| {
          if let Ok(bus_msg) = postcard::from_bytes::<crate::models::BusMessage>(payload) {
            match bus_msg {
              crate::models::BusMessage::Content(raw) => {
                if let Ok(cmd) = postcard::from_bytes::<crate::commands::models::components::GestureTargetCommand>(raw.data) {
                  dispatch_to_core0 = Some(crate::module::comms::CommsToModule::DispatchTarget {
                    component_id: cmd.component_id,
                    target: cmd.target,
                    transition_ms: cmd.transition_ms,
                    flags: cmd.flags,
                    tx_id: [0u8; 16],
                  });
                } else if let Ok(dispatch) = postcard::from_bytes::<crate::module::comms::CommsToModule>(raw.data) {
                  dispatch_to_core0 = Some(dispatch);
                }
              }
              _ => {}
            }
          } else if let Ok(cmd) = postcard::from_bytes::<crate::commands::models::components::GestureTargetCommand>(payload) {
            dispatch_to_core0 = Some(crate::module::comms::CommsToModule::DispatchTarget {
              component_id: cmd.component_id,
              target: cmd.target,
              transition_ms: cmd.transition_ms,
              flags: cmd.flags,
              tx_id: [0u8; 16],
            });
          }
        })
        .await;

      if let Some(dispatch) = dispatch_to_core0 {
        let _ = tx_to_module.send(dispatch).await;
      }
    }
  }
}
