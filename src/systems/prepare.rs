use crate::{
    components::{MeshRef, Transform},
    render::instance::InstanceData,
};
use bevy_ecs::world::World;

pub fn prepare_renderables(world: &mut World, queue: &wgpu::Queue, instance_buffer: &wgpu::Buffer) {
    let mut renderable_query = world.query::<(&Transform, &MeshRef)>();
    let instance_data: Vec<InstanceData> = renderable_query
        .iter(world)
        .map(|(transform, _mesh_ref)| {
            let model_matrix = glam::Mat4::from(transform.0);
            InstanceData {
                model: model_matrix.to_cols_array_2d(),
                normal_matrix: model_matrix.inverse().transpose().to_cols_array_2d(),
            }
        })
        .collect();

    queue.write_buffer(instance_buffer, 0, bytemuck::cast_slice(&instance_data));
}
