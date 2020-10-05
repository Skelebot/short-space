
use std::time::{Duration, Instant};

pub struct Time {
    last_instant: Instant,
    pub delta: f32,
    pub elapsed: Duration,
}

impl Time {
    pub fn new() -> Time {
        Time {
            last_instant: Instant::now(),
            delta: 0.0,
            elapsed: Duration::new(0, 0),
        }
    }
}

use legion::system;
#[system]
pub fn update_time(
    #[resource] time: &mut Time,
) {

    let dur = time.last_instant.elapsed();
    let delta = (dur.as_secs() as f64 + dur.subsec_nanos() as f64 / 1_000_000_000.0) as f32;

    time.elapsed = dur;
    time.delta = delta;
    time.last_instant = Instant::now();
}