pub mod clock_adapter;
pub mod mcp_adapter;
pub mod runtime_adapter;

use can_iso_tp::{
  IsoTpAsyncNode,
  IsoTpConfig,
};
use core::{
  cell::RefCell,
  time::Duration,
};
use defmt::*;
use embassy_embedded_hal::shared_bus::blocking::spi::SpiDevice;
use embassy_rp::{
  Peri,
  gpio::{
    AnyPin,
    Input,
    Level,
    Output,
  },
};
use mcp2515::{
  CanSpeed,
  MCP2515,
  McpSpeed,
  Settings,
  regs::OpMode,
};

use static_cell::ConstStaticCell;

use crate::{
  SpiBus,
  comms::{
    clock_adapter::EmbassyClock,
    mcp_adapter::split_mcp,
    runtime_adapter::EmbassyRuntime,
  },
};

#[embassy_executor::task]
pub async fn comms_thread(
  spi_bus: &'static SpiBus,
  cs_pin: Peri<'static, AnyPin>,
  int_pin: Input<'static>,
) {
  info!("Starting Comms Thread!");
  defmt::flush();

  let mcp_cs = Output::new(cs_pin, Level::High);
  let mcp_spi = SpiDevice::new(spi_bus, mcp_cs);
  let mut mcp = MCP2515::new(mcp_spi);

  let mut delay = embassy_time::Delay;
  let settings = Settings {
    mode: OpMode::Normal,
    can_speed: CanSpeed::Kbps125,
    mcp_speed: McpSpeed::MHz8,
    clkout_en: false,
  };

  match mcp.init(&mut delay, settings) {
    Ok(()) => {
      info!("MCP2515 initialized successfully in Normal mode (125 kbps, 8 MHz oscillator)!");
      defmt::flush();

      debug!("Creating MCP refcell...");
      defmt::flush();
      let mcp_driver = RefCell::new(mcp);
      debug!("MCP refcell created!");
      defmt::flush();

      debug!("Splitting MCP...");
      defmt::flush();
      let (tx, rx) = split_mcp(&mcp_driver, int_pin);
      debug!("Channels split!");
      defmt::flush();

      debug!("Making RX cell, and initing it...");
      defmt::flush();
      static RX_BUF: ConstStaticCell<[u8; 4095]> = ConstStaticCell::new([0u8; 4095]);
      let rx_buf = RX_BUF.take();
      debug!("RX cell inited!");
      defmt::flush();

      debug!("Creating Async ISO-TP node...");
      defmt::flush();
      let iso_tp_cfg = IsoTpConfig {
        tx_id: embedded_can_interface::Id::Standard(embedded_can::StandardId::new(0x7E0).unwrap()),
        rx_id: embedded_can_interface::Id::Standard(embedded_can::StandardId::new(0x7E8).unwrap()),
        tx_addr: 0x7E.into(),
        rx_addr: 0x7F.into(),

        ..Default::default()
      };

      match IsoTpAsyncNode::with_clock(tx, rx, iso_tp_cfg, EmbassyClock, rx_buf) {
        Ok(mut node) => {
          info!("ISO-TP node created successfully!");
          defmt::flush();

          let runtime = EmbassyRuntime {};

          let mut seq: u32 = 0;
          loop {
            info!("Attempting to send ISO-TP frame #{}...", seq);
            defmt::flush();

            match node
              .send(
                &runtime,
                // `AAAA`
                &[0x41, 0x41, 0x41, 0x41],
                Duration::from_millis(500),
              )
              .await
            {
              Ok(_) => {
                info!("Sent `AAAA` (seq {}) successfully!", seq);
              }
              Err(err) => {
                let tec: u8 = mcp_driver
                  .borrow_mut()
                  .read_register::<1, mcp2515::regs::Tec>()
                  .map(|t| t.into_bytes()[0])
                  .unwrap_or(255);
                let rec: u8 = mcp_driver
                  .borrow_mut()
                  .read_register::<1, mcp2515::regs::Rec>()
                  .map(|r| r.into_bytes()[0])
                  .unwrap_or(255);
                let intf: u8 = mcp_driver
                  .borrow_mut()
                  .read_register::<1, mcp2515::regs::CanIntf>()
                  .map(|i| i.into_bytes()[0])
                  .unwrap_or(255);
                let stat: u8 = mcp_driver
                  .borrow_mut()
                  .read_register::<1, mcp2515::regs::CanStat>()
                  .map(|s| s.into_bytes()[0])
                  .unwrap_or(255);
                let status_byte: u8 = mcp_driver
                  .borrow_mut()
                  .read_status()
                  .map(|s| s.into_bytes()[0])
                  .unwrap_or(255);
                error!(
                  "Failed to send ISO-TP frame (seq {}) due to: {:?} (TEC={}, REC={}, INTF=0x{:02x}, CANSTAT=0x{:02x}, STATUS=0x{:02x})",
                  seq,
                  Debug2Format(&err),
                  tec,
                  rec,
                  intf,
                  stat,
                  status_byte,
                );
              }
            }
            defmt::flush();
            seq = seq.wrapping_add(1);
            embassy_time::Timer::after_millis(1000).await;
          }
        }
        Err(err) => {
          error!(
            "Failed to initialize ISO-TP node due to: {:?}",
            Debug2Format(&err)
          );
          defmt::flush();
        }
      }
    }
    Err(err) => {
      error!(
        "Failed to initialize MCP2515 over SPI: {:?}",
        Debug2Format(&err)
      );
      defmt::flush();
    }
  }
}
