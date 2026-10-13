use bevy_ecs::prelude::*;

use crate::components::{MaterialRef, MeshRef, PointLight, Transform};
use crate::render::{InstanceData, LightUniform};

/// Each Renderable's Instance, its model matrix relative to `camera_position`
/// (ADR 0014).
pub fn prepare_renderables(world: &mut World, camera_position: glam::DVec3) -> Vec<InstanceData> {
    let mut renderable_query = world.query::<(&Transform, &MeshRef, &MaterialRef)>();
    renderable_query
        .iter(world)
        .map(|(transform, _mesh_ref, material_ref)| {
            let model_matrix = transform.matrix_relative_to(camera_position);
            InstanceData {
                model: model_matrix.to_cols_array_2d(),
                normal_matrix: model_matrix.inverse().transpose().to_cols_array_2d(),
                material_id: material_ref.0.index(),
                _pad: [0; 3],
            }
        })
        .collect()
}

/// The lights, point lights at their position relative to `camera_position`;
/// past the uniform's four, the rest are dropped. The caller keeps
/// `point_light_query` across frames, so preparing allocates nothing.
pub fn prepare_lights(
    world: &World,
    point_light_query: &mut QueryState<(&Transform, &PointLight)>,
    camera_position: glam::DVec3,
) -> LightUniform {
    let mut light_uniform = LightUniform {
        direction: [0.5, -1.0, -0.3, 0.0],
        color: [1.0, 1.0, 1.0, 1.0],
        ambient: [1.0, 1.0, 1.0, 0.1],
        point_positions: [[0.0; 4]; 4],
        point_colors: [[0.0; 4]; 4],
        num_point_lights: [0.0; 4],
    };
    let mut count = 0;
    for (transform, point_light) in point_light_query.iter(world).take(4) {
        let position = (transform.translation - camera_position).as_vec3();
        light_uniform.point_positions[count] = position.extend(0.0).to_array();
        light_uniform.point_colors[count] =
            point_light.color.extend(point_light.intensity).to_array();
        count += 1;
    }
    light_uniform.num_point_lights[0] = count as f32;
    light_uniform
}
