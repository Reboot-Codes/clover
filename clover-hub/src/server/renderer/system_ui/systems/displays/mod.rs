//! # System UI Displays Subsystem
//!
//! Manages Bevy offscreen Render-to-Texture (RTT) cameras, display readbacks, and streaming
//! QOI-compressed frame envelopes over Zenoh to downstream hardware blitters.
//!
//! ## Zenoh Endpoints
//!
//! ### Publishers
//! - `com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame`:
//!   Streams [`DisplayFrameEnvelope`] frames for proxied displays at their configured FPS.
//!   Implemented by [`display_registrar`].

use crate::server::modman::components::video::displays::models::{
  DisplayFrameEnvelope,
  VirtualDisplayComponent,
};

use crate::server::renderer::system_ui::{
  AnyDisplayComponent,
  SystemUIIPC,
};
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::gpu_readback::{Readback, ReadbackComplete};
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
#[cfg(feature = "compositor")]
use bevy::window::{
  Monitor,
  WindowMode,
  WindowRef,
};
use log::{debug, info, warn};
use queues::*;

/// Identifies a desktop window associated with a specific physical or virtual display.
#[derive(Component)]
#[cfg(feature = "compositor")]
pub struct DisplayWindow {
  /// Display component identifier.
  pub id: String,
}

/// Identifies the camera assigned to render an offscreen or windowed display target.
#[derive(Component)]
pub struct DisplayCamera {
  /// Display component identifier.
  pub id: String,
}

/// Marker component for composited display entities.
#[derive(Component)]
pub struct DisplayComposition {
  /// Display component identifier.
  pub id: String,
}

/// Marker component storing the identifier of a registered virtual display.
#[derive(Component)]
pub struct VirtualDisplayID {
  /// Virtual display component identifier.
  pub id: String,
}

/// Transient marker component referencing the parent display target for a one-off readback request.
#[derive(Component)]
pub struct DisplayReadbackTag {
  pub target: Entity,
}

/// Tracking component for an offscreen display rendered to texture and periodically extracted over Zenoh.
#[derive(Component)]
pub struct ProxiedDisplayTarget {
  /// Unique display component identifier on Zenoh.
  pub id: String,
  /// Asset handle for the offscreen render target image.
  pub image_handle: Handle<Image>,
  /// Native pixel width.
  pub width: u16,
  /// Native pixel height.
  pub height: u16,
  /// Configured framerate floor (default >= 24 FPS).
  pub target_fps: u32,
  /// Frame pacing timer governing readback request intervals.
  pub timer: Timer,
  /// Monotonically increasing frame sequence counter.
  pub seq: u64,
  /// Whether an asynchronous GPU readback is currently in flight.
  pub pending_readback: bool,
  /// Whether desktop mirror window spawning is enabled for this target.
  pub mirror: bool,
  /// Optional bounded channel sender delivering extracted frame envelopes to the dedicated Tokio Zenoh publisher.
  pub frame_sender: Option<tokio::sync::mpsc::Sender<DisplayFrameEnvelope>>,
  /// Reusable pre-allocated scratch buffer for unpadded RGBA pixels.
  pub unpadded_buffer: Vec<u8>,
  /// Reusable pre-allocated buffer for QOI encoded frame bytes.
  pub qoi_buffer: Vec<u8>,
}

