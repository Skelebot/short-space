use nalgebra as na;
use anyhow::{Result, Error};
use legion::{World, Resources, system};

use crate::settings::GameSettings;

mod buffer;
mod color_buffer;
mod data;
pub mod model;
pub mod shader;
pub mod texture;
pub mod viewport;
//mod bitmap_font;
pub mod camera;
// TODO: What's wrong with it?
//mod light;


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
        .window("Game", settings.window_width as u32, settings.window_height as u32)
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
    let color_buffer = color_buffer::ColorBuffer::from_color(na::Vector3::new(0.0, 0.0, 0.0));
    color_buffer.set_used(&gl);
    unsafe {
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    }

    let camera = camera::FpsCamera::new(viewport.get_aspect(), 3.14/2.0, 0.01, 1000.0); 

    let event_pump = sdl.event_pump().map_err(|e| Error::msg(e))?;

    resources.insert(gl);
    resources.insert(gl_context);
    resources.insert(window);
    resources.insert(viewport);
    resources.insert(camera);
    resources.insert(color_buffer);
    resources.insert(sdl);
    resources.insert(event_pump);

    Ok(())
}

use model::Model;
use camera::FpsCamera;
use camera::Camera;

#[system]
pub fn render_prepare(
    #[resource] gl: &gl::Gl,
    #[resource] color_buffer: &mut color_buffer::ColorBuffer,
) {
    //println!("render_prep");
    unsafe {
        // Enable back-face culling
        gl.Enable(gl::CULL_FACE);
        // Clear the depth and color buffers
        gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        // Enable depth testing
        gl.Enable(gl::DEPTH_TEST);
        color_buffer.clear(&gl);
    }
}

#[system(for_each)]
pub fn render(
    position: &na::Isometry3<f32>, 
    model: &Model, 
    #[resource] camera: &FpsCamera, 
    #[resource] gl: &gl::Gl
) {
    //println!("render");
    model.render(gl, &camera.get_view_matrix(), &camera.get_projection_matrix(), &camera.get_position().translation.vector.into(), position)
}

#[system]
pub fn render_finish(
    #[resource] window: &sdl2::video::Window,
) {
    //println!("render_finish");
    window.gl_swap_window();
    //unsafe { gl.Finish(); }
}