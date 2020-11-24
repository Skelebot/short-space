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
    pub ground_entity: Option<Entity>,
    pub flags: u16,
}

use legion::{Entity, system, world::SubWorld, IntoQuery, EntityStore};
use crate::{GameState, GameSettings, time::Time};
use crate::graphics::Camera;
use crate::physics::*;
use crate::input::InputState;

#[system(for_each)]
#[read_component(Collider)]
#[read_component(Entity)]
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
    //collider: &Collider,
    world: &mut SubWorld,
) {
    if game_state.paused { return; }
    // Get the all components belonging to the player-controlled player (here called Atlas)
    // For now, we only care about the Atlas player
    if *entity != atlas.entity { return; }

    player.ground_entity = None;
    // Reduce all timers connected with moving

    // Categorize position
    {
        // See if the player is standing on something solid
        let mut test_vec = position.translation.vector.clone();

        const GROUND_TEST_OFFSET: f32 = 6.20;
        test_vec.z -= GROUND_TEST_OFFSET;

        const JUMP_VELOCITY: f32 = 0.05;

        let moving_up = velocity.linear.z > 0.0;
        let moving_up_rapidly = velocity.linear.z > JUMP_VELOCITY;
        // Was standing on ground, but now am not
        if moving_up_rapidly {
            player.ground_entity = None;
        } else {
            // Try and move down
            // Try to touch ground
            let ray = nc::query::Ray::new(position.translation.vector.into(), test_vec.normalize());
            // TODO: Intersect the ray with everything the player can stand on; not only the map entity
            let mut query = <(&Entity, &Collider)>::query();
            for (entity, collider) in query.iter(world) {
                use nc::query::RayCast;
                let intersection = collider.toi_and_normal_with_ray(
                    &na::Isometry3::identity(),
                    &ray,
                    GROUND_TEST_OFFSET,
                    // This is the solid flag. If the ray is inside a shape and it's true, then the TOI will be set to 0
                    // and normal will be undefined. If the ray is inside a shape and it's false, then it will act as if
                    // the shape is hollow, calculating the normal and toi to it's outer wall.
                    // REVIEW: Solid ray traces are *much* faster than non-solid, this is set to false to make development
                    // easier. Consider changing it to true
                    false
                );
                debug!("hit: {}", intersection.is_some());
                // Was on ground, but now suddenly i'm not
                if let Some(hit) = intersection {
                    // If we hit a steep plane, we are not on ground
                    if hit.normal.z < 0.7 {
                        // TODO: Test four sub-boxes, to see if any of them would have found shallower slope we could stand on (TryTouchGroundInQuadrants)
                    } else {
                        player.ground_entity = Some(*entity);
                    }
                }
            }
        }
    } // Categorize position

    // TODO: Check for ducking
    // check_duck();

    // Finally, handle movement modes

    match player.state {
        // Clear flags etc

        // Technically, Dead shouldn't return, because corpses still fly because of their velocity
        // TODO: Make corpses fly
        // In Source, Dead isn't even an option (REVIEW: Remove?)
        PlayerState::Dead => {},
        PlayerState::Spectator => {
            //check_duck(input_state, player);
            //apply_friction(&player, velocity, time);
            //  wishvel
            let forward = **position * na::Vector3::x();
            let right = **position * na::Vector3::y();
            let scale = 1.0;

            let mut wishvel = forward * scale + right * scale;

            wishvel.z += scale * input_state.upmove;

            let wishdir = wishvel.clone();
            let wishspeed = wishdir.norm();
            //  PM_Accelerate();
            // q2 style
            let currentspeed = na::Vector3::dot(&velocity.linear, &wishdir);
            let addspeed = wishspeed - currentspeed;
            if addspeed > 0.0 {
                let accelspeed = ACCELERATE * time.delta * wishspeed;
                velocity.linear += wishdir * accelspeed;
            }
            position.translation.vector += velocity.linear;
            camera.position = **position;
            // PM_StepSlideMove();

            // PM_DropTimers();
            // return;
        },
        PlayerState::Noclip => {

            //let fmove = input_state.fwdmove * game_settings.noclip_speed;
            //let smove = input_state.sidemove * game_settings.noclip_speed;

            //let forward = (position.rotation * na::Vector3::x()).normalize() * fmove;
            //// TODO: negate?
            //let side = (position.rotation * na::Vector3::y()).normalize() * smove;
            
            //let mut wishvel = forward + side;
            //wishvel.z += input_state.upmove * game_settings.noclip_speed;

            //let wishdir = wishvel.clone();
            //let mut wishspeed = wishdir.norm();

            //// Clamp to max speed
            //if wishspeed > MAXSPEED {
            //    // Scale the velocity
            //    wishvel *= MAXSPEED/wishspeed;
            //    wishspeed = MAXSPEED;
            //}

            // Accelerate
            // This is where the bunnyhop bug occurs
            {
                let wishdir = na::Vector3::new(
                    input_state.sidemove,
                    input_state.fwdmove,
                    input_state.upmove,
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
                position.rotation = camera.position.rotation;
                velocity.linear = position.rotation * wishdir * accelspeed;

            } // Accelerate

            // Bleeding off speed(?)

            // Just move
            position.translation.vector += velocity.linear * time.delta;

            // Sync camera pos to player pos
            camera.position = **position;

            debug!("pos: {}", **position);
            // PM_NoclipMove();
            // PM_DropTimers();
            // Move only the camera
            //let speed = time.delta * game_settings.noclip_speed;
            //println!("upmove: {:?}", input_state.upmove);
            //let direction = 
            //camera.position
            //* na::Vector3::new(
            //    input_state.fwdmove,
            //    -input_state.sidemove,
            //    input_state.upmove,
            //);
            //println!("dir: {:?}", direction);
            //camera.position.translation.vector += direction * speed;
        },
        // No movement at all
        PlayerState::Freeze => {},
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
    if speed < 0.001 { return; }

    let mut drop = 0.0;
    // Apply ground friction
    if player.ground_entity.is_some() {
        // We could do speed.min(STOPSPEED) here, but this is much more descriptive
        let control = if speed < STOPSPEED { speed } else { STOPSPEED };
        drop += control * FRICTION * time.delta;
    }

    let mut newspeed = speed - drop;
    if newspeed < 0.0 { newspeed = 0.0 }
    if newspeed != speed {
        // Determine proportion of old speed we are using
        newspeed /= speed;
        // Scale velocity according to proportion
        velocity.linear *= newspeed;
    }
}


// TODO_E: Move all to GameSettings (same as noclip_speed)
const STOPSPEED: f32 = 100.0;
const DUCKSCALE: f32 = 0.25;
//const SWIMSCALE: f32 = 100.0;

//const ACCELERATE: f32 = 10.0;
const ACCELERATE: f32 = 1.00;
const AIR_ACCELERATE: f32 = 1.0;
const FLY_ACCELERATE: f32 = 4.0;
//const NOCLIP_ACCELERATE: f32 = 4.0;
const NOCLIP_ACCELERATE: f32 = 4.00;

//const FRICTION: f32 = 6.0;
const FRICTION: f32 = 0.06;
const AIR_FRICTION: f32 = 3.0;
const SPECTATOR_FRICTION: f32 = 5.0;

//const MAXSPEED: f32 = 10.0;
const MAXSPEED: f32 = 0.10;