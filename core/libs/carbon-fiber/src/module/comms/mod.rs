

pub mod can2;

use core::future::Future;
use core::pin::Pin;
use core::task::{
  Context,
  Poll,
};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{
  Channel,
  Receiver,
  Sender,
};
use embedded_can::{
  Frame,
  StandardId,
};
use serde::{
  Deserialize,
  Serialize,
};

use crate::alerts::models::components::{
  AlertMessage,
  AlertSeverity,
};
use crate::commands::models::components::{
  ComponentTarget,
  ComponentTelemetry,
  GestureFlags,
  GestureTargetCommand,
};
use crate::module::comms::can2::{
  Can2Frame,
  IsoTpReassembler,
  CAN_ID_ALERT_INFO_BASE,
  CAN_ID_CRITICAL_BASE,
  CAN_ID_SELF_TEST,
  CAN_ID_TELEMETRY,
};

pub trait CommsBus {
  type Frame: Frame;
  type Error: core::fmt::Debug;

  fn send_frame(&mut self, frame: &Self::Frame) -> Result<(), Self::Error>;
  fn receive_frame(&mut self) -> Result<Option<Self::Frame>, Self::Error>;
}

impl CommsBus for () {
  type Frame = Can2Frame;
  type Error = core::convert::Infallible;

  fn send_frame(&mut self, _frame: &Self::Frame) -> Result<(), Self::Error> {
    Ok(())
  }

  fn receive_frame(&mut self) -> Result<Option<Self::Frame>, Self::Error> {
    Ok(None)
  }
}

impl<B: CommsBus> CommsBus for Option<B> {
  type Frame = B::Frame;
  type Error = B::Error;

  fn send_frame(&mut self, frame: &Self::Frame) -> Result<(), Self::Error> {
    if let Some(bus) = self {
      bus.send_frame(frame)
    } else {
      Ok(())
    }
  }

  fn receive_frame(&mut self) -> Result<Option<Self::Frame>, Self::Error> {
    if let Some(bus) = self {
      bus.receive_frame()
    } else {
      Ok(None)
    }
  }
}

pub struct YieldFuture(pub bool);

impl Future for YieldFuture {
  type Output = ();

  fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
    if self.0 {
      Poll::Ready(())
    } else {
      self.0 = true;
      cx.waker().wake_by_ref();
      Poll::Pending
    }
  }
}

/// Core 1: Owns ONLY Comms/CAN and IPC queues.
/// Has zero access to physical components.
/// Core 1: Owns ONLY Comms/CAN and IPC queues.
/// Has zero access to physical components.
pub struct CommsWorker<'a, Bus> {
  pub bus: Bus,
  pub adoption: crate::adoption::module::ModuleAdoptionContext<'a>,
  pub rx_from_module: Receiver<'static, CriticalSectionRawMutex, ModuleToComms, 16>,
  pub tx_to_module: Sender<'static, CriticalSectionRawMutex, CommsToModule, 8>,
  pub reassembler: IsoTpReassembler<256>,
}

impl<'a, Bus> CommsWorker<'a, Bus> {
  pub const fn new(
    bus: Bus,
    adoption: crate::adoption::module::ModuleAdoptionContext<'a>,
    rx_from_module: Receiver<'static, CriticalSectionRawMutex, ModuleToComms, 16>,
    tx_to_module: Sender<'static, CriticalSectionRawMutex, CommsToModule, 8>,
  ) -> Self {
    Self {
      bus,
      adoption,
      rx_from_module,
      tx_to_module,
      reassembler: IsoTpReassembler::new(),
    }
  }
}

