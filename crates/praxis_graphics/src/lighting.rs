//! Lighting system for the graphics engine.
//!
//! This module provides data structures and uniform buffer management for lighting in shaders.
//! It supports directional lights (sun-like, infinite distance) and point lights (omnidirectional
//! with distance-based attenuation).
//!
//! # Architecture
//!
//! The lighting system uses a uniform buffer at descriptor set 0, binding 2 to pass lighting
//! data to shaders. The buffer contains:
//! - Arrays of directional light data
//! - Arrays of point light data
//! - Light counts for each type
//! - Global ambient lighting color
//!
//! # Memory Layout and Alignment
//!
//! The `LightingUniforms` struct uses std140 layout, which is required for uniform buffers
//! in GLSL. Key alignment rules:
//!
//! - **Vec3 alignment**: 16 bytes (same as Vec4 in std140)
//!   - A vec3 takes up 16 bytes even though only 12 are used
//!   - This is why we use padding fields in light structures
//!
//! - **Array stride**: Each array element must be aligned to 16 bytes
//!   - Even for scalar values, array elements have 16-byte stride in std140
//!
//! - **Struct alignment**: Must be a multiple of 16 bytes
//!   - The entire struct size must be a multiple of the alignment of its largest member
//!
//! ## Why Arrays?
//!
//! We use fixed-size arrays instead of dynamic buffers for several reasons:
//!
//! 1. **Shader compatibility**: GLSL requires knowing array sizes at compile time
//! 2. **Performance**: Fixed-size buffers avoid dynamic allocation overhead
//! 3. **Simplicity**: No need for complex buffer resizing or indirection
//! 4. **Predictability**: Memory layout is known at compile time
//!
//! The array sizes (8 directional, 16 point lights) are chosen to balance:
//! - Memory usage (smaller = less VRAM)
//! - Flexibility (larger = more lights)
//! - Common use cases (most scenes use < 10 lights)
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_graphics::lighting::{LightingUniforms, DirectionalLightData, PointLightData};
//! use praxis_graphics::RenderContext;
//!
//! # async fn example(mut render_context: RenderContext) -> praxis_utils::Result<()> {
//! let mut lighting = LightingUniforms::default();
//!
//! // Add a sun-like directional light
//! lighting.directional_lights[0] = DirectionalLightData {
//!     direction: [0.5, -1.0, 0.3, 0.0], // Direction + padding
//!     color: [1.0, 0.95, 0.8, 0.0],     // Color + padding
//!     intensity: 1.0,
//!     _padding: [0.0; 3],               // Align to 16 bytes
//! };
//! lighting.directional_light_count = 1;
//!
//! // Add a point light
//! lighting.point_lights[0] = PointLightData {
//!     position: [0.0, 5.0, 0.0, 0.0],  // Position + padding
//!     color: [1.0, 0.9, 0.7, 0.0],     // Color + padding
//!     intensity: 10.0,
//!     range: 20.0,
//!     _padding: [0.0; 2],              // Align to 16 bytes
//! };
//! lighting.point_light_count = 1;
//!
//! // Update the lighting buffer in the render context
//! render_context.lighting_buffer_mut().update(&lighting)?;
//! # Ok(())
//! # }
//! ```

use praxis_utils::{debug, eyre, info, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
};

/// Maximum number of directional lights supported.
///
/// This limit is chosen to balance memory usage with typical scene requirements.
/// Most real-time scenes use 1-2 directional lights (sun, moon, etc.).
pub const MAX_DIRECTIONAL_LIGHTS: usize = 8;

/// Maximum number of point lights supported.
///
/// This limit allows for moderately complex lighting scenarios without excessive
/// memory usage. Point lights are more common than directional lights in indoor scenes.
pub const MAX_POINT_LIGHTS: usize = 16;

