use anyhow::Result;
use legion::{Resources, World};

pub trait Pass {
    fn resize(
        &mut self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        sc_desc: &mut wgpu::SwapChainDescriptor,
        _world: &mut World,
        _resources: &Resources,
    ) -> Result<()>;
    fn render(
        &mut self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut wgpu::SwapChainTexture,
        _world: &World,
        _resources: &Resources,
    ) -> Result<()>;
}
