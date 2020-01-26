use world::Scene;

pub struct GameState<'a> {
    pub in_menu: bool,
    pub active_scene: &'a mut Scene,
}

impl GameState<'_>{
    pub fn new<'a>(active_scene: &'a mut Scene) -> GameState<'a> {
        GameState {
            in_menu: false,
            active_scene: active_scene,
        }
    }
}
