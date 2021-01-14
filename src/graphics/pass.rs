use eyre::Result;
use legion::{Resources, World};

use super::GraphicsShared;

pub trait Pass {
    fn resize(
        &mut self,
        graphics: &GraphicsShared,
        sc_desc: &wgpu::SwapChainDescriptor,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<()>;
    fn render(
        &mut self,
        graphics: &GraphicsShared,
        encoder: &mut wgpu::CommandEncoder,
        target: &mut wgpu::SwapChainTexture,
        world: &World,
        resources: &Resources,
    );
}
