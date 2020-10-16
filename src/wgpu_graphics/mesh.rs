#[macro_use]
use bytemuck::{Pod, Zeroable};

use wgpu::util::DeviceExt;

pub struct MeshPass {
    pub pipeline: wgpu::RenderPipeline,
    pub mesh_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group_layout: wgpu::BindGroupLayout,
    pub global_bind_group: wgpu::BindGroup,
    pub global_uniform_buf: wgpu::Buffer,
}
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)] 
pub struct Vertex {
    pos: [f32; 3],
    normal: [f32; 3],
}

pub fn vertex(pos: [i16; 3], nor: [i16; 3]) -> Vertex {
    Vertex {
        pos: [pos[0].into(), pos[1].into(), pos[2].into()],
        normal: [nor[0].into(), nor[1].into(), nor[2].into()],
    }
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

pub struct Model {
    pub world: na::Matrix4<f32>,
    // TODO: Rc
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
    pub index_count: usize,
    pub uniform_offset: wgpu::DynamicOffset,
}

pub struct ModelData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

impl Model {
    pub fn from_data(data: ModelData, device: &mut wgpu::Device, pass: &MeshPass) -> Model {

        let vertex_data = data.vertices;
        let index_data = data.indices;

        let vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );
        let index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Cube Index Buffer"),
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsage::INDEX,
            }
        );
        let model_uniform = ModelUniforms {
            model: na::Matrix4::identity().into(),
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Cube uniform buffer"),
            contents: bytemuck::bytes_of(&model_uniform),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pass.mesh_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),   
                }
            ]
        });

        let mut cube_pos = na::Isometry3::<f32>::identity();

        Model {
            world: na::Matrix4::identity(),
            bind_group: bind_group,
            uniform_buf: uniform_buf,
            vertex_buf: vertex_buf,
            index_buf: index_buf,
            index_count: index_data.len(),
            uniform_offset: 0,
        }
    }
}

pub fn create_cube() -> ModelData {
    let vertex_data = [
        // top (0, 0, 1)
        vertex([-1, -1, 1], [0, 0, 1]),
        vertex([1, -1, 1], [0, 0, 1]),
        vertex([1, 1, 1], [0, 0, 1]),
        vertex([-1, 1, 1], [0, 0, 1]),
        // bottom (0, 0, -1)
        vertex([-1, 1, -1], [0, 0, -1]),
        vertex([1, 1, -1], [0, 0, -1]),
        vertex([1, -1, -1], [0, 0, -1]),
        vertex([-1, -1, -1], [0, 0, -1]),
        // right (1, 0, 0)
        vertex([1, -1, -1], [1, 0, 0]),
        vertex([1, 1, -1], [1, 0, 0]),
        vertex([1, 1, 1], [1, 0, 0]),
        vertex([1, -1, 1], [1, 0, 0]),
        // left (-1, 0, 0)
        vertex([-1, -1, 1], [-1, 0, 0]),
        vertex([-1, 1, 1], [-1, 0, 0]),
        vertex([-1, 1, -1], [-1, 0, 0]),
        vertex([-1, -1, -1], [-1, 0, 0]),
        // front (0, 1, 0)
        vertex([1, 1, -1], [0, 1, 0]),
        vertex([-1, 1, -1], [0, 1, 0]),
        vertex([-1, 1, 1], [0, 1, 0]),
        vertex([1, 1, 1], [0, 1, 0]),
        // back (0, -1, 0)
        vertex([1, -1, 1], [0, -1, 0]),
        vertex([-1, -1, 1], [0, -1, 0]),
        vertex([-1, -1, -1], [0, -1, 0]),
        vertex([1, -1, -1], [0, -1, 0]),
    ];

    let index_data: &[u16] = &[
        0, 1, 2, 2, 3, 0, // top
        4, 5, 6, 6, 7, 4, // bottom
        8, 9, 10, 10, 11, 8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    ModelData {
        vertices: vertex_data.to_vec(),
        indices: index_data.to_vec(),
    }
}

pub fn create_plane(size: i16) -> ModelData {
    let vertex_data = [
        vertex([size, -size, 0], [0, 0, 1]),
        vertex([size, size, 0], [0, 0, 1]),
        vertex([-size, -size, 0], [0, 0, 1]),
        vertex([-size, size, 0], [0, 0, 1]),
    ];

    let index_data: &[u16] = &[0, 1, 2, 2, 1, 3];

    ModelData {
        vertices: vertex_data.to_vec(),
        indices: index_data.to_vec(),
    }
}