use std::time::{Duration, Instant};

/// An accumulator that counts up with every frame. When it exceeds PhysicsSettings.step_time,
/// the next physics simulation occurs. Until then, it is effectively a measure of how much
/// more time is required before another whole physics step can be taken - we can use this remainder
/// to get a blending factor between the previous and current physics state by dividing by step_time.
/// This gives a value in range [0,1] which should be used to perform a linear interpolation between
/// the two physics states to get the current state to render.
/// The second number is used to inform other systems whether they can do any physics calculations,
/// and how many physics steps can be executed this frame.
pub struct PhysicsTimer {
    timer: f64,
    steps_due: u8,
    step_time: f64,
}
impl PhysicsTimer {
    pub fn new(step_time: f64) -> Self {
        PhysicsTimer {
            timer: 0.0,
            steps_due: 0,
            step_time,
        }
    }
    pub fn update(&mut self, delta: f64) {
        self.timer += delta;
        self.steps_due = 0;

        let steps = self.timer / self.step_time;
        self.steps_due = steps.floor() as u8;
        self.timer -= steps.floor() * self.step_time;
    }
    pub fn steps_due(&self) -> u8 {
        self.steps_due
    }
    pub fn lerp(&self) -> f64 {
        self.timer / self.step_time
    }
}

pub struct Time {
    pub current: Instant,
    pub delta: Duration,
}

impl Time {
    pub fn update(&mut self) {
        self.delta = self.current.elapsed();
        self.current = Instant::now();
    }
}

impl Default for Time {
    fn default() -> Self {
        Time {
            current: Instant::now(),
            delta: Duration::default(),
        }
    }
}
