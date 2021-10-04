use legion::{Entity, Resources};

mod time;
pub use time::*;

mod position;
pub use position::*;

pub type Scale = na::Vector3<f32>;

/// A component that makes an entity copy the Position of another entity with an offset
pub struct Child {
    pub parent: Entity,
    pub offset: Position,
}

pub fn prepare(resources: &mut Resources) {
    let mut time = resources.get_mut::<Time>().unwrap();
    time.update();
}
