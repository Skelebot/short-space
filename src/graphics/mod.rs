use nalgebra as na;
use anyhow::{Result, Error};
use legion::{World, Resources, system};

use crate::settings::GameSettings;

mod buffer;
mod color_buffer;
mod data;
mod model;
pub use model::Model;
pub mod shader;
mod texture;
pub use texture::Texture;
mod viewport;
pub use viewport::Viewport;
mod camera;
pub use camera::Camera;
// TODO: What's wrong with it?
//mod light;

// Notes on the graphics engine
// 1. OpenGL uses the right-hand coordinate space (x right, y up, -z forward),
// But Blender and everything else uses Z as height, which makes much more sense.
// By applying a few rotations on the Model matrixes before sending them to shaders
// and a few tricks in the look-at view matrix generation we can use the OpenGL's
// coordinate system with ours (x forward, y left, z up)

/// Setup the window and the OpenGL context and add all the necessary resources to the ECS
pub fn setup_window(_world: &mut World, resources: &mut Resources) -> Result<()> {

    let settings = (resources.get::<GameSettings>().ok_or(Error::msg("Failed to get Settings")))?.clone();

    // All sdl2 functions return std::result::Result<{some error}, String> so we can turn them
    // into anyhow::Result<{some error}, anyhow::Error> by wrapping the String in an Error.
    let sdl = sdl2::init().map_err(|e| Error::msg(e))?;
    let video_subsystem = sdl.video().map_err(|e| Error::msg(e))?;
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("short-space", settings.window_width as u32, settings.window_height as u32)
        .opengl()
        .resizable()
        .build()?;

    let gl_context = window.gl_create_context().map_err(|e| Error::msg(e))?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    if settings.vsync {
        video_subsystem.gl_set_swap_interval(1).unwrap();
    } else {
        video_subsystem.gl_set_swap_interval(0).unwrap();
    }

    let viewport = viewport::Viewport::for_window(settings.window_width, settings.window_height);
    viewport.set_used(&gl);
    let color_buffer = color_buffer::ColorBuffer::from_color(na::Vector3::new(0.3, 0.3, 0.3));
    color_buffer.set_used(&gl);
    unsafe {
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    }

    let event_pump = sdl.event_pump().map_err(|e| Error::msg(e))?;

    // Set up the camera
    let camera = Camera::new(viewport.get_aspect(), 3.14/2.0, 0.01, 1000.0); 
    resources.insert(camera);

    // OpenGL config
    unsafe {
        // Enable depth testing
        gl.Enable(gl::DEPTH_TEST);
        // Enable back-face culling
        gl.Enable(gl::CULL_FACE);
    }

    resources.insert(gl);
    resources.insert(gl_context);
    resources.insert(video_subsystem);
    resources.insert(window);
    resources.insert(viewport);
    resources.insert(color_buffer);
    resources.insert(sdl);
    resources.insert(event_pump);


    Ok(())
}

#[system]
pub fn render_prepare(
    #[resource] gl: &gl::Gl,
    #[resource] color_buffer: &mut color_buffer::ColorBuffer,
) {
    unsafe {
        // Clear the depth buffer
        gl.Clear(gl::DEPTH_BUFFER_BIT);
        // Clear the color buffer
        color_buffer.clear(&gl);
    }
}

/// Render every Model with a Position
#[system(for_each)]
pub fn render(
    position: &na::Isometry3<f32>, 
    model: &Model, 
    #[resource] camera: &Camera, 
    #[resource] gl: &gl::Gl
) {
    model.render(gl, &camera.get_view_matrix(), &camera.get_projection_matrix(), &camera.position.translation.vector.into(), position)
}

/// Swap in the backbuffer and wait for OpenGL to finish
#[system]
pub fn render_finish(
    #[resource] window: &sdl2::video::Window,
    #[resource] gl: &gl::Gl,
) {
    window.gl_swap_window();
    unsafe { gl.Finish(); }
}