impl<'a, Bus: CommsBus> CommsWorker<'a, Bus> {
  pub fn transmit_message(&mut self, msg: ModuleToComms) -> Result<(), Bus::Error> {
    let (can_id, payload_buf, payload_len) = match msg {
      ModuleToComms::Alert(ref alert) => {
        let is_critical = matches!(
          alert.severity,
          AlertSeverity::Critical | AlertSeverity::Fatal
        );
        let id = if is_critical {
          CAN_ID_CRITICAL_BASE | ((alert.component_id as u16) & 0x0F)
        } else {
          CAN_ID_ALERT_INFO_BASE | ((alert.component_id as u16) & 0x7F)
        };
        let mut buf = [0u8; 8];
        let len = match postcard::to_slice(alert, &mut buf) {
          Ok(slice) => slice.len(),
          Err(_) => 0,
        };
        (id, buf, len)
      }
      ModuleToComms::SelfTestResult { .. } => {
        let mut buf = [0u8; 8];
        let len = match postcard::to_slice(&msg, &mut buf) {
          Ok(slice) => slice.len(),
          Err(_) => 0,
        };
        (CAN_ID_SELF_TEST, buf, len)
      }
      ModuleToComms::Telemetry { .. } => {
        let mut buf = [0u8; 8];
        let len = match postcard::to_slice(&msg, &mut buf) {
          Ok(slice) => slice.len(),
          Err(_) => 0,
        };
        (CAN_ID_TELEMETRY, buf, len)
      }
    };

    if payload_len > 0 {
      let std_id = StandardId::new(can_id).unwrap_or(StandardId::ZERO);
      if let Some(frame) = Bus::Frame::new(std_id, &payload_buf[..payload_len]) {
        self.bus.send_frame(&frame)?;
      }
    }
    Ok(())
  }

  fn send_adoption_reply(bus: &mut Bus, reply: crate::models::BusMessage) {
    let mut buf = [0u8; 128];
    if let Ok(bytes) = postcard::to_slice(&reply, &mut buf) {
      if bytes.len() <= 7 {
        let mut sf = [0u8; 8];
        sf[0] = bytes.len() as u8;
        sf[1..1 + bytes.len()].copy_from_slice(bytes);
        if let Some(f) = Bus::Frame::new(StandardId::new(0x7E8).unwrap_or(StandardId::ZERO), &sf[..1 + bytes.len()]) {
          let _ = bus.send_frame(&f);
        }
      } else {
        let total = bytes.len();
        let mut ff = [0u8; 8];
        ff[0] = 0x10 | ((total >> 8) as u8 & 0x0F);
        ff[1] = (total & 0xFF) as u8;
        ff[2..8].copy_from_slice(&bytes[..6]);
        if let Some(f) = Bus::Frame::new(StandardId::new(0x7E8).unwrap_or(StandardId::ZERO), &ff) {
          let _ = bus.send_frame(&f);
        }
        let mut sent = 6;
        let mut seq = 1u8;
        while sent < total {
          let chunk = core::cmp::min(7, total - sent);
          let mut cf = [0u8; 8];
          cf[0] = 0x20 | (seq & 0x0F);
          cf[1..1 + chunk].copy_from_slice(&bytes[sent..sent + chunk]);
          if let Some(f) = Bus::Frame::new(StandardId::new(0x7E8).unwrap_or(StandardId::ZERO), &cf[..1 + chunk]) {
            let _ = bus.send_frame(&f);
          }
          sent += chunk;
          seq = (seq + 1) & 0x0F;
        }
      }
    }
  }

