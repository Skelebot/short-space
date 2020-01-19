extern crate gl;
extern crate sdl2;
extern crate nalgebra;
extern crate vec_2_10_10_10;
extern crate half;
extern crate image;
extern crate tobj;
extern crate floating_duration;
#[macro_use] extern crate render_gl_derive;
#[macro_use] extern crate failure;

mod render_gl;
mod camera;
mod resources;
mod input;
mod light;
mod entity;
mod debug;
mod model;
mod settings;
mod game_state;
mod scene;

use settings::GameSettings;
use game_state::GameState;
use scene::Scene;
use failure::err_msg;
use crate::resources::Resources;
use std::path::Path;
use debug::failure_to_string;
use std::time::Instant;
use nalgebra as na;
use camera::FpsCamera;
use floating_duration::TimeAsFloat;
use input::Input;

fn main() {
    if let Err(e) = run() {
        println!("{}", failure_to_string(e));
    }
}

fn run() -> Result<(), failure::Error> {

    let mut settings: GameSettings = Default::default(); 
    settings.debug = false;
    settings.vsync = true;
 
    //--------------------
    let res = Resources::from_relative_exe_path(Path::new("assets")).unwrap();

    let sdl = sdl2::init().map_err(err_msg)?;
    let video_subsystem = sdl.video().map_err(err_msg)?;
    
    let gl_attr = video_subsystem.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(4, 5);

    let window = video_subsystem
        .window("Game", settings.window_width as u32, settings.window_height as u32)
        .opengl()
        .resizable()
        .build()?;

    let _gl_context = window.gl_create_context().map_err(err_msg)?;
    let gl = gl::Gl::load_with(|s| {
        video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void
    });

    if settings.vsync {
        video_subsystem.gl_set_swap_interval(1).unwrap();
    } else {
        video_subsystem.gl_set_swap_interval(0).unwrap();
    }

    let mut viewport = render_gl::Viewport::for_window(settings.window_width, settings.window_height);
    viewport.set_used(&gl);
    let color_buffer = render_gl::ColorBuffer::from_color(na::Vector3::new(0.0, 0.0, 0.0));
    color_buffer.set_used(&gl);
    unsafe {
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl.TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
    }
    //--------------------

    let mut camera = FpsCamera::new(viewport.get_aspect(), 3.14/2.0, 0.01, 1000.0); 
    let mut game_state = GameState::new(&mut camera);
    let scene = Scene::new(&res, &gl, settings.debug)?;
    game_state.active_scene = Some(&scene);

    let mut input = Input::new(settings.mouse_sensitivity, settings.movement_speed);
    
    let mut time = Instant::now();

    //------------------------------
    // main loop
    //------------------------------
    let mut event_pump = sdl.event_pump().map_err(err_msg)?;
    'main: loop {

        let delta = time.elapsed().as_fractional_secs() as f32;
        time = Instant::now();

        //handle input
        for event in event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => break 'main,
                sdl2::event::Event::Window {
                    win_event: sdl2::event::WindowEvent::Resized(w, h),
                    ..
                } => {
                    viewport.update_size(w, h);
                    game_state.active_camera.update_aspect(viewport.get_aspect());
                    viewport.set_used(&gl);
                },
                e => input.handle_event(&e, &mut game_state, delta),
            }
        }
        input.update(&mut game_state, delta);

        // release mouse cursor
        sdl.mouse().set_relative_mouse_mode(!game_state.in_menu);

        //TODO: Move somewhere
        unsafe {
            gl.Enable(gl::CULL_FACE);
            gl.Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl.Enable(gl::DEPTH_TEST);
        }

        //clear the color buffer
        color_buffer.clear(&gl);
        for entity in game_state.active_scene.unwrap().entities.iter() {
            entity.render(
                &gl,
                &game_state.active_camera.get_view_matrix(),
                &game_state.active_camera.get_projection_matrix(),
                &game_state.active_camera.get_position(),
                &game_state.active_scene.unwrap().lights,
            );
        }

        window.gl_swap_window();
        unsafe {
            gl.Finish();
        }
    }

    Ok(())
}

