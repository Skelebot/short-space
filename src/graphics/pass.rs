use eyre::Result;
use legion::{Resources, World};

use super::GraphicsShared;

pub trait Pass {
    fn resize(
        &mut self,
        graphics: &GraphicsShared,
        surface_config: &wgpu::SurfaceConfiguration,
        world: &mut World,
        resources: &mut Resources,
    ) -> Result<()>;
    fn render(
        &mut self,
        graphics: &GraphicsShared,
        encoder: &mut wgpu::CommandEncoder,
        target_view: &mut wgpu::TextureView,
        world: &World,
        resources: &Resources,
        depth_texture_view: &wgpu::TextureView,
    );
}
