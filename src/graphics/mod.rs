
use anyhow::{Result, Error};
use legion::{World, Resources};
use winit::dpi::PhysicalSize;

use wgpu::util::DeviceExt;

mod setup;
pub use setup::setup;

pub mod mesh;
mod pass;
pub use pass::Pass;
mod camera;
pub use camera::Camera;

pub struct Graphics {
    pub device: wgpu::Device,
    pub swap_chain: wgpu::SwapChain,
    pub sc_desc: wgpu::SwapChainDescriptor,
    pub surface: wgpu::Surface,
    pub queue: wgpu::Queue,
    pub window: winit::window::Window,
    pub render_passes: Vec<Box<dyn Pass>>,
}

impl Graphics {
    pub fn resize(&mut self, size: PhysicalSize<u32>, world: &mut World, resources: &mut Resources) -> Result<()> {
        // Recreate the swap chain with the new size
        self.sc_desc.width = size.width;
        self.sc_desc.height = size.height;
        self.swap_chain = self.device.create_swap_chain(&self.surface, &self.sc_desc);
        for pass in self.render_passes.iter_mut() {
            pass.resize(
                &mut self.device,
                &mut self.queue,
                &self.sc_desc,
                &world,
                &resources,
            )?;
        }

        Ok(())
    }

    pub fn render(&mut self, world: &mut World, resources: &mut Resources) -> Result<()> {
        let mut frame = self.swap_chain
            .get_current_frame()
            .map_err(|err| 
                Error::msg("Failed to acquire next swap chain texture")
                    .context(err)
            )?
            .output;

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });                    

        for pass in self.render_passes.iter_mut() {
            pass.render(&mut encoder, &mut self.queue, &mut frame, &world, &resources)?;
        }

        self.queue.submit(Some(encoder.finish()));
        Ok(())
    }
    
    pub fn upload_texture(
        device: &mut wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        srgb: bool,
        img: image::RgbaImage,
    ) -> wgpu::Texture {
        // The physical size of the texture
        let (img_width, img_height) = (img.width(), img.height());
        let texture_extent = wgpu::Extent3d {
            width: img_width,
            height: img_height,
            depth: 1,
        };
        // The texture binding to copy data to and use as a handle to it
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: if srgb { wgpu::TextureFormat::Rgba8UnormSrgb } else { wgpu::TextureFormat::Rgba8Unorm },
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        // Temporary buffer to copy data from into the texture
        let tmp_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&img.into_raw()),
                usage: wgpu::BufferUsage::COPY_SRC,
            }
        );
        // Copy img's pixels from the temporary buffer into the texture buffer
        encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &tmp_buf,
                layout: wgpu::TextureDataLayout {
                    offset: 0,
                    bytes_per_row: 4 * img_width,
                    rows_per_image: img_height,
                },
            }, 
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
            }, 
            texture_extent
        );
        // Return the texture handle
        texture
    }
}