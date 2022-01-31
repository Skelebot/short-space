extern crate color_eyre as eyre;
extern crate nalgebra as na;
extern crate ncollide3d as nc;

#[macro_use]
extern crate log;

mod player;
mod settings;
mod state;
mod ui;

use engine::{
    assets::AssetLoader,
    graphics, input, spacetime,
    state::{CustomEvent, StateMachine},
};

use eyre::{eyre::Context, Result};
use legion::{Resources, World};

use futures::executor::block_on;
use winit::{
    event::{DeviceEvent, Event, WindowEvent},
    event_loop::ControlFlow,
};

pub fn main() -> Result<()> {
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

    // Set up UI
    let mut egui = egui_winit_platform::Platform::new(egui_winit_platform::PlatformDescriptor {
        physical_height: graphics.window.inner_size().height,
        physical_width: graphics.window.inner_size().width,
        scale_factor: graphics.window.scale_factor(),
        style: Default::default(),
        font_definitions: Default::default(),
    });

    info!(
        "Window size: ({:?}, {:?}), * {:?}",
        graphics.window.inner_size().height,
        graphics.window.inner_size().width,
        graphics.window.scale_factor(),
    );

    info!("Running the event loop");
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        let event_captured = egui.captures_event(&event);
        egui.handle_event(&event);
        match &event {
            &Event::NewEvents(_) => {
                // Reset input to values before any events get handled
                // (for example zero the mouse delta)
                input::prepare(&mut resources);
                // Update UI frame timings
                graphics.prepare(&mut resources);
                // Update frame timings
                spacetime::prepare(&mut resources);

                egui.begin_frame();
                // Overwrite the old context with a new one
                resources.insert(egui.context());
            }

            // Handle closing the window and exit events sent by the game
            &Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
            | &Event::UserEvent(CustomEvent::Exit) => {
                state_machine
                    .stop(&mut world, &mut resources)
                    .wrap_err("State machine errored while stopping")
                    .unwrap();
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
            } if !event_captured => input::handle_keyboard_input(input, &mut resources),
            &Event::DeviceEvent {
                event: DeviceEvent::MouseMotion { delta },
                ..
            } if !event_captured => input::handle_mouse_movement(delta, &mut resources),

            // Event::Suspended
            // Event::Resumed

            // Emitted when all of the event loop's input events have been processed and redraw processing is about to begin.
            &Event::MainEventsCleared => {
                if state_machine
                    .update(&mut world, &mut resources)
                    .wrap_err("Fatal error while updating states")
                    .unwrap()
                {
                    send_exit(&resources).unwrap();
                }

                // Prepare to draw the UI (only when a repaint is needed)
                // TODO: Cache the rendered UI and slap it on top of everything;
                // repaint only when needed
                #[allow(irrefutable_let_patterns)]
                let ui = if let (
                    //egui::Output {
                    //    needs_repaint: true,
                    //    ..
                    //},
                    _,
                    data,
                ) = egui.end_frame(Some(&graphics.window))
                {
                    Some((egui.context().tessellate(data), egui.context().texture()))
                } else {
                    None
                };

                // Render
                graphics.render(&mut world, &mut resources, ui).unwrap();
            }
            // Render the frame
            &Event::RedrawRequested(_) => {}
            _ => {}
        }

        // Per-state event handling
        if !event_captured
            && state_machine
                .handle_event(&mut world, &mut resources, event)
                .wrap_err("Fatal error when handling events")
                .unwrap()
        {
            send_exit(&resources).unwrap();
        }
    });
}

/// Sends a signal to the event loop to immediately clean up everything and exit
pub fn send_exit(resources: &Resources) -> Result<()> {
    resources
        .get::<winit::event_loop::EventLoopProxy<CustomEvent>>()
        .unwrap()
        .send_event(CustomEvent::Exit)
        .wrap_err("EventLoop is no more")
}
