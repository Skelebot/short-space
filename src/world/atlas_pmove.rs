use crate::world::atlas::*;
use crate::physics::Physics;
use crate::settings::GameSettings as Settings;
use crate::input::KeyboardInput;
use crate::settings::*;

use nalgebra as na;
extern crate nphysics3d as np;
extern crate ncollide3d as nc;

pub fn pmove (
    atlas: &mut Atlas,
    keyboard_input: &KeyboardInput,
    jump_held: &mut bool,
    physics: &mut Physics,
    _settings: &Settings,
    delta: f32,
) {
    if atlas.state == AtlasState::DEAD { return (); }
    let mut movement = na::Vector3::repeat(0.0);
    if keyboard_input.forward { movement.z -= 1.0; }
    if keyboard_input.backward { movement.z += 1.0; }
    if keyboard_input.left { movement.x -= 1.0; }
    if keyboard_input.right { movement.x += 1.0; }
    if keyboard_input.duck { movement.y -= 1.0; }
    if keyboard_input.jump { movement.y += 1.0; }

    check_duck(atlas, &movement, physics);
    let normal = ground_trace_ray(atlas, physics);
    if movement.y < 1.0 {
        //not holding jump
        *jump_held = false;
    }
    //println!("before: {:?}", atlas.velocity.linear);
    match atlas.state {
        AtlasState::SPECTATOR => (), //TODO: spectator_move(atlas, movement, physics, settings, delta),
        AtlasState::WALKING => walk_move(atlas, &movement, jump_held, physics, normal.unwrap(), delta),
        AtlasState::AIRBORNE => air_move(atlas, &movement, physics, delta),
        AtlasState::DEAD => return,
    }
    //println!("after: {:?}", atlas.velocity.linear);
    //TODO: Animation
    //atlas.animate()
    //TODO: Footstep sounds
    //atlas.footsteps()
    //TODO: Weapons
    //pm_weapon()
    let gravity = na::Vector3::new(0.0, -0.1, 0.0);
    atlas.velocity.linear += gravity;
    //atlas.position.translation.vector += atlas.velocity.linear*0.1;
}

///Handle the movement when player is touching the ground
pub fn walk_move(atlas: &mut Atlas, movement: &na::Vector3<f32>, jump_held: &mut bool, physics: &mut Physics, ground_normal: na::Vector3<f32>, delta: f32) {
    if check_jump(atlas, movement, jump_held) {
        //jumped away
        air_move(atlas, movement, physics, delta);
        return;
    }

    //handle friction
    pm_friction(atlas, delta);

    let mut atl_forward = atlas.position.rotation * movement;
    atl_forward.y = 0.0;

    clip_velocity(&ground_normal, &mut atl_forward, 1.001);

    atl_forward.try_normalize_mut(0.0);

    let mut wishdir = atl_forward.clone();

    let mut wishspeed = wishdir.try_normalize_mut(0.0).unwrap_or(0.0);

    //clamp the speed lower if ducking
    if atlas.ducked {
        wishspeed *= PM_DUCKSCALE;
    }

    pm_accelerate(atlas, wishdir, PM_ACCELERATE * wishspeed, delta);

    let vel = na::Matrix::norm(&atlas.velocity.linear);
    //slide along the ground plane ???
    clip_velocity(&ground_normal, &mut atlas.velocity.linear, 1.001);
    //don't decrease velocity when going up or down a slope
    atlas.velocity.linear.try_normalize_mut(0.0);
    atlas.velocity.linear *= vel;
    //don't do anything if standing still
    if atlas.velocity.linear.x == 0.0 && atlas.velocity.linear.y == 0.0 {
        return;
    }
    //this handles the wall collisions and gravity (?) nphysics does this for us.
    //step_slide_move();
}

