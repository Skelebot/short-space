use super::*;

#[test]
fn camera_test() {
    let aspect = 800.0/600.0;
    let fov = 45.0;
    let znear = 0.1;
    let zfar = 100.0;

    let mut camera = graphics::Camera::new(aspect, fov, znear, zfar);

    let cam_proj = camera.get_projection_matrix();
    let proj = na::Perspective3::new(aspect, fov, znear, zfar);

    assert_eq!(cam_proj, proj.into_inner());

    let pos = na::Isometry3::from_parts(
        na::Translation3::from(
            na::Vector3::new(1.0, 3.0, 2.0)
        ),
        na::UnitQuaternion::from_axis_angle(
            &na::Vector3::z_axis(),
            90.0_f32.to_radians(),
        )
    );
    camera.position = pos;

    assert_eq!(camera.position, pos);

    let cam_view = camera.get_view_matrix();
    let view = {
        let position: na::Point3<f32> = 
            pos.translation.vector.into();
        let target = pos * na::Point3::new(0.0, 1.0, 0.0);
        let up = pos * na::Vector3::z();
        na::Matrix::look_at_rh(&position, &target, &up)
    };

    assert_eq!(cam_view, view);
}
