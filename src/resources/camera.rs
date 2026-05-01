use bevy_ecs::resource::Resource;

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
