use camera::Camera;
use scene::Scene;

pub struct GameState<'a> {
    pub in_menu: bool,
    pub active_camera: &'a mut Camera,
    pub active_scene: Option<&'a Scene>,
}

impl GameState<'_>{
    pub fn new<'a>(active_camera: &'a mut Camera) -> GameState {
        GameState {
            in_menu: false,
            active_camera: active_camera,
            active_scene: None
        }
    }
}
