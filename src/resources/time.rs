use bevy_ecs::prelude::Resource;

#[derive(Resource)]
pub struct Time {
    // Delta Time in seconds
    pub delta_time: f32,
    last_frame: std::time::Instant,
}

impl Time {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            last_frame: std::time::Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        self.delta_time = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
    }
}
