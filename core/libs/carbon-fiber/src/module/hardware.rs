//! # Hardware Vendor Specific Code
//!
//! CarbonFiber needs to interact with a variety of
//!

pub trait Vendor {}

pub struct HardwareIdentity {
  /// Hardware serial/UID (up to 16 bytes, padded with 0 if shorter)
  pub unique_id: [u8; 16],
  pub unique_id_len: usize,
  /// 32 bytes of raw hardware/drift entropy
  pub dynamic_entropy: [u8; 32],
}

impl HardwareIdentity {
  /// Automatically detects platform features and collects hardware serial + drift entropy.
  #[cfg(feature = "mcu")]
  pub fn collect() -> Self {
    #[cfg(feature = "mcu-rp2040")]
    return platform::rp2040::collect();

    #[cfg(feature = "mcu-rp235x")]
    return platform::rp235x::collect();

    #[cfg(feature = "mcu-nrf52")]
    return platform::nrf52::collect();

    #[cfg(feature = "mcu-stm32")]
    return platform::stm32::collect();

    #[cfg(feature = "simulator")]
    return platform::simulator::collect();

    #[cfg(not(any(
      feature = "mcu-rp2040",
      feature = "mcu-rp235x",
      feature = "mcu-nrf52",
      feature = "mcu-stm32",
      feature = "simulator"
    )))]
    compile_error!(
      "You must enable exactly one MCU platform feature (e.g. `mcu-rp2040`, `mcu-rp2350`)."
    );
  }
}

pub mod platform {
  #[cfg(feature = "mcu-rp235x")]
  pub mod rp235x {
    use super::super::HardwareIdentity;

    pub fn collect() -> HardwareIdentity {
      let mut id = [0u8; 16];

      // Read hardware ID (Row 0x0002..0x0005 in OTP memory)
      for i in 0..4 {
        if let Ok(word) = rp235x_hal::otp::read_ecc_word(2 + i) {
          id[i * 2..i * 2 + 2].copy_from_slice(&word.to_le_bytes());
        }
      }

      // 2. Pull from RP2350's hardware TRNG peripheral
      let mut entropy = [0u8; 32];
      let trng_data_reg = 0x400B_0000 as *const u32; // TRNG FIFO/Data register

      for chunk in entropy.chunks_exact_mut(4) {
        let val = unsafe { core::ptr::read_volatile(trng_data_reg) };
        chunk.copy_from_slice(&val.to_le_bytes());
      }

      HardwareIdentity {
        unique_id: id,
        unique_id_len: 8,
        dynamic_entropy: entropy,
      }
    }
  }

  #[cfg(feature = "mcu-rp2040")]
  pub mod rp2040 {
    use super::super::HardwareIdentity;

    pub fn collect() -> HardwareIdentity {
      let mut id = [0u8; 16];

      // Read 64-bit hardware ID from flash
      let mut flash_id = [0u8; 8];
      unsafe {
        cortex_m::interrupt::free(|_cs| {
          rp2040_flash::flash::flash_unique_id(&mut flash_id, true);
        });
      }
      id[..8].copy_from_slice(&flash_id);

      // ROSC drift calculation
      let mut entropy = [0u8; 32];
      let rosc_random_bit = 0x4006_001C as *const u32; // ROSC RANDOMBIT register

      for byte in entropy.iter_mut() {
        let mut accumulator = 0u8;
        for _ in 0..8 {
          // Read LSB of ring oscillator register
          let bit = unsafe { core::ptr::read_volatile(rosc_random_bit) } & 1;
          accumulator = (accumulator << 1) | (bit as u8);

          // Small busy-spin to allow phase jitter to accumulate
          cortex_m::asm::delay(32);
        }
        *byte = accumulator;
      }

      HardwareIdentity {
        unique_id: id,
        unique_id_len: 8,
        dynamic_entropy: entropy,
      }
    }
  }

  #[cfg(feature = "mcu-nrf52")]
  pub mod nrf52 {
    use super::super::HardwareIdentity;

    pub fn collect() -> HardwareIdentity {
      let mut id = [0u8; 16];

      // Read 64-bit hardware ID from FICR
      let ficr = unsafe { &*nrf52840_pac::FICR::ptr() };
      let dev_id0 = ficr.deviceid[0].read().bits();
      let dev_id1 = ficr.deviceid[1].read().bits();
      id[0..4].copy_from_slice(&dev_id0.to_le_bytes());
      id[4..8].copy_from_slice(&dev_id1.to_le_bytes());

      // Read from RNG peripheral
      let mut entropy = [0u8; 32];
      let rng = unsafe { &*nrf52840_pac::RNG::ptr() };
      rng.tasks_start.write(|w| unsafe { w.bits(1) });

      for byte in entropy.iter_mut() {
        while rng.events_valrdy.read().bits() == 0 {}
        rng.events_valrdy.write(|w| unsafe { w.bits(0) });
        *byte = rng.value.read().value().bits();
      }

      rng.tasks_stop.write(|w| unsafe { w.bits(1) });

      HardwareIdentity {
        unique_id: id,
        unique_id_len: 8,
        dynamic_entropy: entropy,
      }
    }
  }

  #[cfg(feature = "simulator")]
  pub mod simulator {
    use super::super::HardwareIdentity;

    pub fn collect() -> HardwareIdentity {
      HardwareIdentity {
        unique_id: [0u8; 16],
        unique_id_len: 16,
        dynamic_entropy: [0u8; 32],
      }
    }
  }
}
