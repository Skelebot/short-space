use super::ModelUniforms;
use super::mesh_pass::MeshBindGroupLayout;

use crate::{graphics::Graphics, asset_loader::data::ModelData};
use wgpu::util::DeviceExt;

pub struct Model {
    // TODO: Rc<> for multiple models with the same data (it's all read-only either way)
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub uniform_buf: wgpu::Buffer,
    pub index_count: usize,
    pub uniform_offset: wgpu::DynamicOffset,
}

impl Model {
    pub fn from_data(data: ModelData, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder, bind_group_layout: &MeshBindGroupLayout) -> Model {

        let vertex_data = data.vertices;
        let index_data = data.indices;

        let vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&vertex_data),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );
        let index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&index_data),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

        let model_uniform = ModelUniforms {
            model: na::Matrix4::identity().into(),
        };
        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&model_uniform),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            compare: None,
            // TODO: Set filters to FilterMode::Linear for smoother textures
            ..Default::default()
        });

        let texture = Graphics::upload_texture(device, encoder, true, data.texture_img);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: None,
            // TODO: Review and customize
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout.0,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),   
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),   
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
            ]
        });

        Model {
            bind_group: bind_group,
            uniform_buf: uniform_buf,
            vertex_buf: vertex_buf,
            index_buf: index_buf,
            index_count: index_data.len(),
            uniform_offset: 0,
        }
    }
}