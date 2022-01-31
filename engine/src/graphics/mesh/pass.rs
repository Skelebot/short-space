use eyre::{eyre::anyhow, Result};
use legion::{IntoQuery, Resources, World};
use spacetime::PhysicsTimer;
use wgpu::util::DeviceExt;

use crate::graphics::{Camera, GraphicsShared, MainCamera, Pass};
use crate::{assets::AssetLoader, spacetime};

use super::render_mesh::RenderMesh;
use super::{material::MaterialShading, pipeline::MeshPipeline};
use crate::graphics::GlobalUniforms;

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
    pub mesh_bind_group_layout: std::rc::Rc<wgpu::BindGroupLayout>,

    pub pipelines: std::rc::Rc<MeshPassPipelines>,
}

impl MeshPass {
    pub fn new(
        device: &wgpu::Device,
        surface_config: &wgpu::SurfaceConfiguration,
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
                        visibility: wgpu::ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
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
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
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
                ],
            });

        // Those get uploaded before rendering every frame either way
        let global_uniforms = GlobalUniforms {
            view_proj: na::Matrix4::identity().into(),
            camera_pos: na::Vector3::identity().into(),
            _padding: [0.0; 9],
        };

        let global_uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Global uniform buffer"),
            contents: bytemuck::bytes_of(&global_uniforms),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let global_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &global_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &global_uniform_buf,
                    offset: 0,
                    // FIXME
                    size: None,
                }),
            }],
        });

        // For loading shaders
        let pipelines = {
            let asset_loader = resources
                .get::<AssetLoader>()
                .ok_or_else(|| anyhow!("Asset loader not found, cannot load shaders"))?;
            MeshPassPipelines {
                untextured: MeshPipeline::shaded(
                    MaterialShading::Untextured,
                    device,
                    surface_config,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                untextured_unlit: MeshPipeline::shaded(
                    MaterialShading::UntexturedUnlit,
                    device,
                    surface_config,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                textured: MeshPipeline::shaded(
                    MaterialShading::Textured,
                    device,
                    surface_config,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                textured_unlit: MeshPipeline::shaded(
                    MaterialShading::TexturedUnlit,
                    device,
                    surface_config,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                textured_emissive: MeshPipeline::shaded(
                    MaterialShading::TexturedEmissive,
                    device,
                    surface_config,
                    &global_bind_group_layout,
                    &mesh_bind_group_layout,
                    &asset_loader,
                ),
                untextured_emissive: MeshPipeline::shaded(
                    MaterialShading::UntexturedEmissive,
                    device,
                    surface_config,
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
            pipelines: std::rc::Rc::new(pipelines),
            mesh_bind_group_layout: std::rc::Rc::new(mesh_bind_group_layout),
        };

        Ok(mesh_pass)
    }
}

impl Pass for MeshPass {
    fn resize(
        &mut self,
        _graphics: &GraphicsShared,
        surface_config: &wgpu::SurfaceConfiguration,
        world: &mut legion::World,
        _resources: &mut legion::Resources,
    ) -> Result<()> {
        // Update every camera's aspect ratio
        let mut query = <&mut Camera>::query();
        query.for_each_mut(world, |camera| {
            camera.update_aspect(surface_config.width as f32 / surface_config.height as f32);
        });
        Ok(())
    }
    fn render(
        &mut self,
        graphics: &GraphicsShared,
        encoder: &mut wgpu::CommandEncoder,
        // Usually the frame
        target_view: &mut wgpu::TextureView,
        depth_texture_view: &wgpu::TextureView,
        world: &legion::World,
        resources: &legion::Resources,
    ) {
        // TODO: Deduplicate code (see DebugPass)
        let lerp = resources
            .get::<PhysicsTimer>()
            .map(|t| t.lerp() as f32)
            .unwrap_or(1.0);

        // Upload global uniforms
        if let Some(main_cam) = resources.get::<MainCamera>() {
            let cam_pos = main_cam.position.current(lerp);
            log::debug!("cam_pos: {:?}", cam_pos);
            let _view_proj = main_cam.camera.projection()
                * main_cam.camera.view(
                    cam_pos.translation.vector.into(),
                    cam_pos.rotation.euler_angles().2.to_degrees(),
                    cam_pos.rotation.euler_angles().1.to_degrees(),
                );
            let view_proj = main_cam.camera.projection() * main_cam.camera.view2(&cam_pos);
            let global_uniforms = GlobalUniforms {
                view_proj: view_proj.into(),
                camera_pos: cam_pos.translation.vector.into(),
                _padding: [0.0; 9],
            };
            graphics.queue.write_buffer(
                &self.global_uniform_buf,
                0,
                bytemuck::bytes_of(&global_uniforms),
            );
        } else {
            // No camera present; can't render
            return;
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
            graphics
                .queue
                .write_buffer(&rmesh.uniform_buf, 0, bytemuck::bytes_of(&transform));
        }

        // Begin rendering

        // Render every mesh
        encoder.push_debug_group("forward rendering pass");
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                // Clear the frame
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: target_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Clear the framebuffer with a color
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                }],
                // Clear the depth buffer
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
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
                    render_pass
                        .set_index_buffer(part.index_buf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.set_vertex_buffer(0, part.vertex_buf.slice(..));
                    render_pass.draw_indexed(0..part.index_count as u32, 0, 0..1);
                }
            }
        }
        encoder.pop_debug_group();
    }
}
