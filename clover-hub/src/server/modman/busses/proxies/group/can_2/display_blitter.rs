//! # CAN 2.0 Display Blitter
//!
//! Subscribes to compressed QOI frame streams published by Bevy's offscreen renderer over Zenoh,
//! decompresses them into 16-bit RGB565 buffers, computes dirty bounding rectangles between
//! consecutive frames, and packages changed pixel data into 32-byte CAN 2.0 [`ComponentTarget::DisplayBlitChunk`]
//! frames transmitted across ISO-TP to adopted hardware modules.
//!
//! ## Zenoh Dependencies
//! - `com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame`:
//!   - **Upstream Provider**: [`crate::server::renderer::system_ui::systems::displays::display_registrar`]
//!   - **Rationale**: Consumes QOI display frames rendered by SystemUI to stream onto physical CAN hardware displays.

use std::{
  sync::Arc,
  time::Duration,
};

use carbon_fiber::commands::models::components::ComponentTarget;
use tokio_util::sync::CancellationToken;
use tracing::{
  debug,
  error,
  info,
  instrument,
  warn,
};

use crate::server::modman::{
  components::video::displays::models::{
    ConnectionType,
    DisplayFrameEnvelope,
  },
  gestures::command_generator::dispatch_target,
  models::{
    components::CloverComponent,
    store::ModManStore,
  },
};

/// Converts 8-bit RGBA channels into 16-bit RGB565 format (5-6-5 bits).
#[inline]
pub fn rgba8_to_rgb565(r: u8, g: u8, b: u8) -> u16 {
  let r5 = ((r as u16) >> 3) & 0x1F;
  let g6 = ((g as u16) >> 2) & 0x3F;
  let b5 = ((b as u16) >> 3) & 0x1F;
  (r5 << 11) | (g6 << 5) | b5
}

/// Converts a raw RGBA buffer into a Vec of RGB565 pixels.
pub fn rgba_buffer_to_rgb565(rgba: &[u8]) -> Vec<u16> {
  rgba
    .chunks_exact(4)
    .map(|chunk| rgba8_to_rgb565(chunk[0], chunk[1], chunk[2]))
    .collect()
}

/// Minimal bounding rectangle enclosing modified pixels between two frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirtyRect {
  pub min_x: u16,
  pub min_y: u16,
  pub max_x: u16,
  pub max_y: u16,
}

impl DirtyRect {
  #[inline]
  pub fn width(&self) -> u16 {
    self.max_x - self.min_x + 1
  }

  #[inline]
  pub fn height(&self) -> u16 {
    self.max_y - self.min_y + 1
  }
}

/// Computes the minimal bounding box enclosing all changed pixels between `curr` and `prev`.
///
/// If `prev` is empty or lengths differ, returns the full display rectangle.
/// If `curr` is completely identical to `prev`, returns `None`.
pub fn compute_dirty_rect(
  curr: &[u16],
  prev: &[u16],
  width: usize,
  height: usize,
) -> Option<DirtyRect> {
  if width == 0 || height == 0 || curr.is_empty() {
    return None;
  }

  if prev.is_empty() || prev.len() != curr.len() {
    return Some(DirtyRect {
      min_x: 0,
      min_y: 0,
      max_x: (width - 1) as u16,
      max_y: (height - 1) as u16,
    });
  }

  let mut min_x = u16::MAX;
  let mut min_y = u16::MAX;
  let mut max_x = 0;
  let mut max_y = 0;
  let mut changed = false;

  for y in 0..height {
    let row_offset = y * width;
    for x in 0..width {
      let idx = row_offset + x;
      if curr[idx] != prev[idx] {
        changed = true;
        let xu = x as u16;
        let yu = y as u16;
        if xu < min_x {
          min_x = xu;
        }
        if xu > max_x {
          max_x = xu;
        }
        if yu < min_y {
          min_y = yu;
        }
        if yu > max_y {
          max_y = yu;
        }
      }
    }
  }

  if changed {
    Some(DirtyRect {
      min_x,
      min_y,
      max_x,
      max_y,
    })
  } else {
    None
  }
}

/// Slices the dirty rectangle of an RGB565 frame into 32-byte (up to 16 pixels) DisplayBlitChunk commands.
pub fn generate_blit_chunks(
  curr: &[u16],
  width: usize,
  _height: usize,
  rect: DirtyRect,
) -> Vec<ComponentTarget> {
  let mut chunks = Vec::new();
  let mut chunk_offset: u16 = 0;

  for y in rect.min_y..=rect.max_y {
    let row_start = y as usize * width;
    let mut x = rect.min_x;
    while x <= rect.max_x {
      let remaining = (rect.max_x - x + 1) as usize;
      let count = remaining.min(16);
      let mut data = [0u8; 32];
      for i in 0..count {
        let pixel = curr[row_start + (x as usize) + i];
        let bytes = pixel.to_le_bytes();
        data[i * 2] = bytes[0];
        data[i * 2 + 1] = bytes[1];
      }

      chunks.push(ComponentTarget::DisplayBlitChunk {
        offset: chunk_offset,
        x,
        y,
        width: count as u16,
        height: 1,
        len: (count * 2) as u8,
        data,
      });

      chunk_offset = chunk_offset.wrapping_add(1);
      x += count as u16;
    }
  }

  chunks
}

