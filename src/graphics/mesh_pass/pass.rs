use anyhow::Result;
use spacetime::PhysicsTimer;

use crate::{assets::AssetLoader, graphics::Pass, player::Atlas, spacetime};
use wgpu::util::DeviceExt;

use crate::graphics::Camera;
use crate::physics;

use legion::{Entity, IntoQuery, Resources, World};

use super::{material::MaterialShading, pipeline::MeshPipeline, GlobalUniforms, RenderMesh};

pub struct MeshPassPipelines {
    pub untextured: MeshPipeline,
    pub untextured_unlit: MeshPipeline,
    pub textured: MeshPipeline,
    pub textured_unlit: MeshPipeline,
    pub textured_emissive: MeshPipeline,
    pub untextured_emissive: MeshPipeline,
}

pub struct MeshPass {
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,

    pub global_uniform_buf: wgpu::Buffer,
    pub mesh_bind_group_layout: wgpu::BindGroupLayout,

    pub pipelines: MeshPassPipelines,

    depth_texture: wgpu::Texture,
    // For clearing
    depth_texture_view: wgpu::TextureView,
}

impl MeshPass {
    pub fn new(
        device: &mut wgpu::Device,
        sc_desc: &wgpu::SwapChainDescriptor,
        _world: &mut World,
        resources: &mut Resources,
    ) -> Result<MeshPass> {
        // Set 1
        let mesh_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Mesh matrix (na::Matrix4 / mat4)
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        // Set 0
        let global_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[
                    // Globals
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStage::VERTEX | wgpu::ShaderStage::FRAGMENT,
                        ty: wgpu::BindingType::UniformBuffer {
                            dynamic: false,
                            min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<
                                GlobalUniforms,
                            >()
                                as wgpu::BufferAddress),
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(global_uniform_buf.slice(..)),
            }],
        });

        // Depth testing
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });

        let depth_texture_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

        // For loading shaders
        let pipelines = {
            let asset_loader = resources
                .get::<AssetLoader>()
                .expect("Asset loader not found, cannot load shaders");
            MeshPassPipelines {
                untextured: MeshPipeline::shaded(
                    MaterialShading::Untextured,
                    device,
                    sc_desc,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                untextured_unlit: MeshPipeline::shaded(
                    MaterialShading::UntexturedUnlit,
                    device,
                    sc_desc,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                textured: MeshPipeline::shaded(
                    MaterialShading::Textured,
                    device,
                    sc_desc,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                textured_unlit: MeshPipeline::shaded(
                    MaterialShading::TexturedUnlit,
                    device,
                    sc_desc,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                textured_emissive: MeshPipeline::shaded(
                    MaterialShading::TexturedEmissive,
                    device,
                    sc_desc,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                untextured_emissive: MeshPipeline::shaded(
                    MaterialShading::UntexturedEmissive,
                    device,
                    sc_desc,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
            }
        };

        let mesh_pass = MeshPass {
            global_bind_group,
            global_bind_group_layout,
            global_uniform_buf,
            pipelines,
            mesh_bind_group_layout,
            depth_texture,
            depth_texture_view,
        };

        Ok(mesh_pass)
    }
}

impl Pass for MeshPass {
    fn resize(
        &mut self,
        device: &mut wgpu::Device,
        _queue: &mut wgpu::Queue,
        sc_desc: &mut wgpu::SwapChainDescriptor,
        world: &mut legion::World,
        _resources: &legion::Resources,
    ) -> Result<()> {
        self.depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: sc_desc.width,
                height: sc_desc.height,
                depth: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsage::OUTPUT_ATTACHMENT,
        });
        self.depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Update every camera's aspect ratio
        let mut query = <&mut Camera>::query();
        query.for_each_mut(world, |camera| {
            camera.update_aspect(sc_desc.width as f32 / sc_desc.height as f32);
        });
        Ok(())
    }
    fn render(
        &mut self,
        _device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        // Usually the frame
        target: &mut wgpu::SwapChainTexture,
        world: &legion::World,
        resources: &legion::Resources,
    ) -> Result<()> {
        let lerp = {
            let physics_timer = resources.get::<PhysicsTimer>().unwrap();
            physics_timer.lerp() as f32
        };

        // Upload global uniforms
        {
            let camera_entity = resources.get::<Atlas>().unwrap().camera;

            let mut cam_query = <(&spacetime::Position, &Camera)>::query();
            let (position, camera) = cam_query.get(world, camera_entity).unwrap();

            let cam_pos = position.current(lerp);

            let view_proj = camera.get_view_projection_matrix(&cam_pos);
            let global_uniforms = GlobalUniforms {
                view_proj: view_proj.into(),
                camera_pos: cam_pos.translation.vector.into(),
            };
            queue.write_buffer(
                &self.global_uniform_buf,
                0,
                bytemuck::bytes_of(&global_uniforms),
            );
        }

        // Select every entity with a RenderMesh, position and maybe a scale
        // TODO: update buffers only if the position or scale have been changed (maybe_changed filter)
        let mut mesh_query =
            <(&RenderMesh, &spacetime::Position, Option<&spacetime::Scale>)>::query();

        // Upload mesh model transform matrices to every model's buffer
        for (rmesh, position, maybe_scale) in mesh_query.iter(world) {
            let mut transform = position.current(lerp).to_homogeneous();
            if let Some(scale) = maybe_scale {
                transform = transform.prepend_nonuniform_scaling(scale);
            }
            let transform: [[f32; 4]; 4] = transform.into();
            queue.write_buffer(&rmesh.uniform_buf, 0, bytemuck::bytes_of(&transform));
        }

        // Begin rendering

        // Render every mesh
        encoder.push_debug_group("forward rendering pass");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                // Clear the frame
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &target.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear the framebuffer with a color
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.01,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                // Clear the depth buffer
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachmentDescriptor {
                    attachment: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });

            // Select every RenderMesh in the world
            render_pass.set_bind_group(0, &self.global_bind_group, &[]);
            for (mesh, _, _) in mesh_query.iter(world) {
                render_pass.set_bind_group(1, &mesh.bind_group, &[]);
                for part in &mesh.parts {
                    // Set the correct pipeline before rendering
                    render_pass.set_pipeline(match part.material.shading {
                        MaterialShading::Untextured => &self.pipelines.untextured.pipeline,
                        MaterialShading::UntexturedUnlit => {
                            &self.pipelines.untextured_unlit.pipeline
                        }
                        MaterialShading::Textured => &self.pipelines.textured.pipeline,
                        MaterialShading::TexturedUnlit => &self.pipelines.textured_unlit.pipeline,
                        MaterialShading::TexturedEmissive => {
                            &self.pipelines.textured_emissive.pipeline
                        }
                        MaterialShading::UntexturedEmissive => {
                            &self.pipelines.untextured_emissive.pipeline
                        }
                    });

                    render_pass.set_bind_group(2, &part.material.bind_group, &[]);
                    render_pass.set_index_buffer(part.index_buf.slice(..));
                    render_pass.set_vertex_buffer(0, part.vertex_buf.slice(..));
                    render_pass.draw_indexed(0..part.index_count as u32, 0, 0..1);
                }
            }
        }
        encoder.pop_debug_group();

        Ok(())
    }
}
