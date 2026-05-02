use bevy_ecs::system::ResMut;

use crate::resources::Input;

pub fn clear_just_pressed(mut input: ResMut<Input>) {
    input.just_pressed.clear();
}
