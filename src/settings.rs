use nalgebra as na;

pub struct GameSettings {
    pub debug: bool,
    pub movement_sensitivity: f32,
    pub mouse_sensitivity: f32,
    pub vsync: bool,
    pub window_width: i32,
    pub window_height: i32,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            debug: false,
            movement_sensitivity: 0.0005,
            mouse_sensitivity: 0.0005,
            vsync: false,
            window_width: 800,
            window_height: 600,
        }
    }
}
