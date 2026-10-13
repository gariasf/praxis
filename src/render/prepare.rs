use std::ops::Range;

use bevy_ecs::prelude::*;

use crate::assets::MeshPool;
use crate::camera::{Camera, projection_matrix};
use crate::components::{MaterialRef, MeshRef, PointLight, Transform};
use crate::render::{CameraUniform, InstanceData, LightUniform};

/// Everything the renderer draws in one frame, owned, so it holds no borrow
/// into the ECS World and can cross to another thread (ADRs 0004, 0016).
#[derive(Default)]
pub struct FrameData {
    pub camera: CameraUniform,
    pub light: LightUniform,
    pub instances: Vec<InstanceData>,
    pub draws: Vec<Draw>,
}

const _: () = {
    const fn assert_send_static<T: Send + 'static>() {}
    assert_send_static::<FrameData>()
};

/// One Primitive drawn for one Instance.
pub struct Draw {
    /// The Primitive's range in the Mesh pool's index buffer.
    pub indices: Range<u32>,
    /// The Primitive's first vertex in the Mesh pool's vertex buffer.
    pub base_vertex: i32,
    /// The Instance's index in `FrameData::instances`.
    pub instance: u32,
}

/// Fills `frame` from the World's camera, lights and Renderables, seen at
/// `aspect_ratio` and placed relative to the camera (ADR 0014), reusing its
/// lists' memory. A Renderable whose Mesh is not in `mesh_pool` is skipped.
pub fn prepare_frame(
    world: &mut World,
    point_light_query: &mut QueryState<(&Transform, &PointLight)>,
    mesh_pool: &MeshPool,
    aspect_ratio: f32,
    frame: &mut FrameData,
) {
    let camera = world.resource::<Camera>();
    let camera_position = camera.position;
    let view_proj = projection_matrix(aspect_ratio) * camera.view_matrix();
    frame.camera = CameraUniform {
        view_proj: view_proj.to_cols_array_2d(),
    };
    frame.light = prepare_lights(world, point_light_query, camera_position);

    frame.instances.clear();
    frame.draws.clear();
    let mut renderable_query = world.query::<(&Transform, &MeshRef, &MaterialRef)>();
    for (transform, mesh_ref, material_ref) in renderable_query.iter(world) {
        let Some(mesh) = mesh_pool.get(mesh_ref.0) else {
            tracing::warn!(
                handle = mesh_ref.0.0,
                "MeshHandle not in pool, skipping entity"
            );
            continue;
        };
        let instance = frame.instances.len() as u32;
        let model_matrix = transform.matrix_relative_to(camera_position);
        frame.instances.push(InstanceData {
            model: model_matrix.to_cols_array_2d(),
            normal_matrix: model_matrix.inverse().transpose().to_cols_array_2d(),
            material_id: material_ref.0.index(),
            _pad: [0; 3],
        });
        frame
            .draws
            .extend(mesh.primitives.iter().map(|primitive| Draw {
                indices: primitive.index_offset..primitive.index_offset + primitive.index_count,
                base_vertex: primitive.vertex_offset as i32,
                instance,
            }));
    }
}

/// The lights, point lights at their position relative to `camera_position`;
/// past the uniform's four, the rest are dropped. The caller keeps
/// `point_light_query` across frames, so preparing allocates nothing.
fn prepare_lights(
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
    let capacity = light_uniform.point_positions.len();
    for (transform, point_light) in point_light_query.iter(world).take(capacity) {
        let position = (transform.translation - camera_position).as_vec3();
        light_uniform.point_positions[count] = position.extend(0.0).to_array();
        light_uniform.point_colors[count] =
            point_light.color.extend(point_light.intensity).to_array();
        count += 1;
    }
    light_uniform.num_point_lights[0] = count as f32;
    light_uniform
}