/// Directional light data for shader consumption (std140 layout).
///
/// A directional light has a direction but no position (infinite distance, like the sun).
/// It affects all objects equally regardless of their position.
///
/// # Memory Layout (std140)
///
/// ```text
/// Offset | Field      | Size  | Alignment
/// -------|------------|-------|----------
/// 0      | direction  | 16    | 16  (vec4 in std140)
/// 16     | color      | 16    | 16  (vec4 in std140)
/// 32     | intensity  | 4     | 4   (float)
/// 36     | _padding   | 12    | 4   (array of 3 floats)
/// Total: 48 bytes (multiple of 16)
/// ```
///
/// The padding ensures the struct size is a multiple of 16 bytes, which is required
/// for array elements in std140 layout.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightData {
    /// Light direction (xyz) + padding (w).
    ///
    /// The direction points *from* the light source (i.e., toward where light travels).
    /// Should be normalized. The w component is padding to match std140 vec3 alignment.
    pub direction: [f32; 4],

    /// Light color (rgb) + padding (a).
    ///
    /// RGB color values typically in range [0, 1]. The w component is padding.
    pub color: [f32; 4],

    /// Light intensity multiplier.
    ///
    /// Scales the light's contribution. Values > 1.0 create brighter lights,
    /// values < 1.0 create dimmer lights.
    pub intensity: f32,

    /// Padding to align struct to 16-byte boundary.
    ///
    /// In std140, array elements must have stride that's a multiple of 16 bytes.
    /// This padding ensures DirectionalLightData is 48 bytes (3 * 16).
    pub _padding: [f32; 3],
}

impl Default for DirectionalLightData {
    fn default() -> Self {
        Self {
            direction: [0.0, -1.0, 0.0, 0.0], // Straight down
            color: [1.0, 1.0, 1.0, 0.0],      // White
            intensity: 0.0,                    // Disabled by default
            _padding: [0.0; 3],
        }
    }
}

/// Point light data for shader consumption (std140 layout).
///
/// A point light has a position and radiates light in all directions with
/// distance-based attenuation.
///
/// # Memory Layout (std140)
///
/// ```text
/// Offset | Field      | Size  | Alignment
/// -------|------------|-------|----------
/// 0      | position   | 16    | 16  (vec4 in std140)
/// 16     | color      | 16    | 16  (vec4 in std140)
/// 32     | intensity  | 4     | 4   (float)
/// 36     | range      | 4     | 4   (float)
/// 40     | _padding   | 8     | 4   (array of 2 floats)
/// Total: 48 bytes (multiple of 16)
/// ```
///
/// The padding ensures the struct size is a multiple of 16 bytes for array alignment.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightData {
    /// Light position (xyz) + padding (w).
    ///
    /// World-space position of the light source. The w component is padding.
    pub position: [f32; 4],

    /// Light color (rgb) + padding (a).
    ///
    /// RGB color values typically in range [0, 1]. The w component is padding.
    pub color: [f32; 4],

    /// Light intensity multiplier.
    ///
    /// Scales the light's contribution. Higher values create brighter lights.
    /// This is multiplied by the color and attenuation.
    pub intensity: f32,

    /// Maximum range of the light in world units.
    ///
    /// Beyond this distance, the light has no effect. This allows for
    /// optimization by culling distant lights.
    pub range: f32,

    /// Padding to align struct to 16-byte boundary.
    ///
    /// Ensures PointLightData is 48 bytes (3 * 16) for std140 array alignment.
    pub _padding: [f32; 2],
}

impl Default for PointLightData {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0, 0.0],
            intensity: 0.0, // Disabled by default
            range: 10.0,
            _padding: [0.0; 2],
        }
    }
}

