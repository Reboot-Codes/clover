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
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::render::view::RenderLayers;
use bevy::render::view::screenshot::{Screenshot, ScreenshotCaptured};
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

/// Tracking component for an offscreen display proxied through ModMan or rendered for testing.
#[derive(Component)]
pub struct ProxiedDisplayTarget {
  /// Unique display component identifier on Zenoh.
  pub id: String,
  /// Asset handle for the offscreen render target image.
  pub image_handle: Handle<Image>,
  /// Configured framerate floor (default >= 24 FPS).
  pub target_fps: u32,
  /// Repeating timer governing frame extraction intervals.
  pub timer: Timer,
  /// Monotonically increasing frame sequence counter.
  pub seq: u64,
  /// Whether an asynchronous frame screenshot/extraction task is in flight.
  pub pending_capture: bool,
  /// Whether desktop mirror window spawning is enabled for this target.
  pub mirror: bool,
}

/// Ingests display registration requests from [`SystemUIIPC::display_registration_queue`]
/// and initializes cameras, render targets, textures, and desktop preview windows.
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
              let frame_duration = std::time::Duration::from_secs_f32(1.0 / target_fps as f32);
              let is_simulated = matches!(
                physical_display_component.connection,
                crate::server::modman::components::video::displays::models::ConnectionType::Simulated
              ) || display_id == "debug_internal_display"
                || display_id == "debug_external_display";
              let is_mirrored = physical_display_component.mirror.unwrap_or(false);

              // Spawn tracking entity for frame extraction
              commands.spawn((
                ProxiedDisplayTarget {
                  id: display_id.clone(),
                  image_handle: image_handle.clone(),
                  target_fps,
                  timer: Timer::new(frame_duration, TimerMode::Repeating),
                  seq: 0,
                  pending_capture: false,
                  mirror: is_mirrored,
                },
                layer.clone(),
              ));

              #[cfg(feature = "compositor")]
              if is_mirrored && !is_simulated {
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

              info!(
                "Initialized ModManProxy display '{display_id}' ({}x{} @ {} FPS, layer {})",
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

/// Periodic frame extraction system for offscreen ModManProxy displays.
pub fn capture_proxied_displays(
  time: Res<Time>,
  mut query: Query<(Entity, &mut ProxiedDisplayTarget)>,
  mut commands: Commands,
  ipc: Res<SystemUIIPC>,
) {
  for (entity, mut proxy) in &mut query {
    proxy.timer.tick(time.delta());
    if proxy.timer.just_finished() && !proxy.pending_capture {
      proxy.pending_capture = true;
      let display_id = proxy.id.clone();
      let session = ipc.zenoh_session.clone();
      let tokio_handle = ipc.tokio_handle.clone();
      let target_entity = entity;
      let seq = proxy.seq;
      proxy.seq = proxy.seq.wrapping_add(1);

      commands
        .spawn(Screenshot::image(proxy.image_handle.clone()))
        .observe(
          move |trigger: Trigger<ScreenshotCaptured>,
                mut targets: Query<&mut ProxiedDisplayTarget>| {
            if let Ok(mut target) = targets.get_mut(target_entity) {
              target.pending_capture = false;
            }
            let captured_img = &trigger.event().0;
            let width = captured_img.texture_descriptor.size.width as u16;
            let height = captured_img.texture_descriptor.size.height as u16;

            if let Some(ref raw_bytes) = captured_img.data {
              match qoi::encode_to_vec(raw_bytes, width as u32, height as u32) {
                Ok(qoi_data) => {
                  let timestamp_ms = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                  let envelope = DisplayFrameEnvelope {
                    seq,
                    timestamp_ms,
                    width,
                    height,
                    qoi_data,
                  };

                  let topic = format!(
                    "com/reboot-codes/clover/hub/modman/components/by-id/{display_id}/display/frame"
                  );
                  match serde_json::to_string(&envelope) {
                    Ok(json) => {
                      let session_clone = session.clone();
                      tokio_handle.spawn(async move {
                        if let Err(e) = session_clone.put(&topic, json).await {
                          log::error!("Failed to publish display frame on {topic}: {e}");
                        }
                      });
                    }
                    Err(e) => {
                      log::error!("Failed to serialize display frame envelope: {e}");
                    }
                  }
                }
                Err(e) => {
                  log::error!("Failed to QOI encode display frame for {display_id}: {e}");
                }
              }
            }
          },
        );
    }
  }
}
