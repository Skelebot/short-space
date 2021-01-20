use crate::{
    assets::{
        self,
        settings::{GameSettings, PhysicsSettings},
        AssetLoader,
    },
    graphics::GraphicsShared,
    physics,
    player::{Atlas, Player, PlayerState},
    spacetime::{Child, PhysicsTimer, Position},
    state::State,
};

use super::{game::GameState, Scoped};

pub struct LoadingState {
    done: bool,
}

impl LoadingState {
    pub fn new() -> Self {
        LoadingState { done: false }
    }
}

impl State for LoadingState {
    fn on_start(&mut self, world: &mut legion::World, resources: &mut legion::Resources) {
        // General
        let settings = resources
            .get::<assets::AssetLoader>()
            .unwrap()
            .load::<GameSettings>("settings/game.ron")
            .unwrap();

        resources.insert(settings);

        // Physics
        let p_settings = resources
            .get::<AssetLoader>()
            .unwrap()
            .load::<PhysicsSettings>("settings/physics.ron")
            .unwrap();
        let timer = PhysicsTimer::new(p_settings.step_time);

        resources.insert(p_settings);
        resources.insert(timer);

        // Player
        let pos: Position = na::Isometry3::from_parts(
            na::Translation3::new(0.0, -2.0, 0.1),
            na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), -90.0_f32.to_radians()),
        )
        .into();
        use nc::shape::{Capsule, ShapeHandle};
        let collider = physics::Collider::from(
            // TODO: Load from settings
            ShapeHandle::new(Capsule::new(2.0, 0.4)),
        );
        let vel = physics::Velocity::new(na::Vector3::repeat(0.0_f32), na::Vector3::repeat(0.0));
        let player = Player {
            state: PlayerState::Noclip,
            ground_entity: None,
            flags: 0,
        };
        
        // TODO: The issue with players and cameras
        // There are two ways to handle this:
        // - Every client has a SINGLE camera (in Resources), which, when spectating, simply gets moved
        //   around, and the rendering system always renders from it
        // - Every player in the world has a camera bound to him, which would allow to make a spectator
        //   mode that renders player point-of-views as tiles in a grid, or allow split-screen gameplay
        // The former is easier to use and faster, the latter is more extensible and sometimes unwieldy.
        
        // Scene
        let atlas_cam = {
            let graphics = resources.get::<GraphicsShared>().unwrap();
            let loader = resources.get::<assets::AssetLoader>().unwrap();
            let scope = Scoped {
                id: std::any::TypeId::of::<GameState>(),
            };
            loader
                .load_scene(world, &graphics, "scenes/test.ron", Some(scope))
                .unwrap();

            // Camera
            // Set up the camera
            let size = graphics.window.inner_size();
            let aspect = size.width as f32 / size.height as f32;
            let camera = crate::graphics::Camera::new(aspect, 45_f32.to_radians(), 0.001, 1000.0);
            world.push((pos, camera))
        };

        // Add the player to the world and keep it's Entity (an ID)
        // so we can add it to a Resource to track the single main player
        let atlas = world.push((pos, collider, vel, player));

        world.entry(atlas_cam).unwrap().add_component(Child {
            parent: atlas,
            offset: na::Isometry3::translation(0.0, 0.0, 0.0).into(),
        });

        let atlas = Atlas {
            player: atlas,
            camera: atlas_cam,
        };
        resources.insert(atlas);
        self.done = true;
    }

    fn handle_event(
        &mut self,
        _world: &mut legion::World,
        _resources: &mut legion::Resources,
        _event: winit::event::Event<crate::state::CustomEvent>,
    ) -> crate::state::Transition {
        crate::state::Transition::None
    }

    fn update(
        &mut self,
        _world: &mut legion::World,
        _resources: &mut legion::Resources,
    ) -> crate::state::Transition {
        if self.done {
            crate::state::Transition::Switch(Box::new(GameState::new()))
        } else {
            crate::state::Transition::None
        }
    }
}
