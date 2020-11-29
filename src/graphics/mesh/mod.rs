mod model;
pub use model::Model;
pub mod mesh_pass;
pub use mesh_pass::MeshPass;

use bytemuck::{Pod, Zeroable};
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)] 
pub struct Vertex {
    pub pos: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GlobalUniforms {
    pub(crate)view_proj: [[f32; 4]; 4],
    pub(crate)camera_pos: [f32; 3],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ModelUniforms {
    pub(crate)model: [[f32; 4]; 4],
}