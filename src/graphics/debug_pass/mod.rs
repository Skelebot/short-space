use bytemuck::{Pod, Zeroable};

mod pass;
pub use pass::DebugPass;

mod lines;
pub use lines::DebugLines;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
pub struct Vertex {
    pub pos_a: [f32; 3],
    pub color_a: [f32; 4],
    pub pos_b: [f32; 3],
    pub color_b: [f32; 4],
}

impl Vertex {
    pub const fn vertex_attrs() -> [wgpu::VertexAttribute; 4] {
        wgpu::vertex_attr_array![
            0 => Float32x2,
            1 => Float32x4,
            2 => Float32x3,
            3 => Float32x4,
        ]
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable, PartialEq)]
pub struct DebugLinesUniforms {
    screen_thickness: [f32; 2],
}
