use legion::{Resources, Schedule, World};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use super::loading::LoadingState;
use engine::ui::StartWindow;

use engine::state::{CustomEvent, State, Transition};

pub struct MainState {
    schedule: Schedule,
}

impl MainState {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            //.add_system()
            .build();
        MainState { schedule }
    }
}

impl State for MainState {
    fn on_start(&mut self, _world: &mut World, resources: &mut Resources) {
        resources.insert(StartWindow {
            opened: true,
            play_pressed: false,
            exit_pressed: false,
        })
    }

    fn on_stop(&mut self, _world: &mut World, resources: &mut Resources) {
        resources.remove::<StartWindow>();
    }

    fn on_pause(&mut self, _world: &mut World, resources: &mut Resources) {
        resources.get_mut::<StartWindow>().unwrap().opened = false;
    }

    fn on_resume(&mut self, _world: &mut World, resources: &mut Resources) {
        let mut start = resources.get_mut::<StartWindow>().unwrap();
        start.opened = true;
        start.play_pressed = false;
    }

    fn handle_event(
        &mut self,
        _world: &mut World,
        resources: &mut Resources,
        event: winit::event::Event<CustomEvent>,
    ) -> Transition {
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
                        .unwrap();
                    // Doesn't matter
                    Transition::None
                }
                Some(VirtualKeyCode::Return) => Transition::Push(Box::new(LoadingState::new())),
                _ => Transition::None,
            },
            _ => Transition::None,
        }
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) -> Transition {
        self.schedule.execute(world, resources);
        // Draw UI
        {
            let ctx = resources.get::<egui::CtxRef>().unwrap();
            let response = egui::SidePanel::left("left_panel")
                .resizable(false)
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
            response.inner
        }
    }
}
