//movement parameters
//pub const PM_STOPSPEED: f32 = 1.0;
//pub const PM_DUCKSCALE: f32 = 0.25;
//pub const PM_ACCELERATE: f32 = 0.15;
//pub const PM_AIRACCELERATE: f32 = 0.05;
//pub const PM_FRICTION: f32 = 1.0;
//pub const JUMP_VELOCITY: f32 = 5.0;
////const PM_FLYFRICTION: f32 = 3.0;
//
//pub const VIEW_HEIGHT: f32 = 0.8;
//pub const DUCK_HEIGHT: f32 = 0.4;
//pub const MAX_ACCEL: f32 = 32.0;
//
//pub const MIN_WALK_NORMAL: f32 = 0.4;

#[derive(Clone)]
pub struct GameSettings {
    pub noclip_speed: f32,
    pub mouse_sensitivity: f32,
    pub vsync: bool,
    pub window_width: i32,
    pub window_height: i32,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            noclip_speed: 10.0,
            mouse_sensitivity: 0.1,
            vsync: true,
            window_width: 800,
            window_height: 600,
        }
    }
}
