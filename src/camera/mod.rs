use bevy_ecs::prelude::*;
use winit::keyboard::KeyCode;

use crate::input::Input;
use crate::time::Time;

#[derive(Resource)]
pub struct Camera {
    pub position: glam::Vec3,
    pub yaw: f32,   // radians, left-right
    pub pitch: f32, // radians, up-down
    pub speed: f32,
    pub sensitivity: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: glam::vec3(0.0, 0.0, 2.0),
            yaw: -std::f32::consts::FRAC_PI_2, // look towards -Z
            pitch: 0.0,
            speed: 2.0,
            sensitivity: 0.003,
        }
    }

    pub fn forward(&self) -> glam::Vec3 {
        glam::vec3(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        )
        .normalize()
    }

    pub fn right(&self) -> glam::Vec3 {
        self.forward().cross(glam::Vec3::Y).normalize()
    }

    pub fn view_matrix(&self) -> glam::Mat4 {
        let target = self.position + self.forward();
        glam::Mat4::look_at_rh(self.position, target, glam::Vec3::Y)
    }
}

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