/// CAN display blitter task for an adopted module.
///
/// Subscribes to Zenoh display frame envelopes for each ModManProxy display component
/// hosted on this module, decompresses QOI, computes dirty bounding boxes, and streams
/// `DisplayBlitChunk` commands via `dispatch_target`.
///
/// ### Zenoh Dependencies
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame`
/// - **Upstream Provider**: [`crate::server::renderer::system_ui::systems::displays::display_registrar`]
/// - **Expected Schema**: Serialized JSON string of [`DisplayFrameEnvelope`]
/// - **Rationale**: Ingests high-frequency rendered frames from SystemUI to transcode into CAN differential updates.
#[instrument(skip(session, cancellation_token, store))]
pub async fn can_display_blitter(

  session: Arc<zenoh::Session>,
  cancellation_token: CancellationToken,
  module_id: String,
  store: Arc<ModManStore>,
) {
  debug!("Initializing CAN display blitter for module: {module_id}...");

  // Wait for module and component registration in store
  let mut display_components: Vec<(u8, String)> = Vec::new();
  for _ in 0..30 {
    if cancellation_token.is_cancelled() {
      return;
    }

    {
      let modules = store.modules.lock().await;
      if let Some(m) = modules.get(&module_id) {
        let components_lock = store.components.lock().await;
        for (idx, (comp_id, _)) in m.components.iter().enumerate() {
          if let Some(comp_entry) = components_lock.get(comp_id) {
            if let CloverComponent::PhysicalDisplayComponent(ref phys) = comp_entry.1 {
              if matches!(phys.connection, ConnectionType::ModManProxy) {
                display_components.push((idx as u8, comp_id.clone()));
              }
            }
          }
        }
      }
    }

    if !display_components.is_empty() {
      break;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;
  }

  if display_components.is_empty() {
    debug!("Module {module_id} has no proxied displays; exiting display blitter.");
    return;
  }

  info!(
    "Starting CAN display blitter for module {module_id} with {} display(s): {:?}",
    display_components.len(),
    display_components
  );

  // Spawn a subscriber worker for each proxied display on this module
  for (comp_idx, comp_id) in display_components {
    let session_clone = session.clone();
    let token_clone = cancellation_token.clone();
    let mod_id = module_id.clone();
    let display_id = comp_id.clone();

    tokio::task::spawn(async move {
      let topic = format!(
        "com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame"
      );

      match session_clone.declare_subscriber(&topic).await {
        Ok(subscriber) => {
          info!("Subscribed to display frames on topic: {topic}");
          let mut prev_frame: Vec<u16> = Vec::new();

          while !token_clone.is_cancelled() {
            tokio::select! {
              _ = token_clone.cancelled() => break,
              sample_res = subscriber.recv_async() => {
                match sample_res {
                  Ok(sample) => {
                    if let Ok(payload_str) = sample.payload().try_to_string() {
                      if let Ok(envelope) = serde_json::from_str::<DisplayFrameEnvelope>(&payload_str) {
                        let w = envelope.width as usize;
                        let h = envelope.height as usize;

                        match qoi::decode_to_vec(&envelope.qoi_data) {
                          Ok((_header, raw_rgba)) => {
                            let curr_frame = rgba_buffer_to_rgb565(&raw_rgba);
                            if let Some(dirty_rect) = compute_dirty_rect(&curr_frame, &prev_frame, w, h) {
                              let chunks = generate_blit_chunks(&curr_frame, w, h, dirty_rect);
                              debug!(
                                "Blitting {} chunks for display {display_id} (dirty rect: {:?})",
                                chunks.len(),
                                dirty_rect
                              );

                              for chunk in chunks {
                                if token_clone.is_cancelled() {
                                  break;
                                }
                                if let Err(e) = dispatch_target(
                                  &session_clone,
                                  &mod_id,
                                  comp_idx,
                                  chunk,
                                  0,
                                ).await {
                                  warn!("Failed to dispatch display blit chunk to {mod_id}/{display_id}: {e}");
                                }
                              }

                              prev_frame = curr_frame;
                            } else {
                              debug!("Display {display_id} frame seq {} unchanged, skipping bus blit.", envelope.seq);
                            }
                          }
                          Err(e) => {
                            error!("Failed to decode QOI frame for display {display_id}: {e}");
                          }
                        }
                      }
                    }
                  }
                  Err(e) => {
                    warn!("Subscriber error on {topic}: {e}");
                    break;
                  }
                }
              }
            }
          }

          debug!("Shutting down display frame subscriber for {display_id}.");
        }
        Err(e) => {
          error!("Failed to subscribe to display frames on {topic}: {e}");
        }
      }
    });
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_rgba8_to_rgb565_color_conversions() {
    // Pure Red (255, 0, 0) -> R5=31, G6=0, B5=0 -> 0xF800
    assert_eq!(rgba8_to_rgb565(255, 0, 0), 0xF800);
    // Pure Green (0, 255, 0) -> R5=0, G6=63, B5=0 -> 0x07E0
    assert_eq!(rgba8_to_rgb565(0, 255, 0), 0x07E0);
    // Pure Blue (0, 0, 255) -> R5=0, G6=0, B5=31 -> 0x001F
    assert_eq!(rgba8_to_rgb565(0, 0, 255), 0x001F);
    // Black (0, 0, 0) -> 0x0000
    assert_eq!(rgba8_to_rgb565(0, 0, 0), 0x0000);
    // White (255, 255, 255) -> 0xFFFF
    assert_eq!(rgba8_to_rgb565(255, 255, 255), 0xFFFF);
  }

  #[test]
  fn test_rgba_buffer_to_rgb565() {
    let rgba = vec![
      255, 0, 0, 255, // Red
      0, 255, 0, 255, // Green
      0, 0, 255, 255, // Blue
      255, 255, 255, 255, // White
    ];
    let rgb565 = rgba_buffer_to_rgb565(&rgba);
    assert_eq!(rgb565, vec![0xF800, 0x07E0, 0x001F, 0xFFFF]);
  }

  #[test]
  fn test_compute_dirty_rect_first_frame_and_unchanged() {
    let curr = vec![0x1234; 64 * 32];
    // First frame (empty prev) -> full screen dirty
    let rect = compute_dirty_rect(&curr, &[], 64, 32).unwrap();
    assert_eq!(
      rect,
      DirtyRect {
        min_x: 0,
        min_y: 0,
        max_x: 63,
        max_y: 31
      }
    );

    // Identical frame -> None (0 bandwidth)
    assert_eq!(compute_dirty_rect(&curr, &curr, 64, 32), None);
  }

  #[test]
  fn test_compute_dirty_rect_sub_region() {
    let w = 16;
    let h = 16;
    let prev = vec![0u16; w * h];
    let mut curr = prev.clone();

    // Modify a 4x4 region from (2, 3) to (5, 6)
    for y in 3..=6 {
      for x in 2..=5 {
        curr[y * w + x] = 0xFFFF;
      }
    }

    let rect = compute_dirty_rect(&curr, &prev, w, h).unwrap();
    assert_eq!(
      rect,
      DirtyRect {
        min_x: 2,
        min_y: 3,
        max_x: 5,
        max_y: 6
      }
    );
  }

  #[test]
  fn test_generate_blit_chunks_slicing() {
    let w = 32;
    let h = 16;
    let curr = vec![0xABCDu16; w * h];

    let rect = DirtyRect {
      min_x: 0,
      min_y: 0,
      max_x: 31,
      max_y: 0,
    };

    let chunks = generate_blit_chunks(&curr, w, h, rect);
    // 32 pixels in row 0 -> 2 chunks of 16 pixels
    assert_eq!(chunks.len(), 2);

    match &chunks[0] {
      ComponentTarget::DisplayBlitChunk {
        offset,
        x,
        y,
        width,
        height,
        len,
        data,
      } => {
        assert_eq!(*offset, 0);
        assert_eq!(*x, 0);
        assert_eq!(*y, 0);
        assert_eq!(*width, 16);
        assert_eq!(*height, 1);
        assert_eq!(*len, 32);
        assert_eq!(&data[0..2], &0xABCDu16.to_le_bytes());
      }
      _ => panic!("Expected DisplayBlitChunk"),
    }

    match &chunks[1] {
      ComponentTarget::DisplayBlitChunk {
        offset,
        x,
        y,
        width,
        height,
        len,
        data,
      } => {
        assert_eq!(*offset, 1);
        assert_eq!(*x, 16);
        assert_eq!(*y, 0);
        assert_eq!(*width, 16);
        assert_eq!(*height, 1);
        assert_eq!(*len, 32);
        assert_eq!(&data[0..2], &0xABCDu16.to_le_bytes());
      }
      _ => panic!("Expected DisplayBlitChunk"),
    }
  }

  #[test]
  fn test_qoi_envelope_roundtrip() {
    let w = 8u16;
    let h = 8u16;
    let mut raw_rgba = Vec::with_capacity((w * h * 4) as usize);
    for _ in 0..(w * h) {
      raw_rgba.extend_from_slice(&[255, 128, 64, 255]);
    }

    let qoi_bytes = qoi::encode_to_vec(&raw_rgba, w as u32, h as u32).unwrap();
    let envelope = DisplayFrameEnvelope {
      seq: 42,
      timestamp_ms: 123456789,
      width: w,
      height: h,
      qoi_data: qoi_bytes,
    };

    let json = serde_json::to_string(&envelope).unwrap();
    let deserialized: DisplayFrameEnvelope = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.seq, 42);
    assert_eq!(deserialized.width, w);
    assert_eq!(deserialized.height, h);

    let (_header, decoded_rgba) = qoi::decode_to_vec(&deserialized.qoi_data).unwrap();
    assert_eq!(decoded_rgba, raw_rgba);
  }
}
