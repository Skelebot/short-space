use eyre::Result;
use legion::{Resources, World};

pub trait Pass {
    fn resize(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        sc_desc: &mut wgpu::SwapChainDescriptor,
        _world: &mut World,
    ) -> Result<()>;
    fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut wgpu::SwapChainTexture,
        _world: &World,
        _resources: &Resources,
    ) -> Result<()>;
}
