//use world::Scene;

pub struct GameState {
    pub should_exit: bool,
    pub paused: bool,
}

impl GameState{
    pub fn new() -> GameState {
        GameState {
            should_exit: false,
            paused: false,
        }
    }
}
