/// A Mesh in the Mesh pool, as prepare needs it; the pool's GPU tables hold
/// what the GPU draws it from.
pub struct Mesh {
    /// Whether a Primitive names no Material, so that only a Renderable with
    /// a Material Handle draws it.
    pub needs_material_handle: bool,
    /// Its range of the vertex Arena, which a bend writes (#208).
    #[cfg_attr(not(test), allow(dead_code))]
    pub vertices: std::ops::Range<u32>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct MeshHandle(pub u32);

/// A Mesh as the Mesh table holds it, at its Handle's index: its Meshlets,
/// whose triangles follow one another in the Meshlet triangle Arena, and the
/// box that holds its vertices.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshData {
    pub first_meshlet: u32,
    pub meshlet_count: u32,
    pub first_triangle: u32,
    pub triangle_count: u32,
    /// The box's lowest corner, then its highest, in mesh space.
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
}

/// A Meshlet as the Meshlet Arena holds it: its vertices in the Meshlet
/// vertex Arena, each an index into the vertex Arena; its triangles in the
/// Meshlet triangle Arena, each its three corners as indices into those
/// vertices, a byte each, lowest byte first; and its Primitive's Material ID,
/// or `NO_MATERIAL`.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshletData {
    pub first_vertex: u32,
    pub first_triangle: u32,
    pub triangle_count: u32,
    pub material_id: u32,
}
