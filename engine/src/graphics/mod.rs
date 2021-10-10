use std::{num::NonZeroU32, rc::Rc};

use egui_wgpu_backend::ScreenDescriptor;
use eyre::{eyre::WrapErr, Result};
use legion::{Resources, World};
use winit::dpi::PhysicalSize;

use wgpu::util::DeviceExt;

mod setup;
pub use setup::setup;

mod camera;
pub use camera::*;

pub mod color;

mod pass;
pub use pass::Pass;

pub mod debug;
pub mod mesh;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    // Alignment 16, size 64
    pub(crate) view_proj: [[f32; 4]; 4],
    // Alignment 16, size 12 (16 taken)
    pub(crate) camera_pos: [f32; 3],
    // Pad to 128
    pub _padding: [f32; 3 + 6],
}

//pub const COMPILED_SHADERS_DIR: &str = "shaders/compiled/";
//pub const COMPILED_VERTEX_SHADER_EXT: &str = ".vert.spv";
//pub const COMPILED_FRAGMENT_SHADER_EXT: &str = ".frag.spv";
pub const WGSL_SHADERS_DIR: &str = "shaders/wgsl/";
pub const WGSL_SHADERS_EXT: &str = ".wgsl";

// It's all pointers either way
#[derive(Clone)]
pub struct GraphicsShared {
    pub device: Rc<wgpu::Device>,
    pub queue: Rc<wgpu::Queue>,
    pub window: Rc<winit::window::Window>,
    pub mesh_layouts: mesh::RenderMeshLayouts,
}

pub struct Graphics {
    pub device: Rc<wgpu::Device>,
    pub queue: Rc<wgpu::Queue>,
    pub window: Rc<winit::window::Window>,

    pub mesh_pass: mesh::MeshPass,
    pub debug_pass: Option<debug::DebugPass>,

    pub surface_config: wgpu::SurfaceConfiguration,
    pub surface: wgpu::Surface,

    depth_texture: wgpu::Texture,
    depth_texture_view: wgpu::TextureView,

    pub shared: GraphicsShared,
}

impl Graphics {
    pub fn prepare(&mut self, _resources: &mut Resources) {}

    pub fn resize(
        &mut self,
        size: PhysicalSize<u32>,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<()> {
        // Recreate the swap chain with the new size
        self.surface_config.width = size.width;
        self.surface_config.height = size.height;
        self.surface.configure(&self.device, &self.surface_config);

        // Resize the depth texture
        self.depth_texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth texture"),
            size: wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        });
        self.depth_texture_view = self
            .depth_texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // Tell all the render passes to resize their internal buffers
        self.mesh_pass
            .resize(&self.shared, &self.surface_config, world, resources)?;

        Ok(())
    }

    pub fn render(
        &mut self,
        world: &mut World,
        resources: &mut Resources,
        ui: Option<(Vec<epaint::ClippedMesh>, std::sync::Arc<epaint::Texture>)>,
    ) -> Result<()> {
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(_) => {
                log::debug!("Reconfiguring surface");
                self.surface.configure(&self.device, &self.surface_config);
                self.surface
                    .get_current_texture()
                    .wrap_err_with(|| "Failed to acquire next swapchain texture")?
            }
        };

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let mut surface_view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        // Render onto the frame with render passes

        {
            log::debug!("Clearing frame");
            // Clear the frame
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachment {
                    view: &surface_view,
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: false,
                    }),
                    stencil_ops: None,
                }),
            });
        }

        log::debug!("Rendering meshes");
        self.mesh_pass.render(
            &self.shared,
            &mut encoder,
            &mut surface_view,
            &self.depth_texture_view,
            world,
            resources,
        );
        // DebugPass needs the lerp value which is present only after the MeshPass is activated
        if let Some(debug_pass) = &mut self.debug_pass {
            debug_pass.render(
                &self.shared,
                &mut encoder,
                &mut surface_view,
                &self.depth_texture_view,
                world,
                resources,
            );
        }

        if let Some((triangles, texture)) = ui {
            log::debug!("Rendering ui");
            let mut egui_rpass = egui_wgpu_backend::RenderPass::new(
                &self.shared.device,
                self.surface_config.format,
                1,
            );
            egui_rpass.update_texture(&self.device, &self.queue, &texture);
            egui_rpass.update_user_textures(&self.device, &self.queue);
            let screen_desc = ScreenDescriptor {
                physical_width: self.surface_config.width,
                physical_height: self.surface_config.height,
                scale_factor: self.window.scale_factor() as f32,
            };
            egui_rpass.update_buffers(&self.device, &self.queue, &triangles[..], &screen_desc);

            egui_rpass
                .execute(
                    &mut encoder,
                    &surface_view,
                    &triangles[..],
                    &screen_desc,
                    None,
                )
                .wrap_err_with(|| "Failed to render UI")?;
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }

    pub fn upload_texture(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        srgb: bool,
        img: image::RgbaImage,
    ) -> wgpu::Texture {
        // The physical size of the texture
        let (img_width, img_height) = (img.width(), img.height());
        let texture_extent = wgpu::Extent3d {
            width: img_width,
            height: img_height,
            depth_or_array_layers: 1,
        };
        // The texture binding to copy data to and use as a handle to it
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if srgb {
                wgpu::TextureFormat::Rgba8UnormSrgb
            } else {
                wgpu::TextureFormat::Rgba8Unorm
            },
            usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
        });
        // Temporary buffer to copy data from into the texture
        let tmp_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&img.into_raw()),
            usage: wgpu::BufferUsages::COPY_SRC,
        });
        // Copy img's pixels from the temporary buffer into the texture buffer
        encoder.copy_buffer_to_texture(
            wgpu::ImageCopyBuffer {
                buffer: &tmp_buf,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: NonZeroU32::new(4 * img_width),
                    rows_per_image: NonZeroU32::new(img_height),
                },
            },
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            texture_extent,
        );
        // Return the texture handle
        texture
    }
}
