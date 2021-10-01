use std::rc::Rc;

use wgpu::util::DeviceExt;

use crate::{assets::data::*, graphics::Graphics};

use super::{material::*, pass::MeshPassPipelines, MeshUniforms};

#[derive(Clone)]
pub struct MeshLayouts {
    pub mesh: Rc<wgpu::BindGroupLayout>,
    pub material: Rc<MeshPassPipelines>,
}

pub struct RenderMeshPart {
    pub material: MeshMaterial,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
}

impl RenderMeshPart {
    pub fn new(
        data: MeshPartData,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        layouts: &MeshLayouts,
    ) -> Self {
        let is_emissive = data.material.color_emissive != [0.0, 0.0, 0.0].into();
        let is_lit = data.material.lighting;
        let is_textured = data.material.diffuse_map.is_some();

        let shading = match (is_lit, is_textured, is_emissive) {
            (false, false, false) => MaterialShading::UntexturedUnlit,
            (false, true, false) => MaterialShading::TexturedUnlit,
            (true, false, false) => MaterialShading::Untextured,
            (true, true, false) => MaterialShading::Textured,
            (true, false, true) => MaterialShading::UntexturedEmissive,
            (true, true, true) => MaterialShading::TexturedEmissive,
            (false, true, true) => panic!("Unsupported shading: unlit textured emissive"),
            (false, false, true) => panic!("Unsupported shading: unlit untextured emissive"),
        };

        let material = MeshMaterial::new(
            shading,
            MaterialFactors {
                diffuse: data
                    .material
                    .color_diffuse
                    .alpha(data.material.alpha)
                    .into(),
                emissive: data.material.color_emissive.into(),
                _padding: [0.0],
            },
            data.material
                .diffuse_map
                .map(|tex| Graphics::upload_texture(device, encoder, false, tex)),
            device,
            layouts,
        )
        .unwrap();

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(&data.indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        RenderMeshPart {
            material,
            vertex_buf,
            index_buf,
            index_count: data.indices.len() as u32,
        }
    }
}

pub struct RenderMesh {
    pub parts: Vec<RenderMeshPart>,

    pub uniform_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl RenderMesh {
    pub fn from_parts(
        parts: Vec<MeshPartData>,
        layouts: &MeshLayouts,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
    ) -> RenderMesh {
        let model_uniform = MeshUniforms {
            model: na::Matrix4::identity().into(),
        };

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&model_uniform),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layouts.mesh,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniform_buf,
                    offset: 0,
                    // FIXME
                    size: None,
                }),
            }],
        });

        let mut render_parts = Vec::with_capacity(parts.len());
        for part_data in parts {
            render_parts.push(RenderMeshPart::new(part_data, device, encoder, layouts))
        }

        RenderMesh {
            parts: render_parts,
            bind_group,
            uniform_buf,
        }
    }
}
