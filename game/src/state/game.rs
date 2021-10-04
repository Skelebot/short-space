use std::any::TypeId;

use crate::{
    spacetime::PhysicsTimer,
    settings::*,
};
use engine::graphics::{color::Rgba, debug::DebugLines, GraphicsShared};
use legion::{Entity, Resources, Schedule, World};
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};

use engine::state::*;

pub struct GameState {
    cursor_grabbed: bool,
    schedule: Schedule,
}

impl GameState {
    pub fn new() -> Self {
        let schedule = Schedule::builder()
            .add_system(engine::physics::step_system())
            .add_system(crate::player::player_movement_system())
            .add_system(crate::player::camera_sync_system())
            .add_system(engine::physics::children_update_system())
            .build();
        GameState {
            schedule,
            cursor_grabbed: false,
        }
    }
}

impl State for GameState {
    fn on_start(&mut self, _world: &mut World, resources: &mut Resources) {
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
            resources.insert(debug_lines);
        }
        resources.insert(engine::ui::FPSWindow {
            opened: true,
            fps: 0.0,
        });
        let graphics = resources.get::<GraphicsShared>().unwrap();
        graphics.window.set_cursor_grab(true).unwrap();
        graphics.window.set_cursor_visible(false);
        self.cursor_grabbed = true;
    }

    fn on_stop(&mut self, world: &mut legion::World, resources: &mut legion::Resources) {
        resources.remove::<GameSettings>();
        resources.remove::<PhysicsSettings>();
        resources.remove::<PhysicsTimer>();
        //resources.remove::<Atlas>();

        use legion::IntoQuery;
        let mut query = <(Entity, &Scoped)>::query();

        #[allow(clippy::needless_collect)]
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
                VirtualKeyCode::Back => Transition::Pop,
                VirtualKeyCode::Escape => {
                    let graphics = resources.get::<GraphicsShared>().unwrap();
                    graphics
                        .window
                        .set_cursor_grab(!self.cursor_grabbed)
                        .unwrap();
                    graphics.window.set_cursor_visible(self.cursor_grabbed);
                    self.cursor_grabbed = !self.cursor_grabbed;
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
