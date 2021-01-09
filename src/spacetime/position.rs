/// A wrapper for nalgebra's Isometry to be used as a component for physical entities
#[derive(Debug, Copy, Clone)]
pub struct Position {
    past: na::Isometry3<f32>,
    future: na::Isometry3<f32>,
}

impl Position {
    pub fn current(&self, lerp: f32) -> na::Isometry3<f32> {
        let translation = na::Translation3::from(
            self.past
                .translation
                .vector
                .lerp(&self.future.translation.vector, lerp),
        );
        let rotation = self.past.rotation.slerp(&self.future.rotation, lerp);

        na::Isometry3::from_parts(translation, rotation)
    }
    pub fn past(&self) -> &na::Isometry3<f32> {
        &self.past
    }
    pub fn future(&self) -> &na::Isometry3<f32> {
        &self.future
    }
    pub fn past_mut(&mut self) -> &mut na::Isometry3<f32> {
        &mut self.past
    }
    pub fn future_mut(&mut self) -> &mut na::Isometry3<f32> {
        &mut self.future
    }
}

impl From<na::Isometry3<f32>> for Position {
    fn from(iso: na::Isometry3<f32>) -> Self {
        Position {
            past: iso,
            future: iso,
        }
    }
}

#[test]
fn test_position_lerp() {
    let mut pos = Position {
        past: na::Isometry3::translation(0.0, 2.0, 0.0),
        future: na::Isometry3::translation(4.0, 4.0, -3.0),
    };

    assert_eq!(pos.current(0.5), na::Isometry3::translation(2.0, 3.0, -1.5));

    pos.past_mut().rotation =
        na::UnitQuaternion::from_axis_angle(&na::Vector::z_axis(), 90.0_f32.to_radians());
    pos.future_mut().rotation =
        na::UnitQuaternion::from_axis_angle(&na::Vector::z_axis(), 0.0_f32.to_radians());

    use approx::RelativeEq;

    assert!(pos.current(0.5).rotation.relative_eq(
        &na::UnitQuaternion::from_axis_angle(&na::Vector::z_axis(), 45.0_f32.to_radians()),
        na::UnitQuaternion::default_max_relative(),
        na::UnitQuaternion::default_max_relative(),
    ))
}
