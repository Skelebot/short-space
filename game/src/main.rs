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
    // TODO: 4096 ??
    let mut egui = egui_winit::State::new(4096, &graphics.window);

    info!(
        "Window size: ({:?}, {:?}), * {:?}",
        graphics.window.inner_size().height,
        graphics.window.inner_size().width,
        graphics.window.scale_factor(),
    );
    
    let mut egui_ctx = egui::Context::default();

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

                let input = egui.take_egui_input(&graphics.window);
                egui_ctx.begin_frame(input);
                // Overwrite the old context with a new one
                resources.insert(egui_ctx.clone());
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
            
            Event::WindowEvent { event: event, .. }=> {
                let captured = egui.on_event(&egui_ctx, &event);
                if captured { return; }
                match event {
                    WindowEvent::KeyboardInput { input, .. } => input::handle_keyboard_input(*input, &mut resources),
                    //WindowEvent::AxisMotion { delta } => input::handle_mouse_movement(delta, &mut resources),
                    _ => (),
                }
            }

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
                let out = egui_ctx.end_frame();
                // TODO: make use of out.needs_repaint
                egui.handle_platform_output(&graphics.window, &egui_ctx, out.platform_output);

                // TODO: Cache ui?
                // Render
                graphics.render(&mut world, &mut resources, Some((&egui_ctx, out.textures_delta, out.shapes))).unwrap();
            }
            // Render the frame
            &Event::RedrawRequested(_) => {}
            _ => {}
        }

        // Per-state event handling
        if state_machine
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
