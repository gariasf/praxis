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

// Layout guard: WGSL rounds the `array<InstanceData>` stride to 144 bytes; the
// Rust struct (incl. `_pad`) must equal it or per-instance reads misalign.
const _: () = assert!(
    std::mem::size_of::<InstanceData>() == 144,
    "InstanceData must stay 144 bytes to match the WGSL storage stride"
);