/// Ingests display registration requests from [`SystemUIIPC::display_registration_queue`]
/// and initializes cameras, render targets, textures, and desktop preview windows.
///
/// ### Zenoh Publishers
/// - **Key Expression**: `com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame`
/// - **Payload Format**: Serialized JSON string of [`DisplayFrameEnvelope`]
/// - **Trigger**: Dispatched at target display framerate (e.g. 24 or 30 FPS) after GPU texture readback.
///
/// #### Payload Example
/// ```json
/// {
///   "seq": 1042,
///   "timestamp_ms": 1730620000000,
///   "width": 128,
///   "height": 64,
///   "qoi_data": [113, 111, 105, 102, 0, 0, 0, 128, 0, 0, 0, 64, 4, 0]
/// }
/// ```
pub fn display_registrar(

  mut commands: Commands,
  ipc: Res<SystemUIIPC>,
  mut images: ResMut<Assets<Image>>,
  #[cfg(feature = "compositor")] monitor_entities_query: Query<(Entity, &Monitor)>,
  vdisplay_query: Query<(Entity, &mut VirtualDisplayComponent, &VirtualDisplayID)>,
  mut next_layer: Local<usize>,
) {
  let ipc_display_reg_queue_clone = ipc.display_registration_queue.clone();
  let mut display_queue = ipc_display_reg_queue_clone.lock().unwrap();

  match display_queue.remove() {
    Ok(display_config) => {
      let display_id = display_config.0;
      let any_display = display_config.1;

      debug!("Found display to register: {}", display_id.clone());

      match any_display {
        AnyDisplayComponent::Physical(physical_display_component) => {
          match physical_display_component.connection {
            #[cfg(feature = "compositor")]
            crate::server::modman::components::video::displays::models::ConnectionType::Direct(
              direct_connection,
            ) => {
              let monitor = if direct_connection.display_id.clone() == "@primary" {
                MonitorSelection::Primary
              } else {
                let mut chosen_monitor = None;
                for (monitor_eid, monitor) in &monitor_entities_query {
                  if monitor.name == Some(direct_connection.display_id.clone()) {
                    chosen_monitor = Some(monitor_eid);
                    break;
                  }
                }

                match chosen_monitor {
                  Some(monitor) => MonitorSelection::Entity(monitor),
                  None => {
                    // TODO: Is this really the best behaviour?
                    warn!("Requested display not found, defaulting to primary display...");
                    MonitorSelection::Primary
                  }
                }
              };

              let windowed = if direct_connection.windowed {
                WindowMode::Windowed
              } else {
                WindowMode::BorderlessFullscreen(monitor.clone())
              };
              let position = if direct_connection.windowed {
                WindowPosition::Centered(monitor.clone())
              } else {
                WindowPosition::Automatic
              };

              match physical_display_component.virtual_display {
                Some(vdisplay_id) => {
                  for (
                    queried_vdisplay_entity,
                    _queried_vdisplay_component,
                    queried_vdisplay_id,
                  ) in &vdisplay_query
                  {
                    if vdisplay_id == queried_vdisplay_id.id {
                      use bevy::window::WindowTheme;

                      let window = commands
                        .get_entity(queried_vdisplay_entity)
                        .unwrap()
                        .insert((
                          Window {
                            title: format!(
                              "Clover SystemUI: Virtual Display: {}",
                              display_id.clone()
                            ),
                            mode: windowed,
                            window_theme: Some(WindowTheme::Dark),
                            position,
                            ..Default::default()
                          },
                          DisplayWindow {
                            id: vdisplay_id.to_string(),
                          },
                        ))
                        .id();

                      commands.spawn((
                        Camera3d::default(),
                        Camera {
                          target: RenderTarget::Window(WindowRef::Entity(window)),
                          ..default()
                        },
                        Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                        DisplayCamera {
                          id: vdisplay_id.to_string(),
                        },
                      ));

                      break;
                    }
                  }
                }
                None => {
                  // TODO: Spawn a composition for this display.

                  use bevy::window::WindowTheme;

                  let window = commands
                    .spawn((
                      Window {
                        title: format!("Clover SystemUI: Display: {}", display_id.clone()),
                        mode: windowed,
                        window_theme: Some(WindowTheme::Dark),
                        position,
                        ..Default::default()
                      },
                      DisplayWindow {
                        id: display_id.clone(),
                      },
                    ))
                    .id();

                  commands.spawn((
                    Camera3d::default(),
                    Camera {
                      target: RenderTarget::Window(WindowRef::Entity(window)),
                      ..default()
                    },
                    Transform::from_xyz(6.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
                    DisplayCamera {
                      id: display_id.clone(),
                    },
                  ));
                }
              }
            }
            crate::server::modman::components::video::displays::models::ConnectionType::ModManProxy
            | crate::server::modman::components::video::displays::models::ConnectionType::Simulated => {
              let width = physical_display_component.resolution.width.get();
              let height = physical_display_component.resolution.height.get();

              let size = Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
              };
              let mut image = Image::new_fill(
                size,
                TextureDimension::D2,
                &[0, 0, 0, 255],
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::default(),
              );
              image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::COPY_SRC;
              image.sampler = ImageSampler::nearest();
              let image_handle = images.add(image);

              *next_layer += 1;
              let layer_idx = *next_layer;
              let layer = RenderLayers::layer(layer_idx);

              // Spawn offscreen Camera3d rendering to this texture
              commands.spawn((
                Camera3d::default(),
                Camera {
                  clear_color: Color::srgb(0.0, 0.0, 0.0).into(),
                  target: RenderTarget::Image(image_handle.clone().into()),
                  ..default()
                },
                Transform::from_xyz(0.0, 0.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                layer.clone(),
                DisplayCamera {
                  id: display_id.clone(),
                },
              ));

              let target_fps = physical_display_component.target_fps.unwrap_or(24).max(24);
              let is_simulated = matches!(
                physical_display_component.connection,
                crate::server::modman::components::video::displays::models::ConnectionType::Simulated
              ) || display_id == "debug_internal_display"
                || display_id == "debug_external_display";
              let is_mirrored = physical_display_component.mirror.unwrap_or(false);

              let (frame_tx, mut frame_rx) =
                tokio::sync::mpsc::channel::<DisplayFrameEnvelope>(2);
              let session = ipc.zenoh_session.clone();
              let pub_topic = format!(
                "com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame"
              );

              // Dedicated, persistent async publisher worker with 1 declared publisher for display frames
              ipc.tokio_handle.spawn(async move {
                let publisher = match session.declare_publisher(&pub_topic).await {
                  Ok(p) => p,
                  Err(e) => {
                    log::error!("Failed to declare Zenoh publisher for {pub_topic}: {e}");
                    return;
                  }
                };

                let mut json_buf = Vec::with_capacity(65536);
                let mut frame_count = 0u64;
                while let Some(envelope) = frame_rx.recv().await {
                  frame_count = frame_count.wrapping_add(1);
                  if frame_count % 150 == 0 {
                    #[cfg(target_os = "linux")]
                    unsafe {
                      extern "C" {
                        fn malloc_trim(pad: usize) -> i32;
                      }
                      malloc_trim(0);
                    }
                  }
                  json_buf.clear();
                  match serde_json::to_writer(&mut json_buf, &envelope) {
                    Ok(()) => {
                      if let Err(e) = publisher.put(&json_buf).await {
                        log::error!("Failed to publish display frame on {pub_topic}: {e}");
                      }
                    }
                    Err(e) => {
                      log::error!("Failed to serialize display frame envelope: {e}");
                    }
                  }
                }
              });

              let frame_duration = std::time::Duration::from_secs_f32(1.0 / target_fps as f32);
              let frame_sender = Some(frame_tx);
              let unpadded_buffer = Vec::with_capacity((width as usize) * (height as usize) * 4);
              let qoi_buffer = vec![0u8; qoi::encode_max_len(width as u32, height as u32, 4)];

              // Spawn persistent tracking entity for offscreen render-to-texture display
              commands
                .spawn((
                  ProxiedDisplayTarget {
                    id: display_id.clone(),
                    image_handle: image_handle.clone(),
                    width: width as u16,
                    height: height as u16,
                    target_fps,
                    timer: Timer::new(frame_duration, TimerMode::Repeating),
                    seq: 0,
                    pending_readback: false,
                    mirror: is_mirrored,
                    frame_sender,
                    unpadded_buffer,
                    qoi_buffer,
                  },
                  layer.clone(),
                ));

              #[cfg(feature = "compositor")]
              if is_mirrored {
                use bevy::window::{Window, WindowMode, WindowTheme};

                let scale_factor = (800 / width.max(1)).max(1).min(16);
                let win_w = (width * scale_factor) as f32;
                let win_h = (height * scale_factor) as f32;

                let mirror_window = commands
                  .spawn((
                    Window {
                      title: format!("Clover Mirror: {} ({}x{})", display_id, width, height),
                      mode: WindowMode::Windowed,
                      window_theme: Some(WindowTheme::Dark),
                      resolution: (win_w, win_h).into(),
                      resizable: true,
                      ..Default::default()
                    },
                    DisplayWindow {
                      id: format!("{}_mirror", display_id),
                    },
                  ))
                  .id();

                *next_layer += 1;
                let mirror_layer_idx = *next_layer;
                let mirror_layer = RenderLayers::layer(mirror_layer_idx);

                commands.spawn((
                  Camera2d::default(),
                  Camera {
                    target: RenderTarget::Window(WindowRef::Entity(mirror_window)),
                    order: 10,
                    ..default()
                  },
                  mirror_layer.clone(),
                ));

                commands.spawn((
                  Sprite {
                    image: image_handle.clone(),
                    custom_size: Some(Vec2::new(win_w, win_h)),
                    ..default()
                  },
                  mirror_layer,
                ));

                info!(
                  "Spawned desktop mirror window for display '{display_id}' ({}x{} scaled to {}x{})",
                  width, height, win_w, win_h
                );
              }

              let display_kind = if is_simulated { "Simulated" } else { "ModManProxy" };
              info!(
                "Initialized {display_kind} display '{display_id}' ({}x{} @ {} FPS, layer {}, streaming to Zenoh)",
                width, height, target_fps, layer_idx
              );
            }
            crate::server::modman::components::video::displays::models::ConnectionType::Stream(
              stream_config,
            ) => todo!(),
          }
        }
        AnyDisplayComponent::Virtual(virtual_display_component) => {
          debug!("Spawing VDisplay: {}", display_id.clone());
          // TODO: Spawn a composition for this display.

          commands.spawn((
            virtual_display_component,
            VirtualDisplayID {
              id: display_id.clone(),
            },
          ));

          debug!("Done spawning VDisplay: {}!", display_id.clone());
        }
      }
    }
    Err(_e) => {}
  }
}

/// Periodically spawns transient, one-off GPU readback requests for proxied displays at their target framerate.
pub fn trigger_display_readbacks(
  time: Res<Time>,
  mut query: Query<(Entity, &mut ProxiedDisplayTarget)>,
  mut commands: Commands,
) {
  for (entity, mut proxy) in &mut query {
    if proxy.frame_sender.is_none() {
      continue;
    }
    proxy.timer.tick(time.delta());
    if proxy.timer.just_finished() && !proxy.pending_readback {
      proxy.pending_readback = true;
      commands
        .spawn((
          Readback::texture(proxy.image_handle.clone()),
          DisplayReadbackTag { target: entity },
        ))
        .observe(on_display_readback_complete);
    }
  }
}

/// Handles completion of a one-off GPU texture readback, encodes QOI, publishes to Zenoh, and despawns the transient readback entity.
pub fn on_display_readback_complete(
  trigger: Trigger<ReadbackComplete>,
  tag_query: Query<&DisplayReadbackTag>,
  mut display_query: Query<&mut ProxiedDisplayTarget>,
  mut commands: Commands,
) {
  let transient_entity = trigger.target();
  commands.entity(transient_entity).despawn();

  let Ok(tag) = tag_query.get(transient_entity) else {
    return;
  };
  let Ok(mut proxy) = display_query.get_mut(tag.target) else {
    return;
  };

  proxy.pending_readback = false;

  let ProxiedDisplayTarget {
    ref id,
    width,
    height,
    ref mut seq,
    ref mut unpadded_buffer,
    ref mut qoi_buffer,
    ref frame_sender,
    ..
  } = *proxy;

  let curr_seq = *seq;
  *seq = curr_seq.wrapping_add(1);

  let raw_bytes = &trigger.event().0;

  // wgpu row alignment requires rows to be aligned to 256 bytes
  let pixel_size = 4usize; // RGBA8
  let width_usize = width as usize;
  let height_usize = height as usize;
  let initial_row_bytes = width_usize * pixel_size;
  let buffered_row_bytes = (initial_row_bytes + 255) & !255;

  let total_unpadded_bytes = initial_row_bytes * height_usize;
  if unpadded_buffer.capacity() < total_unpadded_bytes {
    unpadded_buffer.reserve(total_unpadded_bytes - unpadded_buffer.capacity());
  }

  let src_slice: &[u8] = if buffered_row_bytes > initial_row_bytes {
    unpadded_buffer.clear();
    let mut take_offset = 0;
    for _ in 0..height {
      unpadded_buffer.extend_from_slice(&raw_bytes[take_offset..take_offset + initial_row_bytes]);
      take_offset += buffered_row_bytes;
    }
    unpadded_buffer
  } else {
    raw_bytes.as_slice()
  };

  let Some(ref sender) = frame_sender else {
    return;
  };

  let max_qoi_len = qoi::encode_max_len(width as u32, height as u32, 4);
  if qoi_buffer.len() < max_qoi_len {
    qoi_buffer.resize(max_qoi_len, 0);
  }

  match qoi::encode_to_buf(&mut qoi_buffer[..], src_slice, width as u32, height as u32) {
    Ok(qoi_len) => {
      let qoi_data = qoi_buffer[..qoi_len].to_vec();
      let timestamp_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

      let envelope = DisplayFrameEnvelope {
        seq: curr_seq,
        timestamp_ms,
        width,
        height,
        qoi_data,
      };

      // Send to bounded worker; if worker is still publishing previous frame, drop this frame
      let _ = sender.try_send(envelope);
    }
    Err(e) => {
      log::error!("Failed to QOI encode display frame for {id}: {e}");
    }
  }
}
