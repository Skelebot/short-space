extern crate color_eyre as eyre;
extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[macro_use]
extern crate log;

mod player;
mod state;
mod settings;

use engine::{assets::AssetLoader, graphics, input, spacetime, state::{CustomEvent, StateMachine}};

use eyre::Result;
use legion::{Resources, World};

use futures::executor::block_on;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::ControlFlow,
};

fn main() -> Result<()> {
    // Set up the terminal
    env_logger::init();
    color_eyre::install()?;
    info!("Starting up");

    // Create the world
    let mut world = World::default();
    // Create the resource storage
    let mut resources = Resources::default();

    // AssetLoader is already needed to load shaders
    let loader = AssetLoader::from_relative_exe_path(std::path::Path::new("assets"))?;
    resources.insert(loader);

    // Set up graphics (window, wgpu)
    let (mut graphics, event_loop) = block_on(graphics::setup(&mut world, &mut resources))?;

    // Set up essential resources
    let input_state = input::InputState::default();
    let time = spacetime::Time::default();
    resources.insert(input_state);
    resources.insert(time);

    let mut state_machine = StateMachine::new(state::MainState::new());
    state_machine.start(&mut world, &mut resources)?;

    info!("Running the event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match &event {
            &Event::NewEvents(_) => {
                // Reset input to values before any events get handled
                // (for example zero the mouse delta)
                input::prepare(&mut resources);
                // Update UI frame timings
                graphics.prepare(&mut resources);
                // Update frame timings
                spacetime::prepare(&mut resources);
            }
            // If the user closed the window, exit
            &Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | &Event::UserEvent(CustomEvent::Exit) => {
                state_machine.stop(&mut world, &mut resources);
                *control_flow = ControlFlow::Exit
            }
            // Handle window resizing
            &Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                graphics.resize(size, &mut world, &mut resources).unwrap();
            }
            // Handle user input
            &Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => input::handle_keyboard_input(input, &mut resources),
            &Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } => input::handle_mouse_movement(delta, &mut resources),
            // Event::Suspended
            // Event::Resumed
            // Emitted when all of the event loop's input events have been processed and redraw processing is about to begin.
            &Event::MainEventsCleared => {
                state_machine.update(&mut world, &mut resources);
                // Request rendering
                //graphics.window.request_redraw();
                // Render
                graphics.render(&mut world, &mut resources).unwrap();
            }
            // Render the frame
            &Event::RedrawRequested(_) => {}
            _ => {}
        }
        // Handle events by UI
        graphics.ui_pass.handle_event(&graphics.window, &event);
        state_machine.handle_event(&mut world, &mut resources, event);
    });
}
