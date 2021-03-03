#![allow(clippy::float_cmp)]

use super::*;

//------------------------------
// Camera
//------------------------------

#[test]
fn camera_test() {
    let aspect = 800.0 / 600.0;
    let fov = 45.0;
    let znear = 0.1;
    let zfar = 100.0;

    let camera = graphics::Camera::new(aspect, fov, znear, zfar);

    let cam_proj = camera.projection();
    let proj = na::Perspective3::new(aspect, fov, znear, zfar);

    assert_eq!(cam_proj, proj.into_inner());

    let pos = na::Isometry3::from_parts(
        na::Translation3::from(na::Vector3::new(1.0, 3.0, 2.0)),
        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 90.0_f32.to_radians()),
    );

    let cam_view = camera.view(&pos);
    let view = {
        let position: na::Point3<f32> = pos.translation.vector.into();
        let target = pos * na::Point3::new(0.0, 1.0, 0.0);
        let up = pos * na::Vector3::z();
        na::Matrix::look_at_rh(&position, &target, &up)
    };

    assert_eq!(cam_view, view);
}

//------------------------------
// Input
//------------------------------

use crate::input::{Action, Axis, InputState};
use winit::event::{ElementState::*, VirtualKeyCode::*};
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

//------------------------------
// Colors
//------------------------------
use crate::graphics::color::*;
#[test]
fn test_rgb() {
    let a = Rgb::new(1.0_f32, 2.0, 3.0);
    let b = Rgb::from([1.0_f32, 2.0, 3.0]);
    assert!(a == b);

    let c: [f32; 3] = a.into();
    assert!(c == [1.0, 2.0, 3.0]);
}

#[test]
fn test_rgba() {
    let a = Rgba::new(1.0_f32, 2.0, 3.0, 4.0);
    let b = Rgba::from([1.0_f32, 2.0, 3.0, 4.0]);
    assert!(a == b);

    let c: [f32; 4] = a.into();
    assert!(c == [1.0, 2.0, 3.0, 4.0]);
}

#[test]
fn test_conversion() {
    let a = Rgb::new(1.0_f32, 2.0, 3.0);
    let b = Rgba::new(1.0, 2.0, 3.0, 4.0);
    let a_rgba = a.alpha(4.0);
    let b_rgb = b.rgb();

    assert!(a_rgba == b);
    assert!(b_rgb == a);
}

#[test]
fn test_physics_timer() {
    let mut t = crate::spacetime::PhysicsTimer::new(2.0);

    t.update(1.0);

    assert_eq!(t.lerp(), 0.5);
    assert_eq!(t.steps_due(), 0);

    t.update(2.0);

    assert_eq!(t.lerp(), 0.5);
    assert_eq!(t.steps_due(), 1);

    t.update(5.0);

    assert_eq!(t.lerp(), 0.0);
    assert_eq!(t.steps_due(), 3);
}
