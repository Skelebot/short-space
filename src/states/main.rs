use crate::state::*;
use legion::{Resources, Schedule, World};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use super::{game::GameState, loading::LoadingState};

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
    fn on_start(&mut self, _world: &mut World, _resources: &mut Resources) {}

    fn on_stop(&mut self, _world: &mut World, _resources: &mut Resources) {}

    fn on_pause(&mut self, _world: &mut World, _resources: &mut Resources) {}

    fn on_resume(&mut self, _world: &mut World, _resources: &mut Resources) {}

    fn handle_event(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
        event: winit::event::Event<CustomEvent>,
    ) -> Transition {
        match event {
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => match input {
                KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some(VirtualKeyCode::N),
                    ..
                } => Transition::Push(Box::new(LoadingState::new())),
                _ => Transition::None,
            },
            _ => Transition::None,
        }
    }

    fn update(&mut self, world: &mut World, resources: &mut Resources) -> Transition {
        self.schedule.execute(world, resources);
        Transition::None
    }

    fn update_inactive(&mut self, _world: &mut World, _resources: &mut Resources) {}
}
