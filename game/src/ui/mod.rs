use legion::{Resources, World};

pub mod console;
pub mod debug;

pub trait StatusWindow {
    fn open(&self) -> bool;
    fn update(&mut self, world: &mut World, resources: &mut Resources) -> eyre::Result<()>;
}
