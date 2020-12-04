use anyhow::Result;
use legion::{World, Resources};

pub trait Pass {
    fn render(
        &mut self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut wgpu::SwapChainTexture,
        _world: &World,
        _resources: &Resources,
    ) -> Result<()>;
    fn resize(
        &mut self,
        device: &mut wgpu::Device,
        queue: &mut wgpu::Queue,
        sc_desc: &mut wgpu::SwapChainDescriptor,
        _world: &World,
        _resources: &Resources,
    ) -> Result<()>;
}