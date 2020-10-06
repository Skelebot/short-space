pub struct Atlas { pub entity: Entity }
#[derive(PartialEq, Eq, Debug)]
pub enum PlayerState {
    Normal,     // Can accelerate and turn
    Noclip,     // noclip movement
    Spectator,  // still run into walls
    Dead,       // no acceleration or turning, but still free falling
    Freeze,     // stuck in place
}
#[derive(PartialEq, Eq, Debug)]
pub enum MovementState {
    Grounded,
    Airborne,
}
// Player flags:
const PF_DUCKED: u16 = 1;
const PF_JUMP_HELD: u16 = 2;
pub struct Player {
    pub state: PlayerState,
    pub movement_state: MovementState,
    pub flags: u16,
}

use legion::{Entity, system, world::SubWorld, IntoQuery, EntityStore};
use crate::{GameState, GameSettings, time::Time};
use crate::graphics::Camera;
use crate::physics::*;
use crate::input::InputState;

#[system(for_each)]
pub fn player_movement(
    #[resource] atlas: &Atlas,
    #[resource] input_state: &InputState,
    #[resource] game_state: &GameState,
    #[resource] _physics_settings: &PhysicsSettings,
    #[resource] game_settings: &GameSettings,
    #[resource] camera: &mut Camera,
    #[resource] time: &mut Time,
    entity: &Entity,
    player: &mut Player,
    position: &mut Position,
    velocity: &mut Velocity,
    collider: &Collider,
) {
    if game_state.paused { return; }
    // Get the all components belonging to the player-controlled player (here called Atlas)
    // For now, we only care about the Atlas player
    if *entity != atlas.entity { return; }

    // Save old origin in case we get stuck
    // Save old velocity for crashlanding

    match player.state {
        // Clear flags etc
        // Technically, Dead shouldn't return, because corpses still fly because of their velocity
        // TODO: Make corpses fly
        PlayerState::Dead => {},
        PlayerState::Spectator => {
            // PM_CheckDuck();
            check_duck(input_state, player);
            // PM_FlyMove();
            //  PM_Friction();
            apply_friction(&player, velocity, time);
            //  
            // PM_DropTimers();
            // return;

            // Move only the camera
            //let speed = time.delta * game_settings.movement_speed;
            //if input_state.forward {
            //    // Move in the z direction (camera pointing forward; depth)
            //    let vector = camera.position.rotation * na::Vector::x();
            //    camera.position.translation.vector += vector * speed;
            //}
            //if input_state.right {
            //    let vector = camera.position.rotation * na::Vector::y();
            //    camera.position.translation.vector += vector * speed;
            //}
            //if input_state.left {
            //    let vector = camera.position.rotation * -na::Vector::y();
            //    camera.position.translation.vector += vector * speed;
            //}
            //if input_state.backward {
            //    let vector = camera.position.rotation * -na::Vector::x();
            //    camera.position.translation.vector += vector * speed;
            //}
        },
        PlayerState::Noclip => {
            // PM_NoclipMove();
            // PM_DropTimers();
        },
        // No movement at all
        PlayerState::Freeze => {},
        PlayerState::Normal => {
            // PM_CheckDuck();
            // PM_GroundTrace();
            // PM_DropTimers();
            match player.movement_state {
                MovementState::Grounded => {
                    // PM_WalkMove();
                },
                MovementState::Airborne => {
                    // PM_AirMove();
                },
            }
            // PM_GroundTrace();
            // PM_Weapon();
            // PM_Footsteps();
            // Synchronize the camera's position with the player's
            //let position = atlas_components.get_component::<Position>().unwrap();
            // Double deref: first to strip the borrow, second to actually get the Isometry3
            // TODO: Add player height
            //camera.position = **position;
        }
    }
}

fn check_duck(input_state: &InputState, player: &mut Player) {
    if input_state.upmove < 0.0 {
        // The player is ducking
        player.flags |= PF_DUCKED;
    } else {
        // If the player is ducked, try to stand up
        if player.flags & PF_DUCKED != 0 {
            // TODO: Check for collisions when standing up
            // Trace a ray from above the player's head
            // trace = trace(...)
            // if !trace.allsolid {...}
            player.flags &= !PF_DUCKED;
        }
    }
    if player.flags & PF_DUCKED != 0 {
        // TODO: Set the view height to DUCKED_VIEWHEIGHT
    } else {
        // TODO: Set the view height to DEFAULT_VIEWHEIGHT
    }
}

fn apply_friction(player: &Player, velocity: &mut Velocity, time: &Time) {
    // if pml.walking {vec[2] = 0}
    let speed = velocity.linear.magnitude();
    let mut drop = 0.0;
    // Apply ground friction
    if player.movement_state == MovementState::Grounded {
        // TODO: if getting knocked back, no friction
        let control = speed.min(STOPSPEED);
        drop += control*FRICTION*time.delta;
    }
    // Apply flying friction
    if player.state == PlayerState::Spectator {
        drop += speed*SPECTATOR_FRICTION*time.delta;
    }

    // scale the velocity
    let newspeed = (speed - drop).min(0.0) / speed;
    velocity.linear.x = velocity.linear.x * newspeed;
    velocity.linear.y = velocity.linear.y * newspeed;
    velocity.linear.z = velocity.linear.z * newspeed;
}


const STOPSPEED: f32 = 100.0;
const DUCKSCALE: f32 = 0.25;
//const SWIMSCALE: f32 = 100.0;

const ACCELERATE: f32 = 10.0;
const AIR_ACCELERATE: f32 = 1.0;
const FLY_ACCELERATE: f32 = 4.0;

const FRICTION: f32 = 6.0;
const AIR_FRICTION: f32 = 3.0;
const SPECTATOR_FRICTION: f32 = 5.0;