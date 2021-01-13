use crate::assets::AssetLoader;
use const_format::concatcp;

use super::{
    material::{MaterialFactors, MaterialShading},
    Vertex,
};

const COMPILED_SHADERS_DIR: &str = "shaders/compiled/";
const COMPILED_VERTEX_SHADER_EXT: &str = ".vert.spv";
const COMPILED_FRAGMENT_SHADER_EXT: &str = ".frag.spv";

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
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
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
                visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::UniformBuffer {
                    dynamic: false,
                    min_binding_size: wgpu::BufferSize::new(
                        std::mem::size_of::<MaterialFactors>() as wgpu::BufferAddress
                    ),
                },
                count: None,
            },
            // Texture sampler
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::Sampler { comparison: false },
                count: None,
            },
            // Diffuse texture
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStage::FRAGMENT,
                ty: wgpu::BindingType::SampledTexture {
                    multisampled: false,
                    component_type: wgpu::TextureComponentType::Float,
                    dimension: wgpu::TextureViewDimension::D2,
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
        sc_desc: &wgpu::SwapChainDescriptor,
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
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                ..Default::default()
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[sc_desc.format.into()],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
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
        MeshPipeline {
            part_bind_group_layout,
            pipeline,
        }
    }

    pub fn shaded(
        ty: MaterialShading,
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        global_bind_group_layout: &wgpu::BindGroupLayout,
        mesh_bind_group_layout: &wgpu::BindGroupLayout,
        asset_loader: &AssetLoader,
    ) -> Self {
        let vs_module = device.create_shader_module(wgpu::util::make_spirv(
            &asset_loader
                .load_bytes(concatcp!(
                    COMPILED_SHADERS_DIR,
                    MESH_VERTEX_SHADER_NAME,
                    COMPILED_VERTEX_SHADER_EXT
                ))
                .unwrap(),
        ));
        use MaterialShading::*;
        let fs_module = device.create_shader_module(wgpu::util::make_spirv(
            &asset_loader
                .load_bytes(match ty {
                    Untextured => concatcp!(
                        COMPILED_SHADERS_DIR,
                        UNTEXTURED_SHADER_NAME,
                        COMPILED_FRAGMENT_SHADER_EXT
                    ),
                    UntexturedUnlit => concatcp!(
                        COMPILED_SHADERS_DIR,
                        UNTEXTURED_UNLIT_SHADER_NAME,
                        COMPILED_FRAGMENT_SHADER_EXT
                    ),
                    Textured => concatcp!(
                        COMPILED_SHADERS_DIR,
                        TEXTURED_SHADER_NAME,
                        COMPILED_FRAGMENT_SHADER_EXT
                    ),
                    TexturedUnlit => concatcp!(
                        COMPILED_SHADERS_DIR,
                        TEXTURED_UNLIT_SHADER_NAME,
                        COMPILED_FRAGMENT_SHADER_EXT
                    ),
                    TexturedEmissive => concatcp!(
                        COMPILED_SHADERS_DIR,
                        TEXTURED_EMISSIVE_SHADER_NAME,
                        COMPILED_FRAGMENT_SHADER_EXT
                    ),
                    UntexturedEmissive => concatcp!(
                        COMPILED_SHADERS_DIR,
                        UNTEXTURED_EMISSIVE_SHADER_NAME,
                        COMPILED_FRAGMENT_SHADER_EXT
                    ),
                })
                .unwrap(),
        ));

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
            sc_desc,
            global_bind_group_layout,
            mesh_bind_group_layout,
            part_bind_group_layout,
            vs_module,
            fs_module,
        )
    }
}
