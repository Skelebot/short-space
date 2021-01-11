extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[macro_use]
extern crate log;

mod graphics;

mod assets;
mod input;
mod physics;
mod player;
mod spacetime;
mod state;

#[cfg(test)]
mod tests;

use assets::settings::GameSettings;
use graphics::Graphics;
use spacetime::{Child, Position};

use anyhow::Result;
use legion::{Resources, Schedule, World};

use state::GameState;

use futures::executor::block_on;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    info!("Starting up");

    // Create the Legion world (where entities live)
    let mut world = World::default();
    // Create the resource storage
    let mut resources = Resources::default();

    // AssetLoader is already needed to load shaders
    let loader = assets::AssetLoader::from_relative_exe_path(std::path::Path::new("assets"))?;
    resources.insert(loader);

    // Set up graphics (window, wgpu)
    let (mut graphics, event_loop) = block_on(graphics::setup(&mut world, &mut resources))?;

    setup_resources(&mut world, &mut resources)?;
    setup_scene(&mut world, &mut resources, &mut graphics)?;

    // Create the schedule that will be executed every frame
    let mut schedule = Schedule::builder()
        .add_system(physics::step_system())
        .add_system(player::player_movement_system())
        .add_system(physics::lerp_system())
        .add_system(physics::children_update_system())
        .build();

    info!("Running the event loop");
    event_loop.run(move |event, _, control_flow| {
        {
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
        }
        match event {
            Event::NewEvents(_) => {
                // Reset input to values before any events get handled
                // (for example zero the mouse delta)
                input::prepare(&mut resources);
                // Update frame timings
                spacetime::prepare(&mut resources);
            }
            // If the user closed the window, exit
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            // Handle window resizing
            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                graphics.resize(size, &mut world, &mut resources).unwrap();
            }
            // Handle user input
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => input::handle_keyboard_input(input, &mut resources),
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => input::handle_mouse_movement(delta, &mut resources),
            // Event::Suspended
            // Event::Resumed
            // Emitted when all of the event loop's input events have been processed and redraw processing is about to begin.
            Event::MainEventsCleared => {
                // Run all systems
                schedule.execute(&mut world, &mut resources);
                // Request rendering
                graphics.window.request_redraw();
            }
            // Render the frame
            Event::RedrawRequested(_) => {
                // Render
                graphics.render(&mut world, &mut resources).unwrap();
            }
            _ => {}
        }
    })
}

// TODO: Create a loading state and add a loding screen
fn setup_resources(world: &mut World, resources: &mut Resources) -> Result<()> {
    physics::setup(world, resources)?;

    let settings = resources
        .get::<assets::AssetLoader>()
        .unwrap()
        .load::<GameSettings>("settings/game.ron")?;

    let game_state = GameState::new();

    let input_state = input::InputState::default();

    let time = spacetime::Time::default();

    resources.insert(settings);
    resources.insert(input_state);
    resources.insert(game_state);
    resources.insert(time);

    Ok(())
}

// TODO: Create a loading state and add a loding screen
fn setup_scene(
    world: &mut World,
    resources: &mut Resources,
    graphics: &mut Graphics,
) -> Result<()> {
    {
        let loader = resources.get::<assets::AssetLoader>().unwrap();
        loader.load_scene(world, graphics, "scenes/test.ron")?
    }

    // Create the player
    let pos: Position = na::Isometry3::from_parts(
        na::Translation3::new(0.0, -2.0, 0.1),
        na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), -90.0_f32.to_radians()),
    )
    .into();
    use nc::shape::{Capsule, ShapeHandle};
    let collider = physics::Collider::from(
        // TODO: Load from settings
        ShapeHandle::new(Capsule::new(2.0, 0.4)),
    );
    let vel = physics::Velocity::new(na::Vector3::repeat(0.0_f32), na::Vector3::repeat(0.0));
    use player::*;
    let player = Player {
        state: PlayerState::Noclip,
        ground_entity: None,
        flags: 0,
    };

    // Set up the camera
    let size = graphics.window.inner_size();
    let aspect = size.width as f32 / size.height as f32;
    let camera = crate::graphics::Camera::new(aspect, 45_f32.to_radians(), 0.001, 1000.0);
    let atlas_cam = world.push((pos, camera));

    // Add the player to the world and keep it's Entity (an ID)
    // so we can add it to a Resource to track the single main player
    let atlas = world.push((pos, collider, vel, player));

    world.entry(atlas_cam).unwrap().add_component(Child {
        parent: atlas,
        offset: na::Isometry3::translation(0.0, 0.0, 0.0).into(),
    });

    let atlas = player::Atlas {
        player: atlas,
        camera: atlas_cam,
    };
    resources.insert(atlas);

    Ok(())
}
