use std::num::NonZeroU32;

use bytemuck::{Pod, Zeroable};

use crate::graphics::color::Rgb;

use super::ShadowPass;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct LightUniforms {
    // EXTREMELY IMPORTANT: proj must be the first field
    pub proj: [[f32; 4]; 4],
    // Fields are aligned to a vec4
    pub pos: [f32; 3],
    pub pad1: f32,
    pub color: [f32; 3],
    pub pad2: f32,
}
#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct ShadowUniforms {
    pub proj: [[f32; 4]; 4],
}

pub struct Light {
    // ?
    pub proj: na::Orthographic3<f32>,
    pub color: Rgb,
    pub target_view: wgpu::TextureView,
}

impl Light {
    pub fn new(proj: na::Orthographic3<f32>, color: Rgb, shadow_pass: &mut ShadowPass) -> Self {
        let shadow_target_view =
            shadow_pass
                .shadow_texture
                .create_view(&wgpu::TextureViewDescriptor {
                    label: None,
                    format: None,
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    aspect: wgpu::TextureAspect::All,
                    base_mip_level: 0,
                    level_count: None,
                    base_array_layer: (shadow_pass.light_count + 1) as u32,
                    array_layer_count: NonZeroU32::new(1),
                });

        shadow_pass.light_count += 1;

        Self {
            proj,
            color,
            target_view: shadow_target_view,
        }
    }
    pub fn into_raw(&self, position: &crate::physics::Position) -> LightUniforms {
        let eye: na::Point3<f32> = position.translation.vector.into();
        let target: na::Point3<f32> = position * na::Point3::new(0.0, 0.0, -1.0);
        let up = position * na::Vector3::new(0.0, 0.0, 1.0);

        //let view = na::Matrix::look_at_lh(&eye, &target, &up);
        //let view_proj = self.proj.into_inner() * view;
        use cgmath::{Deg, EuclideanSpace, Matrix4, PerspectiveFov, Point3, Vector3, Ortho};

        let t = Point3::new(
            0.0,
            0.0,
            -1.0
        );
        let mx_view = Matrix4::look_at(ctoa(eye), t, brrr(up));
        let projection = Ortho {
            left: -30.0,
            right: 30.0,
            bottom: -30.0,
            top: 30.0,
            near: -30.0,
            far: 30.0,
        };
        let mx_view_proj = cgmath::Matrix4::from(projection) * mx_view;

        LightUniforms {
            //proj: view_proj.into(),
            proj: *mx_view_proj.as_ref(),
            pos: position.translation.vector.into(),
            pad1: 1.0,
            color: self.color.into(),
            pad2: 1.0,
        }
    }
}

fn ctoa(a: na::Point3<f32>) -> cgmath::Point3<f32> {
    cgmath::Point3::new(
        a.x,
        a.y,
        a.z,
    )
}

fn brrr(a: na::Vector3<f32>) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(
        a.x,
        a.y,
        a.z,
    )
}