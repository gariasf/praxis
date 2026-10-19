use crate::assets::MaterialHandle;

pub struct Primitive {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
    /// What a Renderable without a Material Handle draws it with: the
    /// Material its Primitives name, if any.
    pub material: Option<MaterialHandle>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MeshHandle(pub u32);
