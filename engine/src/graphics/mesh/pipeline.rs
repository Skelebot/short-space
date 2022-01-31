use crate::{
    assets::AssetLoader,
    graphics::{WGSL_SHADERS_DIR, WGSL_SHADERS_EXT},
};
use const_format::concatcp;

use super::{
    material::{MaterialFactors, MaterialShading},
    Vertex,
};

const MESH_VERTEX_SHADER_NAME: &str = "mesh";

const UNTEXTURED_SHADER_NAME: &str = "untex";
const UNTEXTURED_UNLIT_SHADER_NAME: &str = "untex_unlit";
const TEXTURED_SHADER_NAME: &str = "tex";
const TEXTURED_UNLIT_SHADER_NAME: &str = "tex_unlit";
const UNTEXTURED_EMISSIVE_SHADER_NAME: &str = "emissive_untex";
const TEXTURED_EMISSIVE_SHADER_NAME: &str = "emissive";

macro_rules! bind_group_layout_entries {
    (untextured) => {
        &[
            // Material factors
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<MaterialFactors>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            },
        ]
    };
    (textured) => {
        &[
            // Material factors
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<MaterialFactors>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            },
            // Texture sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                count: None,
            },
            // Diffuse texture
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
        ]
    };
}

pub struct MeshPipeline {
    pub part_bind_group_layout: wgpu::BindGroupLayout,
    pub pipeline: wgpu::RenderPipeline,
}

impl MeshPipeline {
    fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
        part_bind_group_layout: wgpu::BindGroupLayout,
        vs_module: wgpu::ShaderModule,
        fs_module: wgpu::ShaderModule,
    ) -> Self {
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[
                global_bind_group_layout,
                mesh_bind_group_layout,
                &part_bind_group_layout,
            ],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &Vertex::vertex_attrs(),
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[surface_config.format.into()],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // TODO: DEPTH_CLAMPING feature (?)
            }),
            // TODO: Multisample antialiasing
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });
        MeshPipeline {
            part_bind_group_layout,
            pipeline,
        }
    }

    pub fn shaded(
        ty: MaterialShading,
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
        asset_loader: &AssetLoader,
    ) -> Self {
        let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                asset_loader
                    .load_str(concatcp!(
                        WGSL_SHADERS_DIR,
                        MESH_VERTEX_SHADER_NAME,
                        WGSL_SHADERS_EXT
                    ))
                    .unwrap()
                    .into(),
            ),
        });
        use MaterialShading::*;
        let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(
                asset_loader
                    .load_str(match ty {
                        Untextured => {
                            concatcp!(WGSL_SHADERS_DIR, UNTEXTURED_SHADER_NAME, WGSL_SHADERS_EXT)
                        }
                        UntexturedUnlit => concatcp!(
                            WGSL_SHADERS_DIR,
                            UNTEXTURED_UNLIT_SHADER_NAME,
                            WGSL_SHADERS_EXT,
                        ),
                        Textured => {
                            concatcp!(WGSL_SHADERS_DIR, TEXTURED_SHADER_NAME, WGSL_SHADERS_EXT,)
                        }
                        TexturedUnlit => concatcp!(
                            WGSL_SHADERS_DIR,
                            TEXTURED_UNLIT_SHADER_NAME,
                            WGSL_SHADERS_EXT,
                        ),
                        TexturedEmissive => concatcp!(
                            WGSL_SHADERS_DIR,
                            TEXTURED_EMISSIVE_SHADER_NAME,
                            WGSL_SHADERS_EXT,
                        ),
                        UntexturedEmissive => concatcp!(
                            WGSL_SHADERS_DIR,
                            UNTEXTURED_EMISSIVE_SHADER_NAME,
                            WGSL_SHADERS_EXT,
                        ),
                    })
                    .unwrap()
                    .into(),
            ),
        });

        let part_bind_group_layout = if ty.is_textured() {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: bind_group_layout_entries!(textured),
            })
        } else {
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: bind_group_layout_entries!(untextured),
            })
        };

        MeshPipeline::new(
            device,
            surface_config,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }
}
