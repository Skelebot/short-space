use engine::graphics::{Camera, MainCamera};

use crate::{
    player::{Player, PlayerState},
    settings::{GameSettings, PhysicsSettings},
    spacetime::{PhysicsTimer, Position},
};

use engine::{
    graphics, physics,
    state::{CustomEvent, Scoped, State, Transition},
};

use super::game::GameState;

pub struct LoadingState {
    done: bool,
}

impl LoadingState {
    pub fn new() -> Self {
        LoadingState { done: false }
    }
}

impl State for LoadingState {
    fn on_start(&mut self, _world: &mut legion::World, _resources: &mut legion::Resources) {}

    fn handle_event(
        &mut self,
        _world: &mut legion::World,
        _resources: &mut legion::Resources,
        _event: winit::event::Event<CustomEvent>,
    ) -> Transition {
        Transition::None
    }

    fn update(
        &mut self,
        world: &mut legion::World,
        resources: &mut legion::Resources,
    ) -> Transition {
        self.continue_loading(world, resources);

        if self.done {
            Transition::Switch(Box::new(GameState::new()))
        } else {
            Transition::None
        }
    }
}

impl LoadingState {
    fn continue_loading(&mut self, world: &mut legion::World, resources: &mut legion::Resources) {
        // Load settings
        let (settings, p_settings) = {
            let asset_loader = resources.get::<engine::assets::AssetLoader>().unwrap();

            let settings = asset_loader
                .load::<GameSettings>("settings/game.ron")
                .unwrap();
            let p_settings = asset_loader
                .load::<PhysicsSettings>("settings/physics.ron")
                .unwrap();

            (settings, p_settings)
        };
        resources.insert(settings);

        let timer = PhysicsTimer::new(p_settings.step_time);
        resources.insert(timer);
        resources.insert(p_settings);

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
            look_pitch: 0.0,
        };

        // Scene
        let camera = {
            let graphics = resources.get::<graphics::GraphicsShared>().unwrap();
            let loader = resources.get::<engine::assets::AssetLoader>().unwrap();
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
            Camera::new(aspect, 45_f32.to_radians(), 0.001, 1000.0)
        };
        // TODO: Maybe move to GameState
        let main_camera = MainCamera {
            camera,
            position: na::Isometry3::identity().into(),
        };
        resources.insert(main_camera);

        // Add the player to the world and keep it's Entity (an ID)
        // so we can add it to a Resource to track the single main player
        let atlas = world.push((pos, collider, vel, player));

        let players: crate::player::Players = vec![atlas];
        resources.insert(players);

        //resources.insert(atlas);
        self.done = true;
    }
}
