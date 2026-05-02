use bevy_ecs::resource::Resource;
use std::collections::HashSet;
use winit::keyboard::KeyCode;

#[derive(Resource, Default)]
pub struct Input {
    pub pressed: HashSet<KeyCode>,
    pub just_pressed: HashSet<KeyCode>,
}
