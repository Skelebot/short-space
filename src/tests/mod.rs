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

// Rust_analyzer complains about these imports being unused
#[allow(unused_imports)]
use winit::event::{VirtualKeyCode::*, ElementState::*};
#[allow(unused_imports)]
use crate::input::{InputState, Axis, Action};
#[test]
fn test_keypress_simple() {
    let mut input_state = InputState::default();

    assert!(!input_state.is_key_pressed(&A));
    input_state.handle_key_event(&A, &Pressed);
    assert!(input_state.is_key_pressed(&A));
    input_state.handle_key_event(&A, &Released);
    assert!(!input_state.is_key_pressed(&A));
}

#[test]
fn test_keypress_bounds() {
    let mut input_state = InputState::default();

    assert!(!input_state.is_key_pressed(&Key1));
    input_state.handle_key_event(&Key1, &Pressed);
    assert!(input_state.is_key_pressed(&Key1));
    input_state.handle_key_event(&Key1, &Released);
    assert!(!input_state.is_key_pressed(&Key1));

    assert!(!input_state.is_key_pressed(&Key1));
    input_state.handle_key_event(&Cut, &Pressed);
    assert!(input_state.is_key_pressed(&Cut));
    input_state.handle_key_event(&Cut, &Released);
    assert!(!input_state.is_key_pressed(&Cut));
}

#[test]
fn test_keyboard_axis() {
    let mut input_state = InputState::default();
    let axis = Axis::KeyboardAxis(NextTrack, PrevTrack);

    assert_eq!(input_state.get_axis_state(&axis), 0.0);
    input_state.handle_key_event(&NextTrack, &Pressed);
    assert_eq!(input_state.get_axis_state(&axis), 1.0);
    input_state.handle_key_event(&PrevTrack, &Pressed);
    assert_eq!(input_state.get_axis_state(&axis), 0.0);
    input_state.handle_key_event(&NextTrack, &Released);
    assert_eq!(input_state.get_axis_state(&axis), -1.0);
    input_state.handle_key_event(&PrevTrack, &Released);
    assert_eq!(input_state.get_axis_state(&axis), 0.0);
}

#[test]
fn test_keyboard_action() {
    let mut input_state = InputState::default();
    let action = Action::KeyboardAction(Power);

    assert!(!input_state.is_action_pressed(&action));
    input_state.handle_key_event(&Power, &Pressed);
    assert!(input_state.is_action_pressed(&action));
    input_state.handle_key_event(&Power, &Released);
    assert!(!input_state.is_action_pressed(&action));
}
