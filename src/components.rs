use bevy_ecs::prelude::*;

use crate::assets::{MaterialHandle, MeshHandle};

#[derive(Component)]
pub struct Transform(pub glam::Affine3A);

#[derive(Component)]
pub struct MeshRef(pub MeshHandle);

#[derive(Component)]
pub struct MaterialRef(pub MaterialHandle);
