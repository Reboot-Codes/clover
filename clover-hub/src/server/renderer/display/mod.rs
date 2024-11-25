use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use simple_error::SimpleError;
use tokio::task::JoinHandle;

pub struct DisplayState {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySpec {
  pub id: String
}

pub async fn register_display<T>(display_map: Arc<HashMap<String, DisplayState>>, display_handles: Arc<HashMap<String, JoinHandle<T>>>, display_spec: DisplaySpec) -> Result<(), SimpleError> {
  let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
    backends: wgpu::Backends::PRIMARY,
    ..Default::default()
  });

  let adapter = instance
    .request_adapter(&wgpu::RequestAdapterOptions {
      power_preference: wgpu::PowerPreference::default(),
      compatible_surface: None,
      force_fallback_adapter: false
    })
    .await
    .unwrap();
  
  let (device, queue) = adapter
    .request_device(&Default::default(), None)
    .await
    .unwrap();

  let texture_size = 256u32;

  let texture_desc = wgpu::TextureDescriptor {
    size: wgpu::Extent3d {
      width: texture_size,
      height: texture_size,
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

  // we need to store this for later
  let u32_size = std::mem::size_of::<u32>() as u32;

  let output_buffer_size = (u32_size * texture_size * texture_size) as wgpu::BufferAddress;
  let output_buffer_desc = wgpu::BufferDescriptor {
    size: output_buffer_size,
    usage: wgpu::BufferUsages::COPY_DST
      // this tells wpgu that we want to read this buffer from the cpu
      | wgpu::BufferUsages::MAP_READ,
    label: None,
    mapped_at_creation: false,
  };
  let output_buffer = device.create_buffer(&output_buffer_desc);

  let vs_src = include_str!("shader.vert");
  let fs_src = include_str!("shader.frag");
  let mut compiler = shaderc::Compiler::new().unwrap();
  let vs_spirv = compiler
    .compile_into_spirv(
        vs_src,
        shaderc::ShaderKind::Vertex,
        "shader.vert",
        "main",
        None,
    )
    .unwrap();
  let fs_spirv = compiler
    .compile_into_spirv(
      fs_src,
      shaderc::ShaderKind::Fragment,
      "shader.frag",
      "main",
      None,
    ).unwrap();
  let vs_data = wgpu::util::make_spirv(vs_spirv.as_binary_u8());
  let fs_data = wgpu::util::make_spirv(fs_spirv.as_binary_u8());
  let vs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Vertex Shader"),
    source: vs_data,
  });
  let fs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
    label: Some("Fragment Shader"),
    source: fs_data,
  });

  let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
    label: Some("Render Pipeline Layout"),
    bind_group_layouts: &[],
    push_constant_ranges: &[],
  });

  let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
    label: Some("Render Pipeline"),
    layout: Some(&render_pipeline_layout),
    vertex: wgpu::VertexState {
      module: &vs_module,
      entry_point: Some("main"),
      buffers: &[],
      compilation_options: Default::default()
    },
    fragment: Some(wgpu::FragmentState {
      module: &fs_module,
      entry_point: Some("main"),
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


  Ok(())
}
