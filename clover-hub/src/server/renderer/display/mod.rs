mod vertices;

use std::{collections::HashMap, mem, sync::Arc};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use tokio::sync::{mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender}, oneshot, Mutex};
use tokio_util::sync::CancellationToken;
use overflow_proof::Checked;
use vertices::Vertex;
use wgpu::{include_wgsl, util::DeviceExt};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySize {
  width: u32,
  height: u32,
}

#[derive(Debug, Clone)]
pub struct DisplayState<'a> {
  pub device_adapter: Arc<(wgpu::Device, wgpu::Queue)>,
  pub size: DisplaySize,
  pub buffer_desc: wgpu::BufferDescriptor<'a>,
  pub frame_number: Checked<u64>,
  pub id: String,
  pub render_pipeline: Arc<wgpu::RenderPipeline>,
  pub textures: HashMap<String, Arc<(wgpu::TextureDescriptor<'a>, wgpu::Texture, wgpu::TextureView)>>,
  pub frames: UnboundedSender<Vec<u8>>,
  pub cancellation_token: CancellationToken,
  pub vertex_buffer: Arc<wgpu::Buffer>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySpec {
  pub id: String,
  pub size: DisplaySize
}

const PIXEL_SIZE: u32 = mem::size_of::<[u8;4]>() as u32;

pub fn sizes(texture_width: u32) -> (u32, u32, u32) {
  let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
  let unpadded_bytes_per_row = PIXEL_SIZE * texture_width;
  let padding = (align - unpadded_bytes_per_row % align) % align;
  let padded_bytes_per_row = unpadded_bytes_per_row + padding;

  (unpadded_bytes_per_row, padding, padded_bytes_per_row)
}

const VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
    Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
    Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
];

