use crate::assets::MaterialHandle;

pub struct Primitive {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    /// The Material its first Primitive names, if any, which a Renderable
    /// without a Material Handle draws every Primitive with until M3's draw
    /// path draws each with its own.
    pub material: Option<MaterialHandle>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MeshHandle(pub u32);
