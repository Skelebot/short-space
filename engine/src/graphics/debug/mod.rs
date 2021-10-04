mod lines;
mod pass;
pub use lines::DebugLines;
pub use pass::DebugPass;

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, PartialEq, Debug)]
/// Could be called simply "Vertex" because that's exactly the data that is sent to the GPU in a vertex buffer
pub struct Line {
    pub pos_a: [f32; 3],
    pub color_a: [f32; 4],
    pub pos_b: [f32; 3],
    pub color_b: [f32; 4],
}

impl Line {
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
struct DebugLinesUniforms {
    screen_thickness: [f32; 2],
}
