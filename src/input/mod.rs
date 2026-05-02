use bevy_ecs::prelude::*;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Resource, Default)]
pub struct Input {
    pub pressed: HashSet<KeyCode>,
    pub just_pressed: HashSet<KeyCode>,
}

pub fn clear_just_pressed(mut input: ResMut<Input>) {
    input.just_pressed.clear();
}
