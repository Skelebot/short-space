use camera::Camera;

pub struct GameState<'a> {
    pub in_menu: bool,
    pub active_camera: &'a mut Camera,
}

impl GameState<'_>{
    pub fn new<'a>(active_camera: &'a mut Camera) -> GameState {
        GameState {
            in_menu: false,
            active_camera: active_camera,
        }
    }
}
