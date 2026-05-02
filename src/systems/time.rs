use bevy_ecs::system::ResMut;

use crate::resources::Time;

pub fn tick_time(mut time: ResMut<Time>) {
    time.tick();
}
