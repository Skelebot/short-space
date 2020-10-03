#[macro_use]
extern crate render_gl_derive;

mod graphics;

mod asset_loader;
mod input;
mod settings;
mod game_state;
//mod physics;
//mod networking;
mod world;
mod time;

use nalgebra as na;
use settings::GameSettings;
use anyhow::{Result, Error};
use legion::{World, Resources, Schedule};

use game_state::GameState;

use crate::asset_loader::AssetLoader;
use std::path::Path;

fn main() -> Result<(), anyhow::Error> {
    run()
}

fn run() -> Result<(), anyhow::Error> {

    // Create the Legion world (where entities live)
    let mut world = World::default();
    // Create the resource storage
    let mut resources = Resources::default();

    // Create settings for the game
    // TODO: Read settings from a file
    let mut settings: GameSettings = Default::default(); 
    settings.debug = false;
    settings.vsync = true;

    // Create the asset loader
    let asset_loader = AssetLoader::from_relative_exe_path(Path::new("assets")).unwrap();

    // Create the game state tracking Resource
    let game_state = GameState::new();

    // Create the frame-delta tracking Resource
    let time = time::Time::new();

    // Insert the resources
    resources.insert(settings);
    resources.insert(asset_loader);
    resources.insert(game_state);
    resources.insert(time);
    
    graphics::setup_window(&mut world, &mut resources)?;
    setup_scene(&mut world, &mut resources)?;

    // Create the schedule that will be executed every frame
    let mut schedule = Schedule::builder()
        .add_thread_local_fn(time::update_time)
        .add_thread_local_fn(input::handle_input)
        .flush()
        .add_thread_local(graphics::render_prepare_system())
        .add_thread_local(graphics::render_system())
        .add_thread_local(graphics::render_finish_system())
        .build();

    // DO NOT USE
    //let font_options = render_gl::bitmap_font::BitmapFontOptions::new(14, 26, 4, "fonts/cherry-13x2.bmp", "fonts/shaders/font");
    //let font = render_gl::bitmap_font::BitmapFont::new(&gl, &res, font_options, viewport.w as u32, viewport.h as u32)?;

    //let mut serv_con = networking::ServerConnection::new("127.0.0.1:28685");
    //serv_con.connect("28686")?;

    //let packet = networking::serializer::create_client_packet(
    //    input.create_input_message(), None);

    //serv_con.send_data(&packet[..])?;
        
    //------------------------------
    // main loop
    //------------------------------
    while !resources.get::<GameState>()
        .ok_or(Error::msg("GameState not found"))?
        .should_exit
    {
        // TODO: scheduled execution?
        schedule.execute(&mut world, &mut resources);
    }

    let gl = resources.get::<gl::Gl>()
        .ok_or(Error::msg("Gl not found"))?;

    // Destroy all things that need to be destroyed
    use legion::IntoQuery;

    let mut query = legion::Write::<graphics::model::Model>::query();

    for model in query.iter_mut(&mut world) {
        unsafe { model.destroy(&gl); }
    }

    Ok(())
}

use graphics::{model::Model, shader};
fn setup_scene(world: &mut World, resources: &mut Resources) -> Result<()> {
    let loader = resources.get::<AssetLoader>()
        .ok_or(Error::msg("AssetLoader not found"))?;

    let gl = resources.get::<gl::Gl>()
        .ok_or(Error::msg("Gl not found"))?;

    let settings = resources.get::<GameSettings>()
        .ok_or(Error::msg("Settings not found"))?;

    let shader = shader::Program::from_res(&gl, &loader, "shaders/model")?;

    // Create the map
    let model = Model::new(
        &loader, 
        &gl, 
        "models/skatepark.obj", 
        &shader, 
        settings.debug
    )?;

    // FIXME
    //unsafe { shader.destroy(&gl); }

    let pos = na::Isometry3::<f32>::identity();
    world.push((model, pos));

    Ok(())
}

