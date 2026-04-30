use crate::resources::MeshHandle;
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Transform(pub glam::Affine3A);

#[derive(Component)]
pub struct MeshRef(pub MeshHandle);
