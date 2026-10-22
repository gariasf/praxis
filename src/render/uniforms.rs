#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub direction: [f32; 4],          // xyz = direction, w = unused
    pub color: [f32; 4],              // rgb = color, a = intensity
    pub ambient_irradiance: [f32; 4], // rgb = the scene's ambient irradiance, w = unused

    pub point_positions: [[f32; 4]; 4], // xyz = position relative to the camera, w = range
    pub point_colors: [[f32; 4]; 4],    // rgb = color, a = intensity
    pub point_light_count: u32,
    pub directional_light_count: u32, // 0 or 1: the uniform holds one directional light
    pub _pad: [u32; 2],
}

#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    /// Projection times the view rotation: positions reach it relative to
    /// the camera (ADR 0014).
    pub view_proj: [[f32; 4]; 4],
    /// The camera's position as an anchor plus offset (`anchor_and_offset`),
    /// from which the shader composes Instances relative to the camera.
    pub anchor: [i32; 3],
    pub _pad_anchor: u32,
    pub offset: [f32; 3],
    pub _pad_offset: u32,
}
