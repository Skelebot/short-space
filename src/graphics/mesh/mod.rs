mod mesh;
use anyhow::{Result, anyhow};
use wgpu::util::DeviceExt;

pub use mesh::RenderMesh;
pub mod mesh_pass;
pub use mesh_pass::MeshPass;
mod mesh_pipeline;

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
pub struct MeshUniforms {
    pub(crate)model: [[f32; 4]; 4],
}

#[derive(Debug)]
pub enum MaterialShading {
    Untextured,
    Textured,
    TexturedUnlit,
    TexturedEmissive,
    UntexturedEmissive,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct MaterialFactors {
    pub diffuse: [f32; 4],
    pub emissive: [f32; 3],
}

pub struct MeshMaterial {
    pub shading: MaterialShading,
    pub factors: MaterialFactors,

    // Even if we only set it once when initializing the material,
    // we have to store it so it doesn't get dropped
    _factors_buf: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
}

impl MeshMaterial {
    pub fn new(
        shading: MaterialShading,
        factors: MaterialFactors,
        texture: Option<wgpu::Texture>,
        device: &mut wgpu::Device,
        mesh_pass: &MeshPass,
    ) -> Result<Self> {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&factors),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = match shading {

            MaterialShading::Untextured | MaterialShading::UntexturedEmissive => {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &mesh_pass.pipelines.untextured.part_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(factors_buf.slice(..))
                        },
                    ]
                })
            },

            MaterialShading::Textured | MaterialShading::TexturedUnlit | MaterialShading::TexturedEmissive => {
                let texture_view = texture.ok_or(anyhow!("Cannot create a textured material without a texture"))?
                    .create_view(&wgpu::TextureViewDescriptor::default());

                // TODO: Review and customize
                // ?
                let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &mesh_pass.pipelines.textured.part_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(factors_buf.slice(..))
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::TextureView(&texture_view),
                        }
                    ]
                })
            },
        };

        Ok(MeshMaterial {
            factors,
            shading,
            _factors_buf: factors_buf,
            bind_group,
        })
    }
}