/// Lighting uniforms passed to shaders (std140 layout).
///
/// This structure contains all lighting data needed by the fragment shader to compute
/// lighting for each pixel. It's bound at descriptor set 0, binding 2.
///
/// # Memory Layout (std140)
///
/// The struct is carefully laid out to match std140 alignment rules:
///
/// ```text
/// Offset | Field                    | Size    | Alignment
/// -------|--------------------------|---------|----------
/// 0      | directional_lights       | 384     | 16  (array of 8 * 48 bytes)
/// 384    | point_lights             | 768     | 16  (array of 16 * 48 bytes)
/// 1152   | ambient_color            | 16      | 16  (vec4)
/// 1168   | directional_light_count  | 4       | 4   (uint)
/// 1172   | point_light_count        | 4       | 4   (uint)
/// 1176   | _padding                 | 8       | 4   (array of 2 uints)
/// Total: 1184 bytes (74 * 16)
/// ```
///
/// The total size is 1184 bytes, which is well within typical uniform buffer limits
/// (minimum guaranteed by Vulkan is 16KB).
///
/// # Usage in Shaders
///
/// In GLSL, this is declared as:
///
/// ```glsl
/// layout(set = 0, binding = 2, std140) uniform LightingData {
///     DirectionalLight directional_lights[8];
///     PointLight point_lights[16];
///     vec4 ambient_color;
///     uint directional_light_count;
///     uint point_light_count;
/// } lighting;
/// ```
///
/// The shader loops through active lights using the count fields:
///
/// ```glsl
/// for (uint i = 0; i < lighting.directional_light_count; i++) {
///     // Process lighting.directional_lights[i]
/// }
/// ```
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightingUniforms {
    /// Array of directional light data.
    ///
    /// Use `directional_light_count` to determine how many lights are active.
    /// Inactive lights (beyond the count) are ignored by the shader.
    pub directional_lights: [DirectionalLightData; MAX_DIRECTIONAL_LIGHTS],

    /// Array of point light data.
    ///
    /// Use `point_light_count` to determine how many lights are active.
    /// Inactive lights (beyond the count) are ignored by the shader.
    pub point_lights: [PointLightData; MAX_POINT_LIGHTS],

    /// Global ambient light color (rgb) + padding (a).
    ///
    /// This is a constant base illumination applied to all objects,
    /// preventing them from being completely black in shadow.
    pub ambient_color: [f32; 4],

    /// Number of active directional lights (0 to MAX_DIRECTIONAL_LIGHTS).
    ///
    /// The shader loops through `directional_lights[0..directional_light_count]`.
    pub directional_light_count: u32,

    /// Number of active point lights (0 to MAX_POINT_LIGHTS).
    ///
    /// The shader loops through `point_lights[0..point_light_count]`.
    pub point_light_count: u32,

    /// Padding to ensure the struct size is a multiple of 16 bytes.
    ///
    /// While not strictly necessary for a single buffer, this maintains
    /// alignment consistency and makes the struct safe for use in arrays.
    pub _padding: [u32; 2],
}

impl Default for LightingUniforms {
    fn default() -> Self {
        Self {
            directional_lights: [DirectionalLightData::default(); MAX_DIRECTIONAL_LIGHTS],
            point_lights: [PointLightData::default(); MAX_POINT_LIGHTS],
            ambient_color: [0.1, 0.1, 0.1, 0.0], // Soft ambient lighting
            directional_light_count: 0,
            point_light_count: 0,
            _padding: [0; 2],
        }
    }
}

/// Manages the lighting uniform buffer for the renderer.
///
/// This struct handles creating and updating the uniform buffer that contains
/// lighting data passed to shaders. The buffer is host-visible and can be
/// updated every frame.
pub struct LightingUniformBuffer {
    /// The uniform buffer containing lighting data.
    buffer: Subbuffer<LightingUniforms>,
}

impl LightingUniformBuffer {
    /// Creates a new lighting uniform buffer with default lighting data.
    ///
    /// The buffer is created as host-visible so it can be updated by the CPU
    /// each frame without needing staging buffers.
    ///
    /// # Arguments
    ///
    /// * `memory_allocator` - Allocator for creating the buffer
    ///
    /// # Returns
    ///
    /// A new `LightingUniformBuffer` initialized with default lighting.
    ///
    /// # Errors
    ///
    /// Returns an error if buffer creation fails.
    pub fn new(memory_allocator: Arc<StandardMemoryAllocator>) -> Result<Self> {
        info!("Creating lighting uniform buffer");
        debug!(
            "Lighting buffer size: {} bytes ({} directional lights, {} point lights)",
            std::mem::size_of::<LightingUniforms>(),
            MAX_DIRECTIONAL_LIGHTS,
            MAX_POINT_LIGHTS
        );

        let buffer = Buffer::from_data(
            memory_allocator,
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            LightingUniforms::default(),
        )
        .map_err(|e| eyre::eyre!("Failed to create lighting uniform buffer: {}", e))?;

        info!("Lighting uniform buffer created successfully");

        Ok(Self { buffer })
    }

    /// Updates the lighting uniform buffer with new lighting data.
    ///
    /// This writes the new lighting data to the host-visible buffer.
    /// The update is immediate and will be visible to shaders in the next frame.
    ///
    /// # Arguments
    ///
    /// * `lighting` - The new lighting data to upload
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer write fails.
    pub fn update(&mut self, lighting: &LightingUniforms) -> Result<()> {
        let mut write_guard = self
            .buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock lighting buffer for writing: {}", e))?;

        *write_guard = *lighting;

        debug!(
            "Updated lighting buffer: {} directional, {} point lights",
            lighting.directional_light_count, lighting.point_light_count
        );

        Ok(())
    }

    /// Returns a reference to the underlying buffer.
    ///
    /// This can be used to bind the buffer to descriptor sets.
    pub fn buffer(&self) -> &Subbuffer<LightingUniforms> {
        &self.buffer
    }
}
