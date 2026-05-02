use bevy_ecs::prelude::*;

#[derive(Resource)]
pub struct Time {
    // Delta time in seconds. Zero on the first tick (no prior frame).
    pub delta_time: f32,
    last_frame: Option<std::time::Instant>,
}

impl Time {
    pub fn new() -> Self {
        Self {
            delta_time: 0.0,
            last_frame: None,
        }
    }

    pub fn tick(&mut self) {
        let now = std::time::Instant::now();
        if let Some(last) = self.last_frame {
            self.delta_time = (now - last).as_secs_f32();
        }
        self.last_frame = Some(now);
    }
}

pub fn tick_time(mut time: ResMut<Time>) {
    time.tick();
}
