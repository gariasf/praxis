use bevy_ecs::prelude::*;

use crate::assets::{MaterialHandle, MeshHandle};

/// Where an entity sits in the World (ADR 0014): translation in f64 metres,
/// rotation and scale in f32, 64 bytes like the `Affine3A` it replaced.
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct Transform {
    pub translation: glam::DVec3,
    pub rotation: glam::Quat,
    pub scale: glam::Vec3,
}

const _: () = assert!(std::mem::size_of::<Transform>() == 64);
const _: () = assert!(std::mem::align_of::<Transform>() == 16);

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: glam::DVec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        scale: glam::Vec3::ONE,
    };

    pub const fn from_translation(translation: glam::DVec3) -> Self {
        Self {
            translation,
            ..Self::IDENTITY
        }
    }

    /// The model matrix relative to `origin`: the translation difference is
    /// taken in f64, then cast, so the matrix holds no World position in f32.
    pub fn matrix_relative_to(&self, origin: glam::DVec3) -> glam::Mat4 {
        glam::Mat4::from_scale_rotation_translation(
            self.scale,
            self.rotation,
            (self.translation - origin).as_vec3(),
        )
    }
}

#[derive(Component)]
pub struct MeshRef(pub MeshHandle);

#[derive(Component)]
pub struct MaterialRef(pub MaterialHandle);

/// A point light at its entity's `Transform`.
#[derive(Component)]
pub struct PointLight {
    pub color: glam::Vec3,
    pub intensity: f32,
}
