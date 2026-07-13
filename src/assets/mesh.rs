pub struct Primitive {
    pub vertex_offset: u32,
    pub index_offset: u32,
    pub index_count: u32,
}

pub struct Mesh {
    pub primitives: Vec<Primitive>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MeshHandle(pub u32);
