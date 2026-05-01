use crate::assets::MaterialHandle;

pub struct Primitive {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub material: MaterialHandle,
    pub texture_bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MeshHandle(pub u32);
