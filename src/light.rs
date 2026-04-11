#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub direction: [f32; 4], // xyz = direction, w = unused
    pub color: [f32; 4],     // rgb = color, a = intensity
    pub ambient: [f32; 4],   // rgb = ambient color, a = strengh
}
