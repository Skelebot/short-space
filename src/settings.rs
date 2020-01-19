
pub struct GameSettings {
    pub debug: bool,
    pub movement_speed: f32,
    pub mouse_sensitivity: f32,
    pub vsync: bool,
    pub window_width: i32,
    pub window_height: i32,
}

impl Default for GameSettings {
    fn default() -> Self {
        GameSettings {
            debug: false,
            movement_speed: 8.0,
            mouse_sensitivity: 8.0,
            vsync: false,
            window_width: 800,
            window_height: 600,
        }
    }
}
