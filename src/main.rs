//#[macro_use]
//extern crate render_gl_derive;

extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[macro_use]
extern crate log;

pub mod graphics;
use graphics::Graphics;

use graphics::mesh::MeshPass;

mod asset_loader;
mod input;
mod settings;
mod game_state;
mod physics;
mod time;
mod player;

#[cfg(test)]
mod tests;

use settings::GameSettings;
use anyhow::{Result, Error};
use legion::{World, Resources, Schedule};

use game_state::GameState;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
};
use futures::executor::block_on;

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    info!("Starting up");

    // Create the Legion world (where entities live)
    let mut world = World::default();
    // Create the resource storage
    let mut resources = Resources::default();

    let (mut graphics, event_loop) = block_on(graphics::setup(&mut world, &mut resources))?;

    let mesh_pass = MeshPass::new(
        &mut graphics.device,
        &graphics.sc_desc,
        &mut world,
        &mut resources
    )?;
    graphics.render_passes.push(Box::new(mesh_pass));

    setup_resources(&mut world, &mut resources, &graphics.window)?;
    setup_scene(&mut world, &mut resources, &mut graphics)?;

    block_on(run(graphics, event_loop, world, resources))?;

    Ok(())
}

async fn run(
    mut graphics: Graphics,
    event_loop: EventLoop<()>,
    mut world: legion::World,
    mut resources: legion::Resources,
) -> Result<()> {

    // Create the schedule that will be executed every frame
    let mut schedule = Schedule::builder()
        .add_thread_local(time::update_time_system())
        .add_thread_local(player::player_movement_system())
        .build();

    debug!("Running the event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        if resources.get::<GameState>().unwrap().should_exit {
            *control_flow = ControlFlow::Exit;
        }
        if !resources.get::<GameState>().unwrap().paused {
            graphics.window.set_cursor_grab(true).unwrap();
            graphics.window.set_cursor_visible(false);
        } else {
            graphics.window.set_cursor_grab(false).unwrap();
            graphics.window.set_cursor_visible(true);
        }
        match event {
            // If the user closed the window, exit
            Event::WindowEvent { event: WindowEvent::CloseRequested, .. } => *control_flow = ControlFlow::Exit,
            // Handle window resizing
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                graphics.resize(size, &mut world, &mut resources).unwrap();
            },
            // Handle user input
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => input::handle_keyboard_input(input, &mut world, &mut resources),
                WindowEvent::CursorMoved { position, .. } => input::handle_cursor_moved(position, &graphics.window, &mut world, &mut resources),
                _ => {},
            },
            // Emitted when all of the event loop's input events have been processed and redraw processing is about to begin.
            // Normally, we would use Event::RedrawRequested for rendering, but we can also just render here, because it's a game
            // that has to render continuously either way.
            Event::MainEventsCleared => {
                // Execute all systems
                schedule.execute(&mut world, &mut resources);
                // Render
                graphics.render(&mut world, &mut resources).unwrap();
            },
            // We already render frames in MainEventsCleared
            Event::RedrawRequested(_) => {},
            _ => {},
        }
    })
}

// TODO: Create a loading state and add a loding screen
fn setup_resources(_world: &mut World, resources: &mut Resources, window: &winit::window::Window) -> Result<()> {
    // Set up the camera
    let size = window.inner_size();
    let aspect = size.width as f32 / size.height as f32;
    let mut camera = crate::graphics::Camera::new(aspect, 45_f32.to_radians(), 0.001, 1000.0); 
    camera.position.rotation = na::UnitQuaternion::from_axis_angle(
        &na::Vector::x_axis(), 
        0.0f32.to_radians()
    );
    resources.insert(camera);

    // Create settings for the game
    // TODO: Read settings from a file
    let settings: GameSettings = Default::default(); 
    
    // Create the asset loader
    //let asset_loader = AssetLoader::from_relative_exe_path(Path::new("assets")).unwrap();

    // Create physics settings
    let phys_settings = physics::PhysicsSettings::default();

    // Create the game state tracking Resource
    let game_state = GameState::new();

    let input_state = input::InputState::default();

    // Create the frame-delta tracking Resource
    let time = time::Time::new();

    let asset_loader = asset_loader::AssetLoader::from_relative_exe_path(std::path::Path::new("assets"))?;

    // Insert the resources
    resources.insert(settings);
    resources.insert(input_state);
    resources.insert(phys_settings);
    resources.insert(asset_loader);
    resources.insert(game_state);
    resources.insert(time);

    Ok(())
}

// TODO: Create a loading state and add a loding screen
fn setup_scene(world: &mut World, resources: &mut Resources, graphics: &mut Graphics) -> Result<()> {
    // Create a (temporary) CommandEncoder for loading data to GPU
    let mut encoder = graphics.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

    {
        let loader = resources.get::<asset_loader::AssetLoader>().unwrap();
        let cube_model_data = loader.load_model("models/testmap.obj")?;

        // This has to be fetched every time we want to upload a model to the GPU.
        // It's more of a quick hack to get things working and will be rewritten in the future
        let mesh_bind_group_layout = resources.get::<graphics::mesh::mesh_pass::MeshBindGroupLayout>()
            .ok_or(Error::msg("MeshPass not found"))?;

        let cube_model = graphics::mesh::Model::from_data(
            cube_model_data,
            &mut graphics.device,
            &mut encoder,
            &mesh_bind_group_layout
        );

        let cube_pos = physics::Position::from(
            na::Isometry3::from_parts(
                na::Translation3::new(0.0, 10.0, 0.0),
                na::UnitQuaternion::from_axis_angle(
                    &na::Vector3::z_axis(),
                    0_f32.to_radians(),
                )
            )
        );

        let cube_scale = physics::Scale::from(
            na::Vector3::new(2.0, 2.0, 2.0)
        );

        world.push((cube_model, cube_pos, cube_scale));
    }
    graphics.queue.submit(Some(encoder.finish()));

    // Create the player
    let pos = 
    physics::Position::from(na::Isometry3::<f32>::from_parts(
        na::Translation3::new(0.0, 0.0, 0.0),
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
        ground_entity: None,
        flags: 0,
    };
    // Add the player to the world and keep it's Entity (an ID)
    // so we can add it to a Resource to track the single main player
    let atlas = world.push((pos, collider, vel, player));

    let atlas = player::Atlas { entity: atlas };
    resources.insert(atlas);

    Ok(())
}