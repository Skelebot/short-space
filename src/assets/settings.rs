use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameSettings {
    pub noclip_speed: f32,
    pub mouse_sensitivity: f32,
    pub sprint_multiplier: f32,
    pub vsync: bool,
    pub window_width: i32,
    pub window_height: i32,

    pub player_height: f32,
    pub player_radius: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PhysicsSettings {
    pub gravity: f32,
    pub air_friction: f32,
    pub step_time: f64,
}
