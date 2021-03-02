use bytemuck::{Pod, Zeroable};
use eyre::{eyre::eyre, Result};
use wgpu::util::DeviceExt;

#[derive(Debug)]
pub enum MaterialShading {
    Untextured,
    Textured,
    UntexturedUnlit,
    TexturedUnlit,
    TexturedEmissive,
    UntexturedEmissive,
}

use MaterialShading::*;

use super::render_mesh::MeshLayouts;

impl MaterialShading {
    pub fn _is_lit(&self) -> bool {
        matches!(self, UntexturedUnlit | TexturedUnlit)
    }
    pub fn is_textured(&self) -> bool {
        match self {
            Textured | TexturedUnlit | TexturedEmissive => true,
            Untextured | UntexturedUnlit | UntexturedEmissive => false,
        }
    }
    pub fn _is_emissive(&self) -> bool {
        matches!(self, TexturedEmissive | UntexturedEmissive)
    }
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
    pub _factors_buf: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

impl MeshMaterial {
    pub fn new(
        shading: MaterialShading,
        factors: MaterialFactors,
        texture: Option<wgpu::Texture>,
        device: &wgpu::Device,
        layouts: &MeshLayouts,
    ) -> Result<Self> {
        let factors_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: None,
            contents: bytemuck::bytes_of(&factors),
            usage: wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        });

        let bind_group = if !shading.is_textured() {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layouts.material.untextured.part_bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer {
                        buffer: &factors_buf,
                        offset: 0,
                        // FIXME
                        size: None,
                    },
                }],
            })
        } else {
            let texture_view = texture
                .ok_or_else(|| eyre!("Cannot create a textured material without a texture"))?
                .create_view(&wgpu::TextureViewDescriptor::default());

            let sampler = device.create_sampler(&wgpu::SamplerDescriptor::default());

            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &layouts.material.textured.part_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer {
                            buffer: &factors_buf,
                            offset: 0,
                            // FIXME
                            size: None,
                        },
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                ],
            })
        };

        Ok(MeshMaterial {
            factors,
            shading,
            _factors_buf: factors_buf,
            bind_group,
        })
    }
}
