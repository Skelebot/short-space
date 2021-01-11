#![allow(dead_code)]
#[derive(Debug)]
pub struct Atlas {
    pub player: Entity,
    pub camera: Entity,
}
#[derive(PartialEq, Eq, Debug)]
pub enum PlayerState {
    Normal,    // Can accelerate and turn
    Noclip,    // noclip movement
    Spectator, // still run into walls
    Dead,      // no acceleration or turning, but still free falling
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
    pub ground_entity: Option<Entity>,
    pub flags: u16,
}

use crate::assets::settings::PhysicsSettings;
use crate::input::{self, InputState};
use crate::physics::*;
use crate::{
    spacetime::{Position, Time},
    GameSettings, GameState,
};
use legion::{system, world::SubWorld, Entity, IntoQuery};

#[system]
#[read_component(Entity)]
#[read_component(Collider)]
#[write_component(Player)]
#[write_component(Position)]
#[write_component(Velocity)]
pub fn player_movement(
    #[resource] _physics_settings: &PhysicsSettings,
    #[resource] atlas: &Atlas,
    #[resource] input_state: &InputState,
    #[resource] game_state: &GameState,
    #[resource] game_settings: &GameSettings,
    #[resource] time: &mut Time,
    world: &mut SubWorld,
) {
    if game_state.paused {
        return;
    }
    
    let mut player_query = <(&mut Player, &mut Position, &mut Velocity)>::query();
    let (player, position, velocity) = player_query.get_mut(world, atlas.player).unwrap();

    // Rotate the player
    let offset: na::Vector2::<f32> = input_state.mouse_delta * game_settings.mouse_sensitivity * time.delta as f32;

    // TODO: Append rotations directly instead of creating new quaternions
    let zrot = na::UnitQuaternion::from_axis_angle(&na::Vector3::z_axis(), offset.x);
    let xrot = na::UnitQuaternion::from_axis_angle(&na::Vector3::x_axis(), offset.y);

    // Note: By changing the order of multiplications here, we can make the camera
    // do all rotations around it's own relative axes (including the z axis),
    // which would make it a full 3D-space camera. This actually isn't good
    // in FPS games, where the player never has to "roll" the camera
    // (rotate around relative x) To fix this, we rotate around the z axis last,
    // so it's always the world's absolute z axis.
    // To make a space-type camera, the z rotation should be performed first.
    // Note 2: When multiplying transformations, the order is actually done backwards
    // (xrot is the first rotation performed, because it's the last one in the multiplication)

    position.future_mut().rotation = zrot * position.future_mut().rotation * xrot;

    // Finally, handle movement modes
    match player.state {
        // Clear flags etc

        // Technically, Dead shouldn't return, because corpses still fly because of their velocity
        // TODO: Make corpses fly
        // In Source, Dead isn't even an option (REVIEW: Remove?)
        PlayerState::Dead => {}
        PlayerState::Spectator => {}
        /*PlayerState::Spectator => {
            //check_duck(input_state, player);
            //apply_friction(&player, velocity, time);
            //  wishvel
            let forward = *position * na::Vector3::x();
            let right = *position * na::Vector3::y();
            let scale = 1.0;

            let mut wishvel = forward * scale + right * scale;

            wishvel.z += scale * input_state.get_axis_state(&input::UP_AXIS);

            let wishdir = wishvel;
            let wishspeed = wishdir.norm();
            let currentspeed = na::Vector3::dot(&velocity.linear, &wishdir);
            let addspeed = wishspeed - currentspeed;
            let accel = 10.0;
            if addspeed > 0.0 {
                let accelspeed = accel * time.delta as f32 * wishspeed;
                velocity.linear += wishdir * accelspeed;
            }
            position.translation.vector += velocity.linear;
            //return;
        }*/
        PlayerState::Noclip => {
            // Accelerate
            {
                let wishdir = na::Vector3::new(
                    input_state.get_axis_state(&input::SIDE_AXIS),
                    input_state.get_axis_state(&input::FWD_AXIS),
                    input_state.get_axis_state(&input::UP_AXIS),
                    //).normalize();
                );
                //let current_speed = velocity.linear.dot(&wishdir);

                //let wishspeed = 0.0;
                //// Reduce wishspeed by the amount of veer
                //let addspeed = wishspeed - current_speed;

                //// If not going to add any speed, done
                //let mut accelspeed = 0.0;
                //if addspeed > 0.0 {
                //    accelspeed = NOCLIP_ACCELERATE * time.delta * wishspeed * FRICTION;
                //}

                //// Cap at addspeed
                //if accelspeed > addspeed { accelspeed = addspeed }

                // Finally, adjust velocity
                let accelspeed = 3.0;
                velocity.linear = position.future_mut().rotation * wishdir * accelspeed;
                if input_state.is_action_pressed(&input::SPRINT_ACTION) {
                    velocity.linear *= game_settings.sprint_multiplier;
                }
            } // Accelerate

            // Bleeding off speed(?)

            // Just move
            position.future_mut().translation.vector += velocity.linear * time.delta as f32;

            // PM_NoclipMove();
            // PM_DropTimers();
            // Move only the camera
            //let speed = time.delta * game_settings.noclip_speed;
            //let direction =
            //camera.position
            //* na::Vector3::new(
            //    input_state.fwdmove,
            //    -input_state.sidemove,
            //    input_state.upmove,
            //);
            //camera.position.translation.vector += direction * speed;
        }
        // No movement at all
        PlayerState::Normal => {
            // PM_CheckDuck();
            // PM_GroundTrace();
            // PM_DropTimers();
            // PM_GroundTrace();
            // PM_Weapon();
            // PM_Footsteps();
            // Synchronize the camera's position with the player's
            //let position = atlas_components.get_component::<Position>().unwrap();
            // Double deref: first to strip the borrow, second to actually get the Isometry3
            // TODO: Add player height
        }
    }
}
