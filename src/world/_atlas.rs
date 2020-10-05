use nalgebra as na;
//use crate::physics::{Physics, Error};
//use super::entity::Entity;
//use crate::camera::{FpsCamera};

//Some variants are never constructed
//TODO: remove after adding spectator
#[allow(dead_code)]
#[derive(PartialEq)]
pub enum AtlasState {
    WALKING,
    AIRBORNE,
    DEAD,
    SPECTATOR,
}

///Main player character singleton
pub struct Atlas {
    pub position: na::Isometry3<f32>,
    pub velocity: na::Vector3<f32>,
    //pub rb_handle: np::object::DefaultBodyHandle,
    //pub cl_handle: np::object::DefaultColliderHandle,
    pub grounded: bool,
    pub ducked: bool,
    //pub camera: FpsCamera,
    pub state: AtlasState,
}

impl Atlas {
    #[allow(dead_code)]
    pub fn new(/*physics: &mut Physics,*/ pos: na::Isometry3<f32>, /*camera: FpsCamera*/) -> Self {
        //FIXME: Make it use setting's view height
        //TODO: Use different hitbox shape
        //let atl_shape = nc::shape::ShapeHandle::new(
        //    nc::shape::Cuboid::new(
        //        na::Vector3::new(0.25, 0.5, 0.25)));
        //let atl_rb_desc = np::object::RigidBodyDesc::new()
        //    .position(pos).mass(1.0)
        //    //.linear_damping(6.0)  //we do the damping by ourselves
        //    .kinematic_rotations(na::Vector3::new(true, true, true))
        //    .kinematic_translations(na::Vector3::new(false, true, false));
        //let atl_rb_handle = physics.add_rigid_body(atl_rb_desc.build());
        //let atl_collider = np::object::ColliderDesc::new(atl_shape)
        //    .density(1.0)
        //    .material(np::material::MaterialHandle::new(
        //            np::material::BasicMaterial::new(0.0, 0.0)))
        //    .build(np::object::BodyPartHandle(atl_rb_handle.clone(), 0));
        //let atl_cl_handle = physics.add_collider(atl_collider);
        Atlas {
            position: pos,
            velocity: na::Vector3::repeat(0.0),
            //rb_handle: atl_rb_handle,
            //cl_handle: atl_cl_handle,
            grounded: false,
            ducked: false,
            //camera: camera,
            state: AtlasState::WALKING,
        }
    }
    //pub fn set_hitbox_height(&mut self, height: f32, physics: &mut Physics) {
    //    let collider = physics.get_collider_mut(self.cl_handle).unwrap();
    //    collider.set_shape(nc::shape::ShapeHandle::new(
    //        nc::shape::Cuboid::new(
    //            na::Vector3::new(0.5, height, 0.5))));
    //}
}
/*
impl Entity for Atlas {
    fn render(&self, _gl: &gl::Gl,
                  _view_matrix: &na::Matrix4<f32>,
                  _proj_matrix: &na::Matrix4<f32>,
                  _camera_pos: &na::Point3<f32>,) {}
    fn update_self_from_physics(&mut self, physics: &Physics) -> Result<(), Error> {
        let rigidbody = physics.get_rigid_body(self.rb_handle)?;
        self.position = *rigidbody.position();
        self.velocity = *rigidbody.velocity();
        Ok(())
    }
    fn update_physics_from_self(&self, physics: &mut Physics) -> Result<(), Error> {
        let rigidbody = physics.get_rigid_body_mut(self.rb_handle)?;
        rigidbody.set_velocity(self.velocity);
        rigidbody.set_position(self.position);
        Ok(())
    }
}
*/