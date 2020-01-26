use nalgebra as na;

pub struct ServerPacket {
    reliable: Vec<ReliableMessage>,
    unreliable: GameState,
}

pub struct ReliableMessage {
    pub id: u32,
    pub type_: u32,
    pub data: Vec<u8>,
}

pub struct GameState {
    players: Vec<Player>,
}

pub struct Player {
    pos: na::Vector3<f32>,
    rot: na::Vector3<f32>,
    hp: u16,
}
