use anyhow::Result;
use legion::{World, Resources};

pub trait Pass {
    //fn setup(
    //    &mut self,
    //    device: &mut wgpu::Device,
    //    size: &winit::dpi::PhysicalSize<u32>,
    //    sc_desc: &wgpu::SwapChainDescriptor,
    //    _world: &mut World,
    //    _resources: &mut Resources,
    //) -> Result<()>;
    fn render(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        queue: &mut wgpu::Queue,
        target: &mut wgpu::SwapChainTexture,
        _world: &World,
        _resources: &Resources,
    ) -> Result<()>;
}