///Handle the movement when player is airborne
pub fn air_move(atlas: &mut Atlas, movement: &na::Vector3<f32>, _physics: &mut Physics, delta: f32) {
    //handle friction
    pm_friction(atlas, delta);

    //TODO: legs direction
    let mut atl_forward = atlas.position * movement;
    atl_forward.y = 0.0;

    atl_forward.try_normalize_mut(0.0);

    let wishdir = atl_forward.clone();
    //let mut wishspeed = wishdir.try_normalize_mut(0.0).unwrap_or(0.0);

    pm_accelerate(atlas, wishdir, PM_AIRACCELERATE, delta);
    //we may have a ground plane that is very steep,
    //even though we don't have a groundentity
    //slide along the steep plane
    //FIXME: slide here
}

///Accelerate the player, this is where the strafe-jump bug happens
fn pm_accelerate(atlas: &mut Atlas, wishdir: na::Vector3<f32>, accel: f32, delta: f32) {
    //q2 style
    let currentspeed = na::Matrix::dot(&atlas.velocity.linear, &wishdir);
    let mut addspeed = accel - currentspeed;
    addspeed = max(min(addspeed, MAX_ACCEL * delta), 0.0);
    atlas.velocity.linear += wishdir * addspeed;
}

///Check if the player is ducking or standing up and update camera height and player's hitbox
pub fn check_duck(atlas: &mut Atlas, movement: &na::Vector3<f32>, physics: &mut Physics) {
    if movement.y < 0.0 {
        atlas.ducked = true;
    } else if atlas.ducked {
        //stand up
        //FIXME: check if it is possible to stand up first
        atlas.ducked = false;
    }
    
    if atlas.ducked {
        //FIXME: do it after the physics step (also)
        //TODO: standing up/ducking transition animation?
        atlas.camera.position.translation.vector = 
            atlas.position.translation.vector + na::Vector3::new(0.0, DUCK_HEIGHT, 0.0);
        atlas.set_hitbox_height(VIEW_HEIGHT, physics);

    } else {
        atlas.camera.position.translation.vector =
            atlas.position.translation.vector + na::Vector3::new(0.0, VIEW_HEIGHT, 0.0);
        atlas.set_hitbox_height(DUCK_HEIGHT, physics);
    }
}

pub fn check_jump(atlas: &mut Atlas, movement: &na::Vector3<f32>, jump_held: &mut bool) -> bool {
    if movement.y < 1.0 {
        //not holding jump
        return false;
    }
    //TODO: must wait for jump to be released
    if *jump_held { return false; }
    *jump_held = true;
     
    //jumping away
    atlas.grounded = false;
    atlas.state = AtlasState::AIRBORNE;
    //println!("Jumping");
    atlas.velocity += np::math::Velocity::linear(0.0, JUMP_VELOCITY, 0.0);
    //TODO: jump sound
    //TODO: jump animation
    return true;
}

///Handle player air or walk friction
fn pm_friction(atlas: &mut Atlas, delta: f32) {
    let mut vel = atlas.velocity.as_vector().clone();
    //ignore slope movement ?
    if atlas.state == AtlasState::WALKING { vel.y = 0.0; }
    let speed = vel.norm(); //length of the vector
    //the if(speed < 1) piece, prolly unnecessary as we have a physics engine but still ?
    if speed < 1.0 {
        vel.x = 0.0;
        vel.z = 0.0;
        return;
    }
    let mut drop = 0.0;
    //apply ground friction
    if atlas.state == AtlasState::WALKING {
        //something about being "knocked back"?
        //TODO: na::clamp instead of this
        let control: f32;
        if speed < PM_STOPSPEED {
            control = PM_STOPSPEED;
        } else { control = speed; }

        drop += control * PM_FRICTION * delta;
    }
    //TODO: Spectator friction
    //scale the velocity
    let mut newspeed = speed - drop;
    if newspeed < 0.0 { newspeed = 0.0 }
    newspeed /= speed;
    atlas.velocity = np::math::Velocity::linear(vel.x * newspeed, vel.y * newspeed, vel.z * newspeed);
}

//Can't figure out what is this thing
pub fn clip_velocity(/*in_: &na::Vector3<f32>,*/ normal: &na::Vector3<f32>, out: &mut na::Vector3<f32>, overbounce: f32) {
    let mut backoff = na::Matrix::dot(&out, &normal);   //dot(&in_, ...
    if backoff < 0.0 {
        backoff *= overbounce;
    } else {
        backoff /= overbounce;
    }
    let change: na::Vector3<f32> = normal * backoff;
    *out = *out - change;
    //out = in_ - change;
}