impl DisplayState<'_> {
  pub fn new(
    device_adapter: Arc<(wgpu::Device, wgpu::Queue)>,
    size: DisplaySize,
    id: String,
    frame_tx: UnboundedSender<Vec<u8>>,
    cancellation_token: CancellationToken
  ) -> Self {
    let device = &device_adapter.0;
    let (_, _, padded_bytes_per_row) = sizes(size.width);

    let buffer_size = (padded_bytes_per_row * size.height) as wgpu::BufferAddress;
    let buffer_desc = wgpu::BufferDescriptor {
      size: buffer_size,
      usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
      label: Some("Output Buffer"),
      mapped_at_creation: false,
    };

    let mut textures = HashMap::new();

    let texture_desc = wgpu::TextureDescriptor {
      size: wgpu::Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
      },
      mip_level_count: 1,
      sample_count: 1,
      dimension: wgpu::TextureDimension::D2,
      format: wgpu::TextureFormat::Rgba8UnormSrgb,
      usage: wgpu::TextureUsages::COPY_SRC
        | wgpu::TextureUsages::RENDER_ATTACHMENT
        ,
      label: None,
      view_formats: &[],
    };

    let texture = device.create_texture(&texture_desc);
    let texture_view = texture.create_view(&Default::default());
    textures.insert("default".to_string(), Arc::new((texture_desc.clone(), texture, texture_view)));

    let shader = device.create_shader_module(include_wgsl!("./shaders/flat_shader.wgsl"));

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
      label: Some("Render Pipeline Layout"),
      bind_group_layouts: &[],
      push_constant_ranges: &[],
    });

    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
      label: Some("Render Pipeline"),
      layout: Some(&render_pipeline_layout),
      vertex: wgpu::VertexState {
        module: &shader,
        entry_point: Some("vs_main"),
        buffers: &[Vertex::desc()],
        compilation_options: Default::default()
      },
      fragment: Some(wgpu::FragmentState {
        module: &shader,
        entry_point: Some("fs_main"),
        targets: &[Some(wgpu::ColorTargetState {
          format: texture_desc.format,
          blend: Some(wgpu::BlendState::REPLACE),
          write_mask: wgpu::ColorWrites::ALL,
        })],
        compilation_options: Default::default()
      }),
      primitive: wgpu::PrimitiveState {
        topology: wgpu::PrimitiveTopology::TriangleList,
        strip_index_format: None,
        front_face: wgpu::FrontFace::Ccw,
        cull_mode: Some(wgpu::Face::Back),
        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
        polygon_mode: wgpu::PolygonMode::Fill,
        unclipped_depth: false,
        conservative: false
      },
      depth_stencil: None,
      multisample: wgpu::MultisampleState {
        count: 1,
        mask: !0,
        alpha_to_coverage_enabled: false,
      },
      multiview: None,
      cache: None
    });

    let vertex_buffer = device.create_buffer_init(
      &wgpu::util::BufferInitDescriptor {
        label: Some("Vertex Buffer"),
        contents: bytemuck::cast_slice(VERTICES),
        usage: wgpu::BufferUsages::VERTEX,
      }
    );

    Self {
      device_adapter: device_adapter.clone(),
      size: size.clone(),
      buffer_desc: buffer_desc.clone(),
      frame_number: Checked::new(0),
      id,
      render_pipeline: Arc::new(render_pipeline),
      textures,
      frames: frame_tx,
      cancellation_token,
      vertex_buffer: Arc::new(vertex_buffer)
    }
  }

  pub async fn render(mut self: Self) {
    if self.cancellation_token.is_cancelled() {return;}

    match {self.frame_number + 1}.check().ok_or(SimpleError::new("Frame counter overflow.")) {
      Ok(val) => {
        self.frame_number = val;
      },
      Err(e) => {
        info!("Resetting frame counter due to: {}", e);
      }
    }

    debug!("Display: {}, frame: {}, rendering...", self.id.clone(), self.frame_number.to_string());

    let device = &self.device_adapter.0;
    let queue = &self.device_adapter.1;

    let encoder_str = format!("Display: {}, frame: {}, encoder", self.id.clone(), self.frame_number.to_string());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
      label: Some(&encoder_str),
    });
    let output_buffer = Arc::new(device.create_buffer(&self.buffer_desc));

    for (texture_id, texture_spec) in self.textures {
      let texture_desc = texture_spec.0.clone();
      let texture = &texture_spec.1;
      let texture_view = &texture_spec.2;
      let pass_str = format!("Display: {}, frame: {}, render pass, texture: {}", self.id.clone(), self.frame_number.to_string(), texture_id);
      let render_pass_desc = wgpu::RenderPassDescriptor {
        label: Some(&pass_str),
        color_attachments: &[
          Some(wgpu::RenderPassColorAttachment {
            view: texture_view,
            resolve_target: None,
            ops: wgpu::Operations {
              load: wgpu::LoadOp::Clear(wgpu::Color {
                r: 0.1,
                g: 0.2,
                b: 0.3,
                a: 1.0,
              }),
              store: wgpu::StoreOp::Store,
            },
          })
        ],
        depth_stencil_attachment: None,
        timestamp_writes: None,
        occlusion_query_set: None
      };

      let mut render_pass = encoder.begin_render_pass(&render_pass_desc);
  
      render_pass.set_pipeline(&self.render_pipeline);
      render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
      render_pass.draw(0..(VERTICES.len() as u32), 0..1);
      drop(render_pass);

      encoder.copy_texture_to_buffer(
        wgpu::ImageCopyTexture {
          aspect: wgpu::TextureAspect::All,
          texture: &texture,
          mip_level: 0,
          origin: wgpu::Origin3d::ZERO,
        },
        wgpu::ImageCopyBuffer {
          buffer: &output_buffer,
          layout: wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(PIXEL_SIZE * self.size.width),
            rows_per_image: Some(self.size.height),
          },
        },
        texture_desc.size,
      );
    }

    queue.submit(Some(encoder.finish()));

    {
      let buffer_slice = output_buffer.slice(..);
  
      // NOTE: We have to create the mapping THEN device.poll() before await
      // the future. Otherwise the application will freeze.
      let (tx, rx) = oneshot::channel();
      
      buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
      });
      
      device.poll(wgpu::Maintain::Wait);
  
      match rx.await {
        Ok(rx_l2) => {
          match rx_l2 {
            Ok(_) => {
              let (unpadded_bytes_per_row, _, padded_bytes_per_row) = sizes(self.size.width);

              let padded_data = buffer_slice.get_mapped_range();
              let data = padded_data
                .chunks(padded_bytes_per_row as _)
                .map(|chunk| { &chunk[..unpadded_bytes_per_row as _]})
                .flatten()
                .map(|x| { *x })
                .collect::<Vec<_>>();
              drop(padded_data);

              match self.frames.send(data) {
                Ok(_) => {},
                Err(e) => {

                }
              };

              debug!("Display: {}, frame: {}, rendered!", self.id.clone(), self.frame_number.to_string());
            },
            Err(e) => {
              
            },
          }
        },
        Err(e) => {
          
        },
      }
    }
    output_buffer.unmap();
  }

  pub fn cleanup(self: Self) {
  }

  pub fn sample_obj(self: Self) {
    &[
      Vertex { position: [0.0, 0.5, 0.0], color: [1.0, 0.0, 0.0] },
      Vertex { position: [-0.5, -0.5, 0.0], color: [0.0, 1.0, 0.0] },
      Vertex { position: [0.5, -0.5, 0.0], color: [0.0, 0.0, 1.0] },
    ];

  }
}

pub async fn register_display/*<T>*/(
  device_adapter: Arc<(wgpu::Device, wgpu::Queue)>,
  display_map: Arc<Mutex<HashMap<String, Arc<DisplayState<'_>>>>>, 
  // display_handles: Arc<HashMap<String, JoinHandle<T>>>, 
  display_spec: DisplaySpec,
  cancellation_token: CancellationToken
) -> Result<UnboundedReceiver<Vec<u8>>, SimpleError> {
  let (frames_tx, frames_rx) = unbounded_channel();

  let state_og = Arc::new(DisplayState::new(
    device_adapter.clone(), 
    display_spec.size, 
    display_spec.id.clone(),
    frames_tx,
    cancellation_token.clone()
  ));

  display_map.lock().await.insert(display_spec.id, state_og.clone());

  tokio::task::spawn(async move {
    tokio::select! {
      _ = async {
        while !cancellation_token.is_cancelled() {
          <DisplayState as Clone>::clone(&state_og).render().await;
        }
      } => {},
      _ = cancellation_token.cancelled() => {
        <DisplayState as Clone>::clone(&state_og).cleanup();
      }
    }
  });

  Ok(frames_rx)
}
