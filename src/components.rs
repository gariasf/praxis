use bevy_ecs::prelude::*;

use crate::assets::{MaterialHandle, MeshHandle};
use crate::transform::WorldTransform;

/// Where an entity sits in the Game world (ADR 0014): translation in f64 metres,
/// rotation and scale in f32, 64 bytes like the `Affine3A` it replaced.
/// A child's is relative to its parent (`ChildOf`).
#[derive(Component, Copy, Clone, Debug, PartialEq)]
#[require(WorldTransform)]
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

    pub(crate) fn to_affine(self) -> glam::DAffine3 {
        glam::DAffine3::from_scale_rotation_translation(
            self.scale.as_dvec3(),
            self.rotation.as_dquat(),
            self.translation,
        )
    }
}

#[derive(Component)]
pub struct MeshRef(pub MeshHandle);

#[derive(Component)]
pub struct MaterialRef(pub MaterialHandle);

/// A point light at its entity's `Transform`.
#[derive(Component, Clone, Copy)]
pub struct PointLight {
    pub color: glam::Vec3,
    pub intensity: f32,
    /// Metres from the light past which it adds nothing (KHR_lights_punctual's
    /// `range`, which must be positive and here finite).
    pub range: f32,
}

/// A directional light shining along its entity's forward axis: -Z turned by
/// its `Transform`, the way a camera looks.
#[derive(Component, Clone, Copy)]
pub struct DirectionalLight {
    pub color: glam::Vec3,
    pub intensity: f32,
}

/// The light a Scene's surroundings give, constant until ADR 0020's
/// irradiance probes (M5) and M6's atmosphere replace it: the irradiance
/// every surface receives from all around, and the radiance of the
/// background, seen wherever nothing is drawn. The default is dark.
#[derive(Resource, Clone, Copy, Default)]
pub struct Environment {
    pub ambient_irradiance: glam::Vec3,
    pub background_radiance: glam::Vec3,
}