  pub async fn ingest_isotp_frame(&mut self, frame: &Bus::Frame) {
    if let Some(payload) = self.reassembler.ingest(frame.data()) {
      if let Ok(bus_msg) = postcard::from_bytes::<crate::models::BusMessage>(payload) {
        match bus_msg {
          crate::models::BusMessage::Content(raw) => {
            if let Ok(cmd) = postcard::from_bytes::<GestureTargetCommand>(raw.data) {
              let _ = self
                .tx_to_module
                .send(CommsToModule::DispatchTarget {
                  component_id: cmd.component_id,
                  target: cmd.target,
                  transition_ms: cmd.transition_ms,
                  flags: cmd.flags,
                  tx_id: [0u8; 16],
                })
                .await;
            } else if let Ok(dispatch) = postcard::from_bytes::<CommsToModule>(raw.data) {
              let _ = self.tx_to_module.send(dispatch).await;
            }
          }
          crate::models::BusMessage::Adoption(adopt) => {
            match adopt {
              crate::adoption::models::AdoptionMessage::Probe(probe) => {
                if self.adoption.parse_probe(&probe).is_ok() {
                  let hello = self.adoption.gen_hello();
                  let reply = crate::models::BusMessage::Adoption(
                    crate::adoption::models::AdoptionMessage::Hello(hello),
                  );
                  Self::send_adoption_reply(&mut self.bus, reply);
                }
              }
              crate::adoption::models::AdoptionMessage::KeyHOMI(homi) => {
                if self.adoption.parse_key_homi(&homi).is_ok() {
                  let himo = self.adoption.gen_key_himo();
                  let reply = crate::models::BusMessage::Adoption(
                    crate::adoption::models::AdoptionMessage::KeyHIMO(himo),
                  );
                  Self::send_adoption_reply(&mut self.bus, reply);
                }
              }
              crate::adoption::models::AdoptionMessage::Confirm(confirm) => {
                let _ = self.adoption.parse_confirm(&confirm);
              }
              crate::adoption::models::AdoptionMessage::Abortion(abort) => {
                self.adoption.state = crate::adoption::models::AdoptionState::Aborted(abort.reason);
              }
              _ => {}
            }
          }
        }
      } else if let Ok(cmd) = postcard::from_bytes::<GestureTargetCommand>(payload) {
        let _ = self
          .tx_to_module
          .send(CommsToModule::DispatchTarget {
            component_id: cmd.component_id,
            target: cmd.target,
            transition_ms: cmd.transition_ms,
            flags: cmd.flags,
            tx_id: [0u8; 16],
          })
          .await;
      } else if let Ok(dispatch) = postcard::from_bytes::<CommsToModule>(payload) {
        let _ = self.tx_to_module.send(dispatch).await;
      }
    }
  }

  pub async fn wait_for_adoption(&mut self) {
    while self.adoption.state != crate::adoption::models::AdoptionState::Confirmed {
      if let Ok(Some(frame)) = self.bus.receive_frame() {
        self.ingest_isotp_frame(&frame).await;
      }
      YieldFuture(false).await;
    }
  }

  pub async fn run(&mut self) {
    // 1. Wait for adoption before entering operational loop
    self.wait_for_adoption().await;

    // 2. Operational loop: drain module messages and service dispatched targets
    loop {
      // Drain messages from module and transmit with priority
      while let Ok(msg) = self.rx_from_module.try_receive() {
        let _ = self.transmit_message(msg);
      }

      // Poll bus for incoming CAN frames and ingest ISO-TP
      if let Ok(Some(frame)) = self.bus.receive_frame() {
        self.ingest_isotp_frame(&frame).await;
      }

      YieldFuture(false).await;
    }
  }
}

#[cfg(feature = "embassy-time")]
impl<'a, D: can2::CanDriver> CommsWorker<'a, can2::CAN2<D>> {
  pub async fn run(&mut self) -> ! {
    self.bus.run(&mut self.adoption, self.rx_from_module, self.tx_to_module).await
  }
}



/// All message types from the Comms thread to the Module thread.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CommsToModule {
  DispatchTarget {
    component_id: u8,
    target: ComponentTarget,
    transition_ms: u16,
    flags: Option<GestureFlags>,
    tx_id: [u8; 16],
  },
  RunSelfTest,
  EmergencyStop,
}

/// All message types from the Module thread to the Comms thread.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ModuleToComms {
  SelfTestResult {
    success: bool,
    failed_component_id: Option<u8>,
    error_code: u8,
  },
  Telemetry {
    tx_id: [u8; 16],
    feedback: ComponentTelemetry,
  },
  Alert(AlertMessage),
}

pub type CommsQueue<const N: usize> = Channel<CriticalSectionRawMutex, CommsToModule, N>;
pub type ModuleQueue<const N: usize> = Channel<CriticalSectionRawMutex, ModuleToComms, N>;

pub struct ModuleChannels<const MOD_SZ: usize = 8, const COM_SZ: usize = 16> {
  pub to_module: CommsQueue<MOD_SZ>,
  pub to_comms: ModuleQueue<COM_SZ>,
}

impl<const MOD_SZ: usize, const COM_SZ: usize> ModuleChannels<MOD_SZ, COM_SZ> {
  pub const fn new() -> Self {
    Self {
      to_module: Channel::new(),
      to_comms: Channel::new(),
    }
  }
}
