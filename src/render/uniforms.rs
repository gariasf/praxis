#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub direction: [f32; 4], // xyz = direction, w = unused
    pub color: [f32; 4],     // rgb = color, a = intensity
    pub ambient: [f32; 4],   // rgb = ambient color, a = strength

    pub point_positions: [[f32; 4]; 4], // xyz = position relative to the camera, w = unused
    pub point_colors: [[f32; 4]; 4],    // rgb = color, a = intensity
    pub num_point_lights: [f32; 4],     // x = count, yzw = padding
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// Projection times the view rotation: positions reach it relative to
    /// the camera (ADR 0014).
    pub view_proj: [[f32; 4]; 4],
}
