mod material;
mod pipeline;
mod render_mesh;
pub use render_mesh::{RenderMesh, RenderMeshLayouts, RenderMeshPart};
mod pass;
pub use pass::MeshPass;

// Alignment table: https://gpuweb.github.io/gpuweb/wgsl/#alignment-and-size

use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
pub struct Vertex {
    // Alignment 16
    pub pos: [f32; 3],
    // Alignment 16
    pub normal: [f32; 3],
    // Alignment 8
    pub uv: [f32; 2],
    // Pad to 64
    pub _padding: [f32; 6],
}

impl Vertex {
    pub const fn vertex_attrs() -> [wgpu::VertexAttribute; 3] {
        wgpu::vertex_attr_array![
            // Position (alignment 16)
            0 => Float32x3,
            // Normal (alignment 16)
            1 => Float32x3,
            // UV (alignment 8)
            2 => Float32x2,
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct MeshUniforms {
    pub(crate) model: [[f32; 4]; 4],
}
