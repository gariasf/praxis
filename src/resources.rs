use bevy_ecs::resource::Resource;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MeshHandle(pub u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MaterialHandle(pub u32);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct TextureHandle(pub u32);

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

pub struct Texture {
    pub view: wgpu::TextureView,
    pub texture: wgpu::Texture,
}

pub struct Material {
    pub bind_group: wgpu::BindGroup,
}

#[derive(Resource, Default)]
pub struct MeshPool(pub Vec<Mesh>);

#[derive(Resource, Default)]
pub struct MaterialPool(pub Vec<Material>);

#[derive(Resource, Default)]
pub struct TexturePool(pub Vec<Texture>);

impl MeshPool {
    pub fn insert(&mut self, mesh: Mesh) -> MeshHandle {
        let handle = MeshHandle(self.0.len() as u32);
        self.0.push(mesh);
        handle
    }
    pub fn get(&self, handle: MeshHandle) -> &Mesh {
        &self.0[handle.0 as usize]
    }
}
