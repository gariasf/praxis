use bevy_ecs::prelude::*;
use winit::keyboard::KeyCode;

use crate::input::Input;
use crate::time::Time;

#[derive(Resource)]
pub struct Camera {
    /// World position in f64 metres (ADR 0014).
    pub position: glam::DVec3,
    pub yaw: f32,   // radians, left-right
    pub pitch: f32, // radians, up-down
    pub speed: f32,
    pub sensitivity: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: glam::dvec3(0.0, 0.0, 2.0),
            yaw: -std::f32::consts::FRAC_PI_2, // look towards -Z
            pitch: 0.0,
            speed: 2.0,
            sensitivity: 0.003,
        }
    }

    /// A camera at `position` facing `target`, which must not be straight
    /// above or below it.
    pub fn looking_at(position: glam::DVec3, target: glam::DVec3) -> Self {
        let direction = (target - position).as_vec3().normalize();
        Self {
            position,
            yaw: direction.z.atan2(direction.x),
            pitch: direction.y.asin(),
            ..Self::new()
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

    /// The view rotation alone: the GPU works relative to the camera
    /// (ADR 0014), so the camera sits at the origin of view space and of
    /// every position the shaders see.
    pub fn view_matrix(&self) -> glam::Mat4 {
        glam::Mat4::look_at_rh(glam::Vec3::ZERO, self.forward(), glam::Vec3::Y)
    }
}

/// Vertical field of view, in radians.
const FIELD_OF_VIEW: f32 = std::f32::consts::FRAC_PI_4;
/// Distance to the near plane in metres. There is no far plane (ADR 0015).
const NEAR_PLANE: f32 = 0.1;

/// Infinite reversed perspective (ADR 0015): a point `distance` metres in
/// front of the camera, measured along the view axis, lands at depth `NEAR_PLANE / distance`, so depth is 1
/// at the near plane and falls towards 0 at infinity. Pair it with a depth
/// buffer cleared to 0 and compared with `Greater`.
pub fn projection_matrix(aspect_ratio: f32) -> glam::Mat4 {
    glam::Mat4::perspective_infinite_reverse_rh(FIELD_OF_VIEW, aspect_ratio, NEAR_PLANE)
}

pub fn fly_camera(input: Res<Input>, time: Res<Time>, mut camera: ResMut<Camera>) {
    let forward_dir = camera.forward().as_dvec3();
    let right_dir = camera.right().as_dvec3();
    let speed = f64::from(camera.speed * time.delta_time);

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

#[cfg(test)]
mod tests {
    use super::*;

    /// Depth matches the analytic `near / distance`, distance along the view
    /// axis, within f32 rounding from the near plane to 10,000 km, and stays
    /// inside (0, 1], so no far plane clips anything in front of the camera.
    #[test]
    fn reversed_depth_is_near_over_distance_with_no_far_plane() {
        let projection = projection_matrix(16.0 / 9.0);
        let mut distance = f64::from(NEAR_PLANE);
        while distance <= 1e7 {
            let view_position = glam::vec3(0.0, 0.0, -distance as f32);
            let clip = projection * view_position.extend(1.0);
            let depth = f64::from(clip.z / clip.w);
            let expected = f64::from(NEAR_PLANE) / -f64::from(view_position.z);
            assert!(
                (depth - expected).abs() <= 4.0 * f64::from(f32::EPSILON) * expected,
                "distance {distance}: depth {depth}, expected {expected}"
            );
            assert!(
                0.0 <= clip.z && clip.z <= clip.w,
                "distance {distance}: clipped, z {} w {}",
                clip.z,
                clip.w
            );
            distance *= 1.5;
        }
    }
}
