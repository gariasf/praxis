use bevy_ecs::prelude::*;

use crate::components::{MaterialRef, MeshRef, Transform};
use crate::render::InstanceData;

pub fn prepare_renderables(world: &mut World) -> Vec<InstanceData> {
    let mut renderable_query = world.query::<(&Transform, &MeshRef, &MaterialRef)>();
    renderable_query
        .iter(world)
        .map(|(transform, _mesh_ref, material_ref)| {
            let model_matrix = glam::Mat4::from(transform.0);
            InstanceData {
                model: model_matrix.to_cols_array_2d(),
                normal_matrix: model_matrix.inverse().transpose().to_cols_array_2d(),
                material_id: material_ref.0.index(),
                _pad: [0; 3],
            }
        })
        .collect()
}
