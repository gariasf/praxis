pub const INSTANCE_BUFFER_INITIAL_CAPACITY: u64 = 16;

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceData {
    pub model: [[f32; 4]; 4],
    pub normal_matrix: [[f32; 4]; 4],
    pub material_id: u32,
    // WGSL rounds the storage stride to 16; pad so Rust matches (128 + 16 = 144).
    pub _pad: [u32; 3],
}
