use legion::{Resources, Schedule, World};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use crate::ui::{console::Console, StatusWindow};

use super::loading::LoadingState;
use eyre::{eyre::WrapErr, Result};

use engine::state::{CustomEvent, State, Transition};

pub struct MainState {
    schedule: Schedule,
    ui: Vec<Box<dyn StatusWindow>>,
    console: Console,
}

impl MainState {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            //.add_system()
            .build();
        MainState {
            schedule,
            ui: Vec::new(),
            console: Console::default(),
        }
    }
}

impl State for MainState {
    fn handle_event(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        event: &winit::event::Event<CustomEvent>,
    ) -> Result<Transition> {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode,
                                ..
                            },
                        ..
                    },
                ..
            } => match virtual_keycode {
                Some(VirtualKeyCode::Escape) => {
                    resources
                        .get::<winit::event_loop::EventLoopProxy<CustomEvent>>()
                        .unwrap()
                        .send_event(CustomEvent::Exit)
                        .wrap_err("EventLoop is no more")?;
                    // Doesn't matter
                    Ok(Transition::None)
                }
                Some(VirtualKeyCode::Return) => Ok(Transition::Push(Box::new(LoadingState::new()))),
                _ => Ok(Transition::None),
            },
            _ => Ok(Transition::None),
        }
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) -> Result<Transition> {
        self.schedule.execute(world, resources);
        // Draw UI
        {
            let ctx = resources.get::<egui::CtxRef>().unwrap();
            // divided by pixels per point for a value in pixels, plus offset for margin
            let left_center = (ctx.available_rect().left_center().to_vec2()
                / ctx.pixels_per_point())
                + egui::Vec2::new(50.0, 0.0);
            let response = egui::Window::new("Main menu")
                // When set with anchor LEFT_CENTER, the window jitters until the window is first resized.
                // No idea why it happens
                //.anchor(egui::Align2::LEFT_CENTER, [50.0, 0.0])
                .fixed_pos(left_center.to_pos2())
                .resizable(false)
                .title_bar(false)
                .frame(egui::Frame {
                    ..Default::default()
                })
                .show(&ctx, |ui| {
                    ui.label("ENDLESS JOSH");
                    if ui.button("Start").clicked() {
                        return Transition::Push(Box::new(LoadingState::new()));
                    };
                    if ui.button("Exit").clicked() {
                        return Transition::Pop;
                    };
                    Transition::None
                });
            // The window is not collapsible and not closeable, so inner and response are never None
            response
                .map(|r| Ok(r.inner.unwrap()))
                .unwrap_or(Ok(Transition::None))
        }
    }

    fn handle_event_inactive(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        event: &winit::event::Event<CustomEvent>,
    ) -> Result<Transition> {
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode,
                                ..
                            },
                        ..
                    },
                ..
            } => match virtual_keycode {
                Some(VirtualKeyCode::P) => {
                    self.console.open = !self.console.open;
                    Ok(Transition::None)
                }
                _ => Ok(Transition::None),
            },
            _ => Ok(Transition::None),
        }
    }

    fn update_inactive(&mut self, world: &mut World, resources: &mut Resources) -> Result<()> {
        for window in &mut self.ui {
            window.update(world, resources);
        }

        Ok(())
    }
}
