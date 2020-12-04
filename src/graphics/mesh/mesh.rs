use wgpu::util::DeviceExt;

use crate::{graphics::Graphics, asset_loader::data::*};

use super::{MaterialFactors, MaterialShading, MeshMaterial, MeshPass, MeshUniforms};

pub struct RenderMeshPart {
    pub material: MeshMaterial,
    pub vertex_buf: wgpu::Buffer,
    pub index_buf: wgpu::Buffer,
    pub index_count: u32,
}

impl RenderMeshPart {
    pub fn new(data: MeshPartData, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder, mesh_pass: &MeshPass) -> Self {
        
        let textured = data.material.diffuse_map.is_some();
        let shading = match data.material.lighting {
            true => match data.material.color_emissive {
                // See https://github.com/rust-lang/rust/issues/36890
                x if x == [0.0, 0.0, 0.0] => if textured { MaterialShading::Textured } else { MaterialShading::Untextured },
                _ => if textured { MaterialShading::TexturedEmissive } else { MaterialShading::UntexturedEmissive },
            },
            false => match data.material.color_emissive {
                x if x == [0.0, 0.0, 0.0] => if textured { MaterialShading::TexturedUnlit } else { todo!("Untextured unlit") },
                _ => if textured { todo!("Textured unlit emissive") } else { todo!("Untextured unlit emissive") },
            },
        };

        log::debug!("Creating mesh part with shading: {:?}", shading);

        let material = MeshMaterial::new(
            shading,
            MaterialFactors {
                diffuse: [
                    data.material.color_diffuse[0],
                    data.material.color_diffuse[1],
                    data.material.color_diffuse[2],
                    data.material.alpha
                ],
                emissive: data.material.color_emissive,
            },
            data.material.diffuse_map.map(|tex| Graphics::upload_texture(device, encoder, false, tex)),
            device,
            mesh_pass,
        ).unwrap();

        let vertex_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&data.vertices),
                usage: wgpu::BufferUsage::VERTEX,
            }
        );
        let index_buf = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&data.indices),
                usage: wgpu::BufferUsage::INDEX,
            }
        );

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
    pub fn from_parts(parts: Vec<MeshPartData>, mesh_pass: &MeshPass, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> RenderMesh {

        let model_uniform = MeshUniforms {
            model: na::Matrix4::identity().into(),
        };

        let uniform_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&model_uniform),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &mesh_pass.mesh_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),   
                },
            ]
        });

        let mut render_parts = Vec::with_capacity(parts.len());
        for part_data in parts {
            render_parts.push(RenderMeshPart::new(
                part_data,
                device,
                encoder,
                mesh_pass,
            ))
        };

        RenderMesh {
            parts: render_parts,
            bind_group,
            uniform_buf,
        }
    }
}