use crate::spacetime::{Child, PhysicsTimer, Position, Time};
use legion::{system, world::SubWorld};
use legion::{Entity, IntoQuery};

// Shamelessly stolen from nphysics (https://www.nphysics.org/rustdoc/nphysics3d/algebra/struct.Velocity3.html)
mod velocity;
pub use velocity::Velocity;

pub struct Collider {
    pub handle: nc::shape::ShapeHandle<f32>,
}

impl From<nc::shape::ShapeHandle<f32>> for Collider {
    fn from(shape: nc::shape::ShapeHandle<f32>) -> Self {
        Self { handle: shape }
    }
}

#[system]
#[read_component(Collider)]
#[write_component(Position)]
#[write_component(Velocity)]
pub fn step(
    #[resource] p_timer: &mut PhysicsTimer,
    #[resource] time: &Time,
    //#[resource] physics_settings: &PhysicsSettings,
    world: &mut SubWorld,
) {
    p_timer.update(time.delta);
    for _ in 0..p_timer.steps_due() {
        // Update Positions
        <&mut Position>::query().for_each_mut(world, |p| {
            let future = p.future();
            *p.past_mut() = *future;
        });
    }
}

#[system]
#[write_component(Position)]
#[read_component(Child)]
#[read_component(Entity)]
pub fn children_update(world: &mut SubWorld) {
    let child_query = <(Entity, &Child)>::query();
    let (w_a, mut w_b) = world.split_for_query(&child_query);

    <(Entity, &Child)>::query().for_each(&w_a, |(e, c)| {
        let parent_pos = *<&Position>::query()
            .get(&w_b, c.parent)
            .expect("Parent entity doesn't exist or doesn't have a Position");
        let position = <&mut Position>::query().get_mut(&mut w_b, *e).unwrap();
        *position = parent_pos;
    });
}
