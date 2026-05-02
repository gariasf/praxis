use bevy_ecs::system::{Res, ResMut};
use winit::keyboard::KeyCode;

use crate::resources::{Camera, Input, Time};

pub fn fly_camera(input: Res<Input>, time: Res<Time>, mut camera: ResMut<Camera>) {
    let forward_dir = camera.forward();
    let right_dir = camera.right();
    let speed = camera.speed * time.delta_time;

    if input.pressed.contains(&KeyCode::KeyW) {
        camera.position += forward_dir * speed;
    }
    if input.pressed.contains(&KeyCode::KeyS) {
        camera.position -= forward_dir * speed;
    }
    if input.pressed.contains(&KeyCode::KeyD) {
        camera.position += right_dir * speed;
    }
    if input.pressed.contains(&KeyCode::KeyA) {
        camera.position -= right_dir * speed;
    }
    if input.pressed.contains(&KeyCode::Space) {
        camera.position.y += speed;
    }
    if input.pressed.contains(&KeyCode::ShiftLeft) {
        camera.position.y -= speed;
    }
}
