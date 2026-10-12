use bevy_ecs::prelude::*;

#[derive(Resource, Default)]
pub struct Time {
    // Seconds since the previous frame, zero on the first. Injected through
    // `State::update`: the window measures it, a capture fixes it.
    pub delta_time: f32,
}
