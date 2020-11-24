use bytemuck::{Pod, Zeroable};

use anyhow::{Result, Error};

use wgpu::util::DeviceExt;
use super::Pass;

use crate::graphics::Camera;

use legion::{World, Resources, IntoQuery};

pub struct MeshPass {
    pub pipeline: wgpu::RenderPipeline,
    pub mesh_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,
    pub global_uniform_buf: wgpu::Buffer,
    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,
}

impl MeshPass {
    pub fn new(
        device: &mut wgpu::Device,
        window: &winit::window::Window,
        sc_desc: &wgpu::SwapChainDescriptor,
        _world: &mut World,
        _resources: &mut Resources,
    ) -> Result<MeshPass> {
        // Load shaders from disk
        let vs_module = device.create_shader_module(wgpu::include_spirv!("shader.vert.spv"));
        let fs_module = device.create_shader_module(wgpu::include_spirv!("shader.frag.spv"));

        // Set 0
        let global_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: wgpu::BufferSize::new(
                            std::mem::size_of::<GlobalUniforms>() as wgpu::BufferAddress,
                        ),
                    },
                    count: None
                }
            ]
        });

        // Set 1
        let mesh_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Model matrix (na::Matrix4 / mat4)
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer {
                        dynamic: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // Texture sampler (sampler2D)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                    count: None,
                },
                // Sampled texture (texture2D)
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
            ],
        });

        // Those get uploaded before rendering every frame either way
        let global_uniforms = GlobalUniforms {
            view_proj: na::Matrix4::identity().into(),
            camera_pos: na::Vector3::identity().into(),
        };

        let global_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global uniform buffer"),
            contents: bytemuck::bytes_of(&global_uniforms),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(global_uniform_buf.slice(..)),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("main pipeline layout"),
            bind_group_layouts: &[
                // Set 0
                &global_bind_group_layout,
                // Set 1
                &mesh_bind_group_layout,
            ],
            push_constant_ranges: &[]
        });

        // Depth testing
        let depth_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                label: Some("depth texture"),
                size: wgpu::Extent3d {
                    width: sc_desc.width,
                    height: sc_desc.height,
                    depth: 1
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Depth32Float,
                usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            }
        );

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("main pipeline"),
            layout: Some(&pipeline_layout),
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            //rasterization_state: Some(wgpu::RasterizationStateDescriptor {
            //    front_face: wgpu::FrontFace::Ccw,
            //    //cull_mode: wgpu::CullMode::Back,
            //    cull_mode: wgpu::CullMode::None,
            //    depth_bias: 0,
            //    depth_bias_slope_scale: 0.0,
            //    depth_bias_clamp: 0.0,
            //    clamp_depth: false,
            //}),
            rasterization_state: None,
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[sc_desc.format.into()],
            //    // ?: Why is this repeated?
            //    wgpu::ColorStateDescriptor {
            //        format: swap_chain_desc.format,
            //        color_blend: wgpu::BlendDescriptor::REPLACE,
            //        alpha_blend: wgpu::BlendDescriptor::REPLACE,
            //        write_mask: wgpu::ColorWrite::ALL,
            //    },
            //    wgpu::ColorStateDescriptor {
            //        format: swap_chain_desc.format,
            //        color_blend: wgpu::BlendDescriptor::REPLACE,
            //        alpha_blend: wgpu::BlendDescriptor::REPLACE,
            //        write_mask: wgpu::ColorWrite::ALL,
            //    },
            //],
            depth_stencil_state: Some(wgpu::DepthStencilStateDescriptor {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilStateDescriptor::default(),
            }),
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: 
                        &wgpu::vertex_attr_array![
                            // Position
                            0 => Float3,
                            // Normal
                            1 => Float3,
                            // UV
                            2 => Float2,
                        ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        let mesh_pass = MeshPass {
            pipeline: pipeline,
            mesh_bind_group_layout,
            global_bind_group_layout,
            global_bind_group: global_bind_group,
            global_uniform_buf: global_uniform_buf,
            depth_texture: depth_texture,
            depth_texture_view: depth_texture_view,
        };

        Ok(mesh_pass)
    }
}

impl Pass for MeshPass {
    fn resize(&mut self, sc_desc: &wgpu::SwapChainDescriptor, device: &mut wgpu::Device) -> Result<()> {
        self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
            }
        );
        self.depth_texture_view = self.depth_texture.create_view(&wgpu::TextureViewDescriptor::default());
        Ok(())
    }
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &mut wgpu::Queue,
        // Usually the frame
        target: &mut wgpu::SwapChainTexture,
        world: &legion::World,
        resources: &legion::Resources,
    ) -> Result<()> {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &target.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    // Clear the framebuffer with a color
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                attachment: &self.depth_texture_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        });
        let camera = resources.get::<Camera>().ok_or(Error::msg("Couldn't find the Camera"))?;

        // Upload global uniforms
        let view_proj = camera.get_vp_matrix();
        let global_uniforms = GlobalUniforms {
            view_proj: view_proj.into(),
            camera_pos: na::Vector3::new(0.0, 0.0, 0.0).into(),
        };
        queue.write_buffer(
            &self.global_uniform_buf,
            0,
            bytemuck::bytes_of(&global_uniforms)
        );
        // Draw with our pipeline
        // Per pass
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.global_bind_group, &[]);
        // Per entity
        use crate::physics::*;
        let mut query = <(&Model, &Position, Option<&Scale>)>::query();
        for (model, position, maybe_scale) in query.iter(world) {
            let mut transform = position.to_homogeneous();
            if let Some(scale) = maybe_scale {
                transform = transform.prepend_nonuniform_scaling(scale);
            }
            let transform: [[f32; 4]; 4] = transform.into();
            queue.write_buffer(
                &model.uniform_buf,
                0,
                bytemuck::bytes_of(&transform)
            );
            render_pass.set_bind_group(1, &model.bind_group, &[]);
            // pass.set_bind_group(1, entity_bind_group, &[entity.uniform_offset])
            render_pass.set_index_buffer(model.index_buf.slice(..));
            render_pass.set_vertex_buffer(0, model.vertex_buf.slice(..));
            render_pass.draw_indexed(
                0 .. model.index_count as u32,
                0,
                0..1,
            );
        }
        Ok(())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)] 
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub(crate)view_proj: [[f32; 4]; 4],
    pub(crate)camera_pos: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ModelUniforms {
    pub(crate)model: [[f32; 4]; 4],
}

pub struct ModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    pub texture_img: image::RgbaImage,
}

use crate::asset_loader::AssetLoader;
pub struct Model {
    // TODO: Rc<> for multiple models with the same data (it's all read-only either way)
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
    pub index_count: usize,
    pub uniform_offset: wgpu::DynamicOffset,
}

impl Model {
    pub fn from_data(data: ModelData, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder, pass: &MeshPass) -> Model {

        let vertex_data = data.vertices;
        let index_data = data.indices;

        let vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );
        let index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        let model_uniform = ModelUniforms {
            model: na::Matrix4::identity().into(),
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&model_uniform),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            compare: None,
            // TODO: Set filters to FilterMode::Linear for smoother textures
            ..Default::default()
        });

        let texture = AssetLoader::upload_texture(device, encoder, true, data.texture_img);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            // TODO: Review and customize
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pass.mesh_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),   
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),   
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ]
        });

        Model {
            bind_group: bind_group,
            uniform_buf: uniform_buf,
            vertex_buf: vertex_buf,
            index_buf: index_buf,
            index_count: index_data.len(),
            uniform_offset: 0,
        }
    }
}