pub mod light;
pub use light::*;

use crate::asset_loader::AssetLoader;

use anyhow::Result;
use legion::{Resources, World};

use super::Vertex;

// TODO: Move to settings (separate struct for grahics settings?)
const SHADOW_RES: u32 = 2048;
const MAX_LIGHTS: usize = 5; // Keep this in sync with shaders

pub struct ShadowPass {
    pub pipeline: wgpu::RenderPipeline,

    pub shadow_texture: wgpu::Texture,
    pub shadow_sampler: wgpu::Sampler,

    pub light_count: usize,
    pub light_uniform_size: wgpu::BufferAddress,
    pub light_uniform_buf: wgpu::Buffer,

    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
}

impl ShadowPass {
    pub fn new(
        device: &mut wgpu::Device,
        _sc_desc: &wgpu::SwapChainDescriptor,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
        _world: &mut World,
        resources: &mut Resources,
    ) -> Result<Self> {
        let light_uniform_size =
            (MAX_LIGHTS * std::mem::size_of::<LightUniforms>()) as wgpu::BufferAddress;
        let light_uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: light_uniform_size,
            usage: wgpu::BufferUsage::UNIFORM
                | wgpu::BufferUsage::COPY_SRC
                | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                height: SHADOW_RES,
                width: SHADOW_RES,
                depth: MAX_LIGHTS as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED,
        });

        let uniform_size = std::mem::size_of::<ShadowUniforms>() as wgpu::BufferAddress;
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStage::VERTEX,
                ty: wgpu::BindingType::UniformBuffer {
                    min_binding_size: wgpu::BufferSize::new(uniform_size),
                    dynamic: false,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            // ?
            bind_group_layouts: &[&bind_group_layout, mesh_bind_group_layout],
            push_constant_ranges: &[],
        });

        let uniform_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: uniform_size,
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
            }],
        });

        // Load the shader
        let vs_module = {
            let asset_loader = resources
                .get::<AssetLoader>()
                .expect("Asset loader not found, cannot load shaders");
            let bytes = asset_loader.load_bytes("shaders/compiled/bake_shadows.vert.spv")?;
            let source = wgpu::util::make_spirv(&bytes);
            device.create_shader_module(source)
        };

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("shadow"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: None,
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                // TODO: Cull front faces for better shadows (?)
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 2, // what bilinear filtering??
                depth_bias_slope_scale: 2.0,
                depth_bias_clamp: 0.0,
                clamp_depth: device.features().contains(wgpu::Features::DEPTH_CLAMPING),
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint32,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &Vertex::vertex_attrs(),
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        Ok(Self {
            pipeline,
            shadow_texture,
            shadow_sampler,
            light_uniform_size,
            light_count: 0,
            light_uniform_buf,
            bind_group,
            uniform_buf,
        })
    }
}
