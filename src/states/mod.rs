use std::any::TypeId;

pub mod game;
pub mod loading;
pub mod main;

// Marker component for entities which get removed when on_stop() of their containing state gets removed
#[derive(Clone, Copy)]
pub struct Scoped {
    id: TypeId,
}
