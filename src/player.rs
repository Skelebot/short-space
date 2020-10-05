pub struct Atlas { pub entity: Entity }
pub enum PlayerState {
    Playing,
    Spectator,
    // Dead, etc
}
pub enum MovementState {
    Grounded,
    Airborne,
}
pub struct Player {
    pub state: PlayerState,
    pub movement_state: MovementState,
}

use legion::{Entity, system, world::SubWorld, IntoQuery, EntityStore};
use crate::{GameState, GameSettings, time::Time};
use crate::graphics::Camera;
use crate::physics::*;
use crate::input::InputState;

#[system]
#[read_component(Player)]
#[write_component(Position)]
#[write_component(Velocity)]
pub fn player_movement(
    #[resource] atlas: &Atlas,
    #[resource] input_state: &InputState,
    #[resource] game_state: &GameState,
    #[resource] _physics_settings: &PhysicsSettings,
    #[resource] game_settings: &GameSettings,
    #[resource] camera: &mut Camera,
    #[resource] time: &mut Time,
    world: &mut SubWorld,
) {
    if game_state.paused { return; }
    // Get the all components belonging to the player-controlled player (here called Atlas)
    let atlas_components = world.entry_mut(atlas.entity).unwrap();
    let player = atlas_components.get_component::<Player>().unwrap();

    match player.state {
        PlayerState::Spectator => {
            let speed = time.delta * game_settings.movement_speed;
            // Move only the camera
            // TODO: Refactor
            if input_state.forward {
                // Move in the z direction (camera pointing forward; depth)
                let vector = camera.position.rotation * na::Vector::x();
                // Ignore the pitch (rotation along the x axis),
                // then renormalize to discard the effect of ignoring the pitch
                camera.position.translation.vector += vector * speed;
            }
            if input_state.right {
                let vector = camera.position.rotation * na::Vector::y();
                camera.position.translation.vector += vector * speed;
            }
            if input_state.left {
                let vector = camera.position.rotation * -na::Vector::y();
                camera.position.translation.vector += vector * speed;
            }
            if input_state.backward {
                let vector = camera.position.rotation * -na::Vector::x();
                camera.position.translation.vector += vector * speed;
            }
        },
        PlayerState::Playing => {
            match player.movement_state {
                MovementState::Airborne => {
                    // Apply gravity
                    //let mut position = atlas_components.get_component_mut::<Position>().unwrap();
                    //position += physics_settings.gravity 
                    // Apply air friction
                    // Collisions with walls
                },
                MovementState::Grounded => {
                    // Walk
                    // Collisions with walls
                },
            }
            // Synchronize the camera's position with the player's
            let position = atlas_components.get_component::<Position>().unwrap();
            // Double deref: first to strip the borrow, second to actually get the Isometry3
            // TODO: Add player height
            camera.position = **position;
        }
    }





}
