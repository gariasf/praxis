use crate::assets::{Material, Mesh, MeshHandle, Texture};
use bevy_ecs::prelude::Resource;

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
