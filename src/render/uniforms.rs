/// Lights the light buffer holds, directional and point lights together: a
/// fixed capacity, provisional until M4's split of the 8 GB Budget sizes it.
pub const LIGHT_CAPACITY: u32 = 4096;

/// The light buffer's counts, its directional lights first, and the scene's
/// ambient illuminance.
#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightUniform {
    pub ambient_illuminance: [f32; 4], // rgb = the scene's ambient illuminance, w = unused
    pub directional_light_count: u32,
    pub point_light_count: u32,
    pub _pad: [u32; 2],
}

/// A light as the light buffer holds it, directional lights first, then point
/// lights. A point light's position reaches the GPU as an Anchor plus an f32
/// offset, which shaders compose relative to the camera (ADR 0014); a
/// directional light has no position, and its offset holds the direction it
/// shines along.
#[repr(C)]
#[derive(Copy, Clone, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightData {
    pub anchor: [i32; 3],
    /// A point light's range, in metres; 0 for a directional light.
    pub range: f32,
    pub offset: [f32; 3],
    pub _pad_offset: u32,
    /// Its colour times its intensity: candela for a point light, lux for a
    /// directional light (ADR 0023).
    pub intensity: [f32; 3],
    pub _pad_intensity: u32,
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
    /// What the lit pass scales luminance by (`Camera::exposure`).
    pub exposure: f32,
    pub offset: [f32; 3],
    pub _pad_offset: u32,
}
