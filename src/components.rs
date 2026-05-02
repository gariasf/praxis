use bevy_ecs::prelude::*;

use crate::assets::MeshHandle;

#[derive(Component)]
pub struct Transform(pub glam::Affine3A);

#[derive(Component)]
pub struct MeshRef(pub MeshHandle);
