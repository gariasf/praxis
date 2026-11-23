use crate::assets::MaterialHandle;
use crate::cooked::MESH_MAX_LODS;

/// A Mesh in the Mesh pool, as prepare needs it; the pool's GPU tables hold
/// what the GPU draws it from.
pub struct Mesh {
    /// Whether a Primitive with triangles names no Material, so that only a
    /// Renderable with a Material Handle draws it.
    pub needs_material_handle: bool,
    /// Its range of the vertex Arena, which a bend writes.
    pub vertices: std::ops::Range<u32>,
    /// The box that holds its vertices, and every shape a bend gives them,
    /// its lowest corner then its highest, as the Mesh table holds it.
    pub bounds: [[f32; 3]; 2],
    /// Each Primitive's `CookedPrimitive::texel_density`, in Primitive order.
    pub texel_densities: Vec<f32>,
    /// Each Primitive's Material, in Primitive order; `None` for one that
    /// names none or is not drawn.
    pub materials: Vec<Option<MaterialHandle>>,
}

/// A Mesh's index in the Mesh table, which the GPU sees, and the generation
/// of that index it was loaded in, which no later Mesh there shares.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct MeshHandle {
    pub index: u32,
    pub generation: u32,
}

/// A Mesh as the Mesh table holds it, at its Handle's index: each LOD of its
/// chain at the LOD's index, LOD 0 first, as its Meshlets, whose triangles
/// follow one another in the Meshlet triangle Arena, its Thin Primitives'
/// last, with its LOD error; and the box that holds its vertices.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshData {
    /// The LODs it is drawn from: none for a Mesh no pass draws.
    pub lod_count: u32,
    pub first_meshlets: [u32; MESH_MAX_LODS],
    pub meshlet_counts: [u32; MESH_MAX_LODS],
    pub first_triangles: [u32; MESH_MAX_LODS],
    pub triangle_counts: [u32; MESH_MAX_LODS],
    /// How many of each LOD's triangles, its last, are its Thin Primitives'.
    pub thin_triangle_counts: [u32; MESH_MAX_LODS],
    pub lod_errors_metres: [f32; MESH_MAX_LODS],
    /// The box's lowest corner, then its highest, in mesh space.
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    /// The index Arena's entries from `first_index` count from `first_vertex`.
    pub first_vertex: u32,
    pub first_index: u32,
}

/// A Meshlet as the Meshlet Arena holds it: its vertices in the Meshlet
/// vertex Arena, each an index into the vertex Arena; its triangles in the
/// Meshlet triangle Arena, each its three corners as indices into those
/// vertices, a byte each, lowest byte first; its Primitive's Material ID,
/// or `NO_MATERIAL`; and its Thin Primitive's radius, 0 for any other's.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshletData {
    pub first_vertex: u32,
    pub first_triangle: u32,
    pub triangle_count: u32,
    pub material_id: u32,
    pub thin_radius_metres: f32,
}
