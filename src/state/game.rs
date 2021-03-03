use std::any::TypeId;

use crate::{
    assets::settings::{GameSettings, PhysicsSettings},
    graphics,
    player::Atlas,
    spacetime::PhysicsTimer,
    state::*,
};
use graphics::{color::Rgba, debug_pass::DebugLines, GraphicsShared, MeshPassEnable};
use legion::{Entity, Resources, Schedule, World};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use super::Scoped;

pub struct GameState {
    schedule: Schedule,
}

impl GameState {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            .add_system(crate::physics::step_system())
            .add_system(crate::player::player_movement_system())
            .add_system(crate::physics::children_update_system())
            .build();
        GameState { schedule }
    }
}

impl State for GameState {
    fn on_start(&mut self, _world: &mut World, resources: &mut Resources) {
        resources.insert(MeshPassEnable);
        // Insert debug lines
        {
            let mut debug_lines = DebugLines::new();
            debug_lines.thickness = 3.0;
            debug_lines.push_line(
                na::Vector3::repeat(0.0),
                na::Vector3::new(0.0, 0.0, 10.0),
                Rgba::new(0.0, 0.0, 1.0, 1.0),
            );
            debug_lines.push_line(
                na::Vector3::repeat(0.0),
                na::Vector3::new(10.0, 0.0, 0.0),
                Rgba::new(1.0, 0.0, 0.0, 1.0),
            );
            debug_lines.push_line(
                na::Vector3::repeat(0.0),
                na::Vector3::new(0.0, 10.0, 0.0),
                Rgba::new(0.0, 1.0, 0.0, 1.0),
            );
            debug_lines.push_line_gradient(
                na::Vector3::repeat(0.0),
                na::Vector3::repeat(10.0),
                Rgba::new(1.0, 0.0, 1.0, 1.0),
                Rgba::new(0.0, 1.0, 1.0, 1.0),
            );
            resources.insert(debug_lines);
        }
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

    fn handle_event(
        &mut self,
        _world: &mut World,
        _resources: &mut Resources,
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
                VirtualKeyCode::Back => Transition::Pop,
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
}
