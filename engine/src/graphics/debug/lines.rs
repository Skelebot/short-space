#![allow(dead_code)]
use crate::graphics::color::Rgba;

pub struct DebugLines {
    pub thickness: f32,
    pub vec: Vec<super::Line>,
}

impl DebugLines {
    pub fn new() -> Self {
        DebugLines {
            thickness: 1.0,
            vec: Vec::new(),
        }
    }

    pub fn push_line(&mut self, a: na::Vector3<f32>, b: na::Vector3<f32>, color: Rgba) {
        self.vec.push(super::Line {
            pos_a: a.into(),
            color_a: color.into(),
            pos_b: b.into(),
            color_b: color.into(),
        })
    }

    pub fn push_line_gradient(
        &mut self,
        a: na::Vector3<f32>,
        b: na::Vector3<f32>,
        color_a: Rgba,
        color_b: Rgba,
    ) {
        self.vec.push(super::Line {
            pos_a: a.into(),
            color_a: color_a.into(),
            pos_b: b.into(),
            color_b: color_b.into(),
        })
    }
}

impl Default for DebugLines {
    fn default() -> Self {
        Self::new()
    }
}
