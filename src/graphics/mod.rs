
use anyhow::{Result, Error};
use legion::{World, Resources};
use winit::dpi::PhysicalSize;

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
}
