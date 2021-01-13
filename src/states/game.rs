use std::any::TypeId;

use crate::{
    assets::settings::{GameSettings, PhysicsSettings},
    graphics::{self, Graphics},
    player::Atlas,
    spacetime::PhysicsTimer,
    state::*,
};
use eyre::{eyre::eyre, Result};
use graphics::{GraphicsShared, MeshPassEnable};
use legion::{Entity, Resources, Schedule, World};
use winit::{
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::EventLoopProxy,
};

use super::Scoped;

pub struct GameState {
    schedule: Schedule,
    paused: bool,
}

impl GameState {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            .add_system(crate::physics::step_system())
            .add_system(crate::player::player_movement_system())
            .add_system(crate::physics::lerp_system())
            .add_system(crate::physics::children_update_system())
            .build();
        GameState {
            schedule,
            paused: false,
        }
    }
}

impl State for GameState {
    fn on_start(&mut self, _world: &mut World, resources: &mut Resources) {
        resources.insert(MeshPassEnable);
        let graphics = resources.get::<GraphicsShared>().unwrap();
        graphics.window.set_cursor_grab(true).unwrap();
        graphics.window.set_cursor_visible(false);
    }

    fn on_stop(&mut self, world: &mut legion::World, resources: &mut legion::Resources) {
        resources.remove::<MeshPassEnable>();
        resources.remove::<GameSettings>();
        resources.remove::<PhysicsSettings>();
        resources.remove::<PhysicsTimer>();
        resources.remove::<Atlas>();

        use legion::IntoQuery;
        let mut query = <(Entity, &Scoped)>::query();
        let to_remove: Vec<Entity> = query
            .iter(world)
            .filter(|(_, scope)| scope.id == TypeId::of::<Self>())
            .map(|(e, _)| *e)
            .collect();
        to_remove.into_iter().for_each(|e| {
            world.remove(e);
        });
        let graphics = resources.get::<GraphicsShared>().unwrap();
        graphics.window.set_cursor_grab(false).unwrap();
        graphics.window.set_cursor_visible(true);
    }

    fn on_pause(&mut self, _world: &mut World, _resources: &mut Resources) {}

    fn on_resume(&mut self, _world: &mut World, _resources: &mut Resources) {}

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
                                virtual_keycode: Some(code),
                                ..
                            },
                        ..
                    },
                ..
            } => match code {
                VirtualKeyCode::M => {
                    let proxy = resources.get::<EventLoopProxy<CustomEvent>>().unwrap();
                    proxy.send_event(CustomEvent::UngrabCursor).unwrap();
                    Transition::Pop
                }
                VirtualKeyCode::Escape => {
                    // Pause the game
                    Transition::None
                }
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
