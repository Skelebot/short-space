#[macro_use]
extern crate render_gl_derive;

extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[macro_use]
extern crate log;

mod graphics;
mod wgpu_graphics;

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
    env_logger::init();
    info!("Starting up");
    wgpu_graphics::start()?;
    //run()
    Ok(())
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

    wgpu_graphics::setup(&mut world, &mut resources);
    
    graphics::setup_window(&mut world, &mut resources)?;
    setup_scene(&mut world, &mut resources)?;

    // Create the schedule that will be executed every frame
    let mut schedule = Schedule::builder()
        .add_thread_local(time::update_time_system())
        .add_thread_local(input::handle_input_system())
        .add_thread_local(player::player_movement_system())
        .add_thread_local(test_system())
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
    let mut query = legion::Write::<graphics::Model>::query();

    for model in query.iter_mut(&mut world) {
        unsafe { model.destroy(&gl); }
    }

    Ok(())
}

// TODO: Create a Loading state and add a loding screen
use graphics::{Model, shader};
fn setup_scene(world: &mut World, resources: &mut Resources) -> Result<()> {

    let map;
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
            "models/warsztaty.obj", 
            &shader, 
        )?;
        let pos = na::Isometry3::<f32>::identity();
        let collider = physics::Collider::from(
            nc::shape::ShapeHandle::new(
                model.get_trimesh()
                //nc::shape::Cuboid::new(na::Vector3::new(10.0, 10.0, 0.5))
            )
        );
        map = world.push((model, pos, collider));
        world.entry(map).unwrap().add_component(map);

        // Create a box (mainly for debugging)
        let model = Model::new(
            &loader, 
            &gl, 
            "models/xyz_cube.obj", 
            &shader, 
        )?;
        let vel = physics::Velocity::new(
            na::Vector3::repeat(0.0), 
            na::Vector3::repeat(0.0)
        );
        let pos = na::Isometry3::<f32>::from_parts(
            na::Translation3::new(0.0, -2.0, 3.0),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), 20.0_f32.to_radians()),
        );
        world.push((model, pos, vel));
    }
    
    // Create the player
    let pos = 
    physics::Position::from(na::Isometry3::<f32>::from_parts(
        na::Translation3::new(1.0, 5.0, 3.0),
        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), -90.0_f32.to_radians()),
    ));
    use nc::shape::{ShapeHandle, Capsule};
    let collider = physics::Collider::from(
        ShapeHandle::new(Capsule::new(1.0, 1.0))
    );
    let vel = physics::Velocity::new(
        na::Vector3::repeat(0.0_f32), 
        na::Vector3::repeat(0.0)
    );
    use player::*;
    let player = Player {
        state: PlayerState::Noclip,
        ground_entity: Some(map),
        flags: 0,
    };
    // Add the player to the world and keep it's Entity (an ID)
    // so we can add it to a Resource to track the single main player
    let atlas = world.push((pos, collider, vel, player));

    let atlas = player::Atlas { entity: atlas };
    resources.insert(atlas);

    Ok(())
}

use legion::{system, Entity, world::SubWorld, IntoQuery};
#[system]
#[read_component(Entity)]
pub fn test(
    world: &mut SubWorld
) {
    //let mut query = <&Entity>::query();
    //for entity in query.iter(world) {
    //    println!("entity: {:?}", entity);
    //}
}