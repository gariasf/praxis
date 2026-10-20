/// Instances the Instance buffer holds, 64 MiB of them: one fixed capacity
/// for the app, tests, bench and CI until M4's split of the 8 GB Budget
/// sizes it.
pub const INSTANCE_CAPACITY: u32 = 1 << 19;

/// Edge of the grid World positions are anchored to on the GPU, in metres
/// (ADR 0014); every shader module declares it from this one.
pub const ANCHOR_CELL_SIZE: f64 = 64.0;

/// Splits a World position into the integer anchor of its grid cell and its
/// f32 offset from that anchor's corner, so no World position reaches the GPU
/// in f32 (ADR 0014).
pub fn anchor_and_offset(position: glam::DVec3) -> ([i32; 3], [f32; 3]) {
    let anchor = (position / ANCHOR_CELL_SIZE).floor();
    let offset = position - anchor * ANCHOR_CELL_SIZE;
    (anchor.as_ivec3().to_array(), offset.as_vec3().to_array())
}

/// The columns of `matrix`, each padded to the 16 bytes WGSL gives a `vec3`
/// column of a `mat3x3`.
pub fn padded_columns(matrix: glam::Mat3) -> [[f32; 4]; 3] {
    [matrix.x_axis, matrix.y_axis, matrix.z_axis].map(|column| column.extend(0.0).to_array())
}

/// The Mesh reference of a freed slot's Instance, which draws nothing.
pub const NO_MESH: u32 = u32::MAX;

/// The Material ID of a Primitive that names no Material.
pub const NO_MATERIAL: u32 = u32::MAX;

/// One Instance where its Transform puts it: the anchor and offset of its
/// pivot, which do not change as the camera moves, and its rotation-scale;
/// and its Mesh's Handle, or `NO_MESH`.
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub rotation_scale: [[f32; 4]; 3],
    pub normal_matrix: [[f32; 4]; 3],
    pub anchor: [i32; 3],
    pub material_id: u32,
    pub offset: [f32; 3],
    pub mesh: u32,
}
