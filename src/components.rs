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

/// Bends its Renderable's Mesh on the GPU each frame from `rest`, a Mesh of
/// the same vertices whose +Y axis runs up its middle from y = 0: a stand-in
/// for skinning (#208). The middle bends into a circular arc towards the
/// azimuth, and every vertex turns with the middle at its height. The GPU
/// writes the bent Mesh's vertices, so it is the Renderable's own: two
/// Renderables bending one Mesh would race. Game state, set each Tick in the
/// fixed schedule; frames draw the curvature blended between the two newest
/// Ticks' (ADR 0022).
#[derive(Component, Clone, Copy)]
pub struct Bend {
    pub rest: MeshHandle,
    /// The direction the middle bends towards, about +Y from +X towards +Z,
    /// in radians.
    pub azimuth: f32,
    /// Radians the middle turns through per metre of its height, after the
    /// newest Tick and after the one before.
    pub curvature: f32,
    pub previous_curvature: f32,
}

/// A point light at its entity's `Transform`.
#[derive(Component, Clone, Copy)]
pub struct PointLight {
    pub color: glam::Vec3,
    /// Luminous intensity, in candela (ADR 0023).
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
    /// Illuminance on a surface facing it, in lux (ADR 0023).
    pub intensity: f32,
}
