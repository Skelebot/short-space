mod render_mesh;
pub use render_mesh::RenderMesh;

pub mod pass;
pub use pass::MeshPass;

mod material;
mod pipeline;

use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

impl Vertex {
    pub fn vertex_attrs() -> [wgpu::VertexAttributeDescriptor; 3] {
        wgpu::vertex_attr_array![
            // Position
            0 => Float3,
            // Normal
            1 => Float3,
            // UV
            2 => Float2,
        ]
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub(crate) view_proj: [[f32; 4]; 4],
    pub(crate) camera_pos: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MeshUniforms {
    pub(crate) model: [[f32; 4]; 4],
}
