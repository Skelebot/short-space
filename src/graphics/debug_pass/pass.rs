use color_eyre::eyre::anyhow;
use const_format::concatcp;
use eyre::Result;
use legion::{Resources, World};
use wgpu::util::DeviceExt;

use crate::graphics::{
    COMPILED_FRAGMENT_SHADER_EXT, COMPILED_SHADERS_DIR, COMPILED_VERTEX_SHADER_EXT,
};
use crate::{
    assets::AssetLoader,
    graphics::{mesh_pass::GlobalUniforms, Camera, Pass},
    player::Atlas,
    spacetime::PhysicsTimer,
};

use super::DebugLinesUniforms;

const LINE_SHADER_NAME: &str = "line";

pub struct DebugPass {
    pub per_frame_bind_group_layout: wgpu::BindGroupLayout,
    pub per_frame_bind_group: wgpu::BindGroup,

    pub global_uniform_buf: wgpu::Buffer,
    pub line_uniform_buf: wgpu::Buffer,

    pub vertex_buf: wgpu::Buffer,

    pub pipeline: wgpu::RenderPipeline,
}

impl DebugPass {
    pub fn new(
        device: &wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        _window: &winit::window::Window,
        _queue: &wgpu::Queue,
        _world: &mut World,
        resources: &mut Resources,
    ) -> Result<Self> {
        // Set 0
        let per_frame_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Globals
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                GlobalUniforms,
                            >()
                                as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                    // Line uniforms
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                DebugLinesUniforms,
                            >()
                                as wgpu::BufferAddress),
                        },
                        count: None,
                    },
                ],
            });

        let global_uniforms = GlobalUniforms {
            view_proj: na::Matrix4::identity().into(),
            camera_pos: na::Vector3::identity().into(),
        };

        let line_uniforms = DebugLinesUniforms {
            screen_thickness: [0.0, 0.0],
        };

        let global_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&global_uniforms),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let line_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&line_uniforms),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let per_frame_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &per_frame_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &global_uniform_buf,
                        offset: 0,
                        // FIXME
                        size: None,
                    },
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &line_uniform_buf,
                        offset: 0,
                        // FIXME
                        size: None,
                    },
                },
            ],
        });

        let (vs_module, fs_module) = {
            let asset_loader = resources
                .get::<AssetLoader>()
                .ok_or_else(|| anyhow!("Asset loader not found, cannot load shaders"))?;

            let vs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::util::make_spirv(
                    &asset_loader
                        .load_bytes(concatcp!(
                            COMPILED_SHADERS_DIR,
                            LINE_SHADER_NAME,
                            COMPILED_VERTEX_SHADER_EXT,
                        ))
                        .unwrap(),
                ),
                flags: wgpu::ShaderFlags::default(),
            });
            let fs_module = device.create_shader_module(&wgpu::ShaderModuleDescriptor {
                label: None,
                source: wgpu::util::make_spirv(
                    &asset_loader
                        .load_bytes(concatcp!(
                            COMPILED_SHADERS_DIR,
                            LINE_SHADER_NAME,
                            COMPILED_FRAGMENT_SHADER_EXT,
                        ))
                        .unwrap(),
                ),
                flags: wgpu::ShaderFlags::default(),
            });
            (vs_module, fs_module)
        };

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&per_frame_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<super::Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Instance,
                    attributes: &super::Vertex::vertex_attrs(),
                }],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                strip_index_format: Some(wgpu::IndexFormat::Uint16),
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::None,
                polygon_mode: wgpu::PolygonMode::Fill,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
                // TODO: DEPTH_CLAMPING feature
                clamp_depth: false,
            }),
            // TODO: Multisample antialiasing
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &[sc_desc.format.into()],
            }),
        });

        let vertex_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            mapped_at_creation: false,
            usage: wgpu::BufferUsage::VERTEX,
            // FIXME
            size: 0,
        });

        Ok(DebugPass {
            per_frame_bind_group_layout,
            per_frame_bind_group,
            global_uniform_buf,
            line_uniform_buf,
            pipeline,
            vertex_buf,
        })
    }
}

impl Pass for DebugPass {
    fn resize(
        &mut self,
        _graphics: &crate::graphics::GraphicsShared,
        _sc_desc: &wgpu::SwapChainDescriptor,
        _world: &mut World,
        _resources: &mut Resources,
    ) -> Result<()> {
        Ok(())
    }

    fn render(
        &mut self,
        graphics: &crate::graphics::GraphicsShared,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut wgpu::SwapChainTexture,
        world: &World,
        resources: &Resources,
        depth_texture_view: &wgpu::TextureView,
    ) {
        let lerp = {
            let physics_timer = resources.get::<PhysicsTimer>().unwrap();
            physics_timer.lerp() as f32
        };
        // TODO: Merge with MeshPass
        // Upload global uniforms
        {
            let camera_entity = resources.get::<Atlas>().unwrap().camera;

            use legion::IntoQuery;
            let mut cam_query = <(&crate::spacetime::Position, &Camera)>::query();
            let (position, camera) = cam_query.get(world, camera_entity).unwrap();

            let cam_pos = position.current(lerp);

            let view_proj = camera.view_projection(&cam_pos);
            let global_uniforms = GlobalUniforms {
                view_proj: view_proj.into(),
                camera_pos: cam_pos.translation.vector.into(),
            };
            graphics.queue.write_buffer(
                &self.global_uniform_buf,
                0,
                bytemuck::bytes_of(&global_uniforms),
            );
        }

        // Begin rendering
        encoder.push_debug_group("debug rendering pass");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                // Clear the frame
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear the framebuffer with a color
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                // Clear the depth buffer
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            if let Some(lines) = resources.get::<super::DebugLines>() {
                // Set up buffers
                // TODO: Copy only if anything changed
                {
                    self.vertex_buf =
                        graphics
                            .device
                            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                                label: None,
                                contents: bytemuck::cast_slice(&lines.vec),
                                usage: wgpu::BufferUsage::VERTEX,
                            });
                    graphics.queue.write_buffer(
                        &self.line_uniform_buf,
                        0,
                        bytemuck::bytes_of(&DebugLinesUniforms {
                            screen_thickness: [
                                // TODO: get width/height from sc_desc instead
                                lines.thickness / (graphics.window.inner_size().width as f32),
                                lines.thickness / (graphics.window.inner_size().height as f32),
                            ],
                        }),
                    )
                }
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.per_frame_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
                render_pass.draw(0..4, 0..lines.vec.len() as _);
            }
        }
        encoder.pop_debug_group();
    }
}
