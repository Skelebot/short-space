#[macro_use]
extern crate render_gl_derive;
extern crate nalgebra as na;
extern crate ncollide3d as nc;

mod graphics;
mod asset_loader;
mod input;
mod settings;
mod game_state;
mod physics;
//mod networking;
mod world;
mod time;
mod player;

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
    let settings: GameSettings = Default::default(); 
    
    // Create physics settings
    let phys_settings = physics::PhysicsSettings::default();

    // Create the asset loader
    let asset_loader = AssetLoader::from_relative_exe_path(Path::new("assets")).unwrap();

    // Create the game state tracking Resource
    let game_state = GameState::new();

    let input_state = input::InputState::default();

    // Create the frame-delta tracking Resource
    let time = time::Time::new();

    // Insert the resources
    resources.insert(settings);
    resources.insert(input_state);
    resources.insert(phys_settings);
    resources.insert(asset_loader);
    resources.insert(game_state);
    resources.insert(time);
    
    graphics::setup_window(&mut world, &mut resources)?;
    setup_scene(&mut world, &mut resources)?;

    use legion::system;
    #[system]
    fn aa(#[resource] world: &mut World) {
        println!("{:?}", world.is_empty());
    }

    // Create the schedule that will be executed every frame
    let mut schedule = Schedule::builder()
        .add_thread_local(time::update_time_system())
        .add_thread_local(input::handle_input_system())
        .add_system(player::player_movement_system())
        .flush()
        .add_thread_local(graphics::render_prepare_system())
        .add_thread_local(graphics::render_system())
        .add_thread_local(graphics::render_finish_system())
        .build();

    //------------------------------
    // main loop
    //------------------------------
    while !resources.get::<GameState>()
        .ok_or(Error::msg("GameState not found"))?
        .should_exit
    {
        // Execute the schedule
        schedule.execute(&mut world, &mut resources);
    }

    let gl = resources.get::<gl::Gl>()
        .ok_or(Error::msg("Gl not found"))?;

    // Destroy all things that need to be destroyed
    use legion::IntoQuery;

    let mut query = legion::Write::<graphics::Model>::query();

    for model in query.iter_mut(&mut world) {
        unsafe { model.destroy(&gl); }
    }

    Ok(())
}

// TODO: Create a Loading state and add a loding screen
use graphics::{Model, shader};
fn setup_scene(world: &mut World, resources: &mut Resources) -> Result<()> {

    {
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
        let pos = na::Isometry3::<f32>::from_parts(
            na::Translation3::new(0.0, 0.0, 0.0),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
        );
        world.push((model, pos));
        // Create a box
        let model = Model::new(
            &loader, 
            &gl, 
            "models/xyz_cube.obj", 
            &shader, 
            settings.debug
        )?;
        let pos = na::Isometry3::<f32>::from_parts(
            na::Translation3::new(0.0, 3.0, 0.0),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
        );
        world.push((model, pos));
    }
    
    // Create the player
    let pos = na::Isometry3::<f32>::from_parts(
        na::Translation3::new(0.0, 5.0, 2.0),
        na::UnitQuaternion::from_axis_angle(&na::Vector3::y_axis(), 0.0),
    );
    use nc::shape::{ShapeHandle, Capsule};
    let collider = physics::Collider::from(
        ShapeHandle::new(Capsule::new(1.0, 1.0))
    );
    let vel = physics::Velocity::new(
        na::Vector3::repeat(0.0), 
        na::Vector3::repeat(0.0)
    );
    use player::*;
    let player = Player {
        state: PlayerState::Spectator,
        movement_state: MovementState::Airborne,
    };
    let atlas = world.push((pos, collider, vel, player));

    let atlas = player::Atlas { entity: atlas };
    resources.insert(atlas);

    Ok(())
}