pub fn ground_trace_ray(atlas: &mut Atlas, physics: &mut Physics) -> Option<na::Vector3<f32>> {
    let mut touching_ground = false;
    let mut normal: Option<na::Vector3<f32>> = None;
    let orig = na::Point3::from(atlas.position.translation.vector);
    let dir = na::Vector3::new(0.0, -1.0, 0.0);
    let ray = nc::query::Ray::new(orig, dir);

    for collider in physics.colliders.iter() {
        let shape = collider.1.shape();
        let intersection = shape.as_ray_cast().expect("Shape does not implement RayCast")
            .toi_and_normal_with_ray(&na::Isometry3::identity(), &ray, false);
        if let Some(intersec) = intersection {
            touching_ground = true;
            normal = Some(intersec.normal);
            println!("normal: {}", intersec.normal.y);
            if intersec.normal.y < MIN_WALK_NORMAL || intersec.toi > 1.0 {
                atlas.grounded = false;
                atlas.state = AtlasState::AIRBORNE;
                return None
            }
        }
    }
    if !touching_ground {
        atlas.grounded = false;
        atlas.state = AtlasState::AIRBORNE;
    }
    atlas.grounded = true;
    atlas.state = AtlasState::WALKING;
    return normal;
}

///Check if the player is standing on the ground and change player's state accordingly.
///Makes the player slide off steep ground (slopes) and crash_land() if the player just impacted
///the ground.
///Returns the ground plane's normal or None
pub fn ground_trace(atlas: &mut Atlas, physics: &mut Physics) -> Option<na::Vector3<f32>>{
    let mut touching_ground = false;
    let mut normal: Option<na::Vector3<f32>> = None;
    let contacts = physics.geom_world.contacts_with(&physics.colliders, atlas.cl_handle, true);
    if let Some(contacts_some) = contacts {
        //If there are contacts with the player body
        for contact in contacts_some {
            //contact is (CLhandle, Collider, CLhandle, Collider, ContactAlghoritm, ContactManifold)
            //println!("{:?}", physics.get_body(contact.3.body()).unwrap().is_static());
            if true {//physics.get_body(contact.3.body()).unwrap().is_static() {
                touching_ground = true;
                //atlas is touching the ground, check the deepest contact's normal
                //to get the steepiness of the contacting face
                normal = Some(contact.5.deepest_contact()
                              .expect("Something went wrong when retrieving deepest contact")
                              .contact.normal.as_ref().clone()*-1.0);
                //println!("Touching ground with normal: {:?}", normal);
                //If the plane is too steep
                if normal.unwrap().y < MIN_WALK_NORMAL {
                    //slide off the slope
                    atlas.grounded = false;
                    atlas.state = AtlasState::AIRBORNE;
                    //normal = None;
                    return None;
                }
            }
        }
    }
    //Free fall
    if !touching_ground {
        //We just transitioned into freefall, start jumping animation
        //if player.grounded {}
        atlas.grounded = false;
        atlas.state = AtlasState::AIRBORNE;
        return None;
    }
    //If the player is on ground
    //if touching_ground {
    //But we are already sure if we didn't return yet
    //We just landed, apply damage if the velocity was too high, play OOF sound
    //if !atlas.grounded {}
    //TODO: landing time?
    //don't do landing time if we were just going down a slope
    atlas.grounded = true;
    atlas.state = AtlasState::WALKING;
    return normal;
    //TODO: landing sound
}

//Because float does not impl Ord
//I can't use std::min and std::max
//so to avoid excessive crates
//i wrote those two here.
#[allow(dead_code)]
fn min(a: f32, b: f32) -> f32 {
    if a < b { return a; }
    else { return b; }
}

#[allow(dead_code)]
fn max(a: f32, b: f32) -> f32 {
    if a > b { return a; }
    else { return b; }
}

#[allow(dead_code)]
fn abs(a: f32) -> f32 {
    if a < 0.0 { return a * -1.0 }
    else { return a; }
}
