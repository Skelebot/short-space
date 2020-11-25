//#[macro_use]
//extern crate render_gl_derive;

extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[macro_use]
extern crate log;

//mod graphics;
pub mod wgpu_graphics;
pub use wgpu_graphics as graphics;

mod asset_loader;
mod input;
mod settings;
mod game_state;
mod physics;
//mod networking;
mod world;
mod time;
mod player;

#[cfg(test)]
mod tests;

use settings::GameSettings;
use anyhow::{Result, Error};
use legion::{World, Resources, Schedule};
use wgpu_graphics::Pass;

use game_state::GameState;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    info!("Starting up");

    // Create the Legion world (where entities live)
    let mut world = World::default();
    // Create the resource storage
    let mut resources = Resources::default();

    use futures::executor::block_on;
    let (
        mut device, 
        swap_chain,
        sc_desc,
        surface,
        queue,
        window,
        event_loop,
    ) = block_on(wgpu_graphics::setup(&mut world, &mut resources))?;

    setup_resources(&mut world, &mut resources, &window)?;

    let mesh_pass = wgpu_graphics::mesh::MeshPass::new(&mut device, &sc_desc, &mut world, &mut resources)?;
    resources.insert(mesh_pass);

    // Create a (temporary) CommandEncoder for loading data to GPU
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    setup_scene(&mut world, &mut resources, &mut device, &mut encoder)?;
    queue.submit(Some(encoder.finish()));

    block_on(
        run(
            device,
            swap_chain,
            sc_desc,
            surface,
            queue,
            window,
            event_loop,
            world,
            resources
        )
    )?;

    Ok(())
}

async fn run(
    mut device: wgpu::Device,
    mut swap_chain: wgpu::SwapChain,
    mut sc_desc: wgpu::SwapChainDescriptor,
    surface: wgpu::Surface,
    mut queue: wgpu::Queue,
    window: Window,
    event_loop: EventLoop<()>,
    mut world: legion::World,
    mut resources: legion::Resources,
) -> Result<()> {

    // Create the schedule that will be executed every frame
    let mut schedule = Schedule::builder()
        .add_thread_local(time::update_time_system())
        .add_thread_local(player::player_movement_system())
        .build();
        //.add_thread_local(input::handle_input_system())
        //.add_thread_local(test_system())
        //.flush()
        //.add_thread_local(graphics::render_prepare_system())
        //.add_thread_local(graphics::render_system())
        //.add_thread_local(graphics::render_finish_system())
        //.build();


    debug!("Running the event loop");
    event_loop.run(move |event, _, control_flow| {
        // Have the closure take ownership of the resources.
        // event_loop.run never returns, so we must do this to ensure 
        // the resources are properly cleaned up.
        // By moving all of those resources to an empty variable, all of them get dropped
        // and their drop() functions get called.

        *control_flow = ControlFlow::Poll;
        if resources.get::<GameState>().unwrap().should_exit {
            *control_flow = ControlFlow::Exit;
        }
        if !resources.get::<GameState>().unwrap().paused {
            window.set_cursor_grab(true).unwrap();
            window.set_cursor_visible(false);
        } else {
            window.set_cursor_grab(false).unwrap();
            window.set_cursor_visible(true);
        }
        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            Event::WindowEvent { event: WindowEvent::Resized(size), .. } => {
                // Recreate the swap chain with the new size
                sc_desc.width = size.width;
                sc_desc.height = size.height;
                let mut camera = resources.get_mut::<graphics::Camera>().unwrap();
                camera.update_aspect(size.width as f32/size.height as f32);
                let proj_view: [[f32; 4]; 4] = camera.get_vp_matrix().into();
                swap_chain = device.create_swap_chain(&surface, &sc_desc);

                // Update the mesh pass
                let mut mesh_pass = resources.get_mut::<graphics::mesh::MeshPass>().unwrap();
                queue.write_buffer(
                    &mesh_pass.global_uniform_buf,
                    0,
                    // FIXME: cast_slice()?
                    bytemuck::bytes_of(&proj_view),
                );
                mesh_pass.resize(&sc_desc, &mut device).unwrap();
            },
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::KeyboardInput { input, .. } => input::handle_keyboard_input(input, &mut world, &mut resources),
                WindowEvent::CursorMoved { position, .. } => input::handle_cursor_moved(position, &window, &mut world, &mut resources),
                _ => {},
            },
            // Emmited when all of the event loop's input events have been processed and redraw processing is about to begin.
            // Normally, we would use Event::RedrawRequested for rendering, but we can also just render here, because it's a game
            // that has to render continuously either way.
            Event::MainEventsCleared => {
                // Execute the schedule
                schedule.execute(&mut world, &mut resources);

                //window.request_redraw();

                // Render
                let mut frame = swap_chain
                    .get_current_frame()
                    .map_err(|err| 
                        Error::msg("Failed to acquire next swap chain texture")
                            .context(err)
                    )
                    .unwrap()
                    .output;

                let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });                    

                let mut mesh_pass = resources.get_mut::<graphics::mesh::MeshPass>().unwrap();
                mesh_pass.render(&mut encoder, &mut queue, &mut frame, &world, &resources).unwrap();

                queue.submit(Some(encoder.finish()));
            },
            Event::RedrawRequested(_) => {
            },
            _ => {},
        }
    });
}

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

// TODO: Create a Loading state and add a loding screen
fn setup_scene(world: &mut World, resources: &mut Resources, device: &mut wgpu::Device, encoder: &mut wgpu::CommandEncoder) -> Result<()> {

    {
        let loader = resources.get::<asset_loader::AssetLoader>().unwrap();
        let cube_model_data = loader.load_simple_model("models/test_cube.obj")?;

        let mesh_pass = resources.get::<graphics::mesh::MeshPass>()
            .ok_or(Error::msg("MeshPass not found"))?;

        let cube_model = graphics::mesh::Model::from_data(
            cube_model_data,
            device,
            encoder,
            &mesh_pass
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

#[allow(dead_code, unused_imports)]
use legion::{system, Entity, world::SubWorld, IntoQuery};
#[system]
#[read_component(Entity)]
pub fn test(
    _world: &mut SubWorld
) {
    //let mut query = <&Entity>::query();
    //for entity in query.iter(world) {
    //    debug!("entity: {:?}", entity);
    //}
}
