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
            intensity: 0.0,                   // Disabled by default
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_directional_light_data_default() {
        let light = DirectionalLightData::default();

        assert_eq!(light.direction, [0.0, -1.0, 0.0, 0.0]);
        assert_eq!(light.color, [1.0, 1.0, 1.0, 0.0]);
        assert_eq!(light.intensity, 0.0);
        assert_eq!(light._padding, [0.0; 3]);
    }

    #[test]
    fn test_directional_light_data_creation() {
        let light = DirectionalLightData {
            direction: [0.5, -0.8, 0.3, 0.0],
            color: [1.0, 0.9, 0.8, 0.0],
            intensity: 1.5,
            _padding: [0.0; 3],
        };

        assert_eq!(light.direction[0], 0.5);
        assert_eq!(light.direction[1], -0.8);
        assert_eq!(light.direction[2], 0.3);
        assert_eq!(light.color, [1.0, 0.9, 0.8, 0.0]);
        assert_eq!(light.intensity, 1.5);
    }

    #[test]
    fn test_point_light_data_default() {
        let light = PointLightData::default();

        assert_eq!(light.position, [0.0, 0.0, 0.0, 0.0]);
        assert_eq!(light.color, [1.0, 1.0, 1.0, 0.0]);
        assert_eq!(light.intensity, 0.0);
        assert_eq!(light.range, 10.0);
        assert_eq!(light._padding, [0.0; 2]);
    }

    #[test]
    fn test_point_light_data_creation() {
        let light = PointLightData {
            position: [5.0, 10.0, 3.0, 0.0],
            color: [0.8, 0.3, 0.2, 0.0],
            intensity: 25.0,
            range: 50.0,
            _padding: [0.0; 2],
        };

        assert_eq!(light.position, [5.0, 10.0, 3.0, 0.0]);
        assert_eq!(light.color, [0.8, 0.3, 0.2, 0.0]);
        assert_eq!(light.intensity, 25.0);
        assert_eq!(light.range, 50.0);
    }

    #[test]
    fn test_lighting_uniforms_default() {
        let lighting = LightingUniforms::default();

        assert_eq!(lighting.directional_light_count, 0);
        assert_eq!(lighting.point_light_count, 0);
        assert_eq!(lighting.ambient_color, [0.1, 0.1, 0.1, 0.0]);
        assert_eq!(lighting._padding, [0; 2]);
    }

    #[test]
    fn test_lighting_uniforms_single_directional_light() {
        let mut lighting = LightingUniforms::default();

        lighting.directional_lights[0] = DirectionalLightData {
            direction: [0.0, -1.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0, 0.0],
            intensity: 1.0,
            _padding: [0.0; 3],
        };
        lighting.directional_light_count = 1;

        assert_eq!(lighting.directional_light_count, 1);
        assert_eq!(lighting.directional_lights[0].intensity, 1.0);
    }

    #[test]
    fn test_lighting_uniforms_single_point_light() {
        let mut lighting = LightingUniforms::default();

        lighting.point_lights[0] = PointLightData {
            position: [0.0, 5.0, 0.0, 0.0],
            color: [1.0, 0.5, 0.0, 0.0],
            intensity: 10.0,
            range: 20.0,
            _padding: [0.0; 2],
        };
        lighting.point_light_count = 1;

        assert_eq!(lighting.point_light_count, 1);
        assert_eq!(lighting.point_lights[0].intensity, 10.0);
        assert_eq!(lighting.point_lights[0].range, 20.0);
    }

    #[test]
    fn test_lighting_uniforms_multiple_lights() {
        let mut lighting = LightingUniforms::default();

        // Add multiple directional lights
        for i in 0..3 {
            lighting.directional_lights[i] = DirectionalLightData {
                direction: [i as f32, -1.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0, 0.0],
                intensity: (i + 1) as f32 * 0.5,
                _padding: [0.0; 3],
            };
        }
        lighting.directional_light_count = 3;

        // Add multiple point lights
        for i in 0..5 {
            lighting.point_lights[i] = PointLightData {
                position: [i as f32 * 2.0, 5.0, 0.0, 0.0],
                color: [1.0, 0.5, 0.0, 0.0],
                intensity: (i + 1) as f32 * 5.0,
                range: 10.0 + i as f32 * 2.0,
                _padding: [0.0; 2],
            };
        }
        lighting.point_light_count = 5;

        assert_eq!(lighting.directional_light_count, 3);
        assert_eq!(lighting.point_light_count, 5);

        // Verify the lights were set correctly
        assert_eq!(lighting.directional_lights[0].intensity, 0.5);
        assert_eq!(lighting.directional_lights[1].intensity, 1.0);
        assert_eq!(lighting.directional_lights[2].intensity, 1.5);

        assert_eq!(lighting.point_lights[0].intensity, 5.0);
        assert_eq!(lighting.point_lights[4].intensity, 25.0);
    }

    #[test]
    fn test_lighting_uniforms_max_directional_lights() {
        let mut lighting = LightingUniforms::default();

        // Fill all directional light slots
        for i in 0..MAX_DIRECTIONAL_LIGHTS {
            lighting.directional_lights[i] = DirectionalLightData {
                direction: [0.0, -1.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0, 0.0],
                intensity: i as f32,
                _padding: [0.0; 3],
            };
        }
        lighting.directional_light_count = MAX_DIRECTIONAL_LIGHTS as u32;

        assert_eq!(
            lighting.directional_light_count,
            MAX_DIRECTIONAL_LIGHTS as u32
        );
        assert_eq!(lighting.directional_lights[0].intensity, 0.0);
        assert_eq!(
            lighting.directional_lights[MAX_DIRECTIONAL_LIGHTS - 1].intensity,
            (MAX_DIRECTIONAL_LIGHTS - 1) as f32
        );
    }

    #[test]
    fn test_lighting_uniforms_max_point_lights() {
        let mut lighting = LightingUniforms::default();

        // Fill all point light slots
        for i in 0..MAX_POINT_LIGHTS {
            lighting.point_lights[i] = PointLightData {
                position: [i as f32, 0.0, 0.0, 0.0],
                color: [1.0, 1.0, 1.0, 0.0],
                intensity: i as f32 * 2.0,
                range: 10.0,
                _padding: [0.0; 2],
            };
        }
        lighting.point_light_count = MAX_POINT_LIGHTS as u32;

        assert_eq!(lighting.point_light_count, MAX_POINT_LIGHTS as u32);
        assert_eq!(lighting.point_lights[0].intensity, 0.0);
        assert_eq!(
            lighting.point_lights[MAX_POINT_LIGHTS - 1].intensity,
            (MAX_POINT_LIGHTS - 1) as f32 * 2.0
        );
    }

    #[test]
    fn test_lighting_uniforms_custom_ambient() {
        let mut lighting = LightingUniforms::default();
        lighting.ambient_color = [0.2, 0.3, 0.4, 0.0];

        assert_eq!(lighting.ambient_color, [0.2, 0.3, 0.4, 0.0]);
    }

    #[test]
    fn test_directional_light_data_size() {
        // Verify struct size matches std140 requirements (48 bytes = 3 * 16)
        assert_eq!(std::mem::size_of::<DirectionalLightData>(), 48);
    }

    #[test]
    fn test_point_light_data_size() {
        // Verify struct size matches std140 requirements (48 bytes = 3 * 16)
        assert_eq!(std::mem::size_of::<PointLightData>(), 48);
    }

    #[test]
    fn test_lighting_uniforms_size() {
        // Verify total struct size
        // 8 directional lights * 48 bytes = 384
        // 16 point lights * 48 bytes = 768
        // ambient_color (vec4) = 16
        // 2 u32 counts = 8
        // 2 u32 padding = 8
        // Total = 1184 bytes
        assert_eq!(std::mem::size_of::<LightingUniforms>(), 1184);
    }

    #[test]
    fn test_directional_light_data_alignment() {
        // Verify 16-byte alignment for std140
        assert_eq!(std::mem::align_of::<DirectionalLightData>(), 16);
    }

    #[test]
    fn test_point_light_data_alignment() {
        // Verify 16-byte alignment for std140
        assert_eq!(std::mem::align_of::<PointLightData>(), 16);
    }

    #[test]
    fn test_lighting_uniforms_alignment() {
        // Verify 16-byte alignment for std140
        assert_eq!(std::mem::align_of::<LightingUniforms>(), 16);
    }

    #[test]
    fn test_lighting_uniforms_zero_initialization() {
        let lighting = LightingUniforms::default();

        // Verify all unused lights are zero-initialized (intensity = 0)
        for i in 0..MAX_DIRECTIONAL_LIGHTS {
            assert_eq!(lighting.directional_lights[i].intensity, 0.0);
        }

        for i in 0..MAX_POINT_LIGHTS {
            assert_eq!(lighting.point_lights[i].intensity, 0.0);
        }
    }

    #[test]
    fn test_bytemuck_pod_trait() {
        // Verify that our types implement Pod (Plain Old Data)
        // This is required for safe GPU memory transfers
        use bytemuck::{Pod, Zeroable};

        fn assert_pod<T: Pod + Zeroable>() {}

        assert_pod::<DirectionalLightData>();
        assert_pod::<PointLightData>();
        assert_pod::<LightingUniforms>();
    }

    #[test]
    fn test_lighting_uniforms_modification() {
        let mut lighting = LightingUniforms::default();

        // Initial state
        assert_eq!(lighting.directional_light_count, 0);

        // Modify
        lighting.directional_lights[0].intensity = 1.0;
        lighting.directional_light_count = 1;

        // Verify modification
        assert_eq!(lighting.directional_light_count, 1);
        assert_eq!(lighting.directional_lights[0].intensity, 1.0);

        // Modify again
        lighting.directional_lights[1].intensity = 2.0;
        lighting.directional_light_count = 2;

        // Verify both lights
        assert_eq!(lighting.directional_light_count, 2);
        assert_eq!(lighting.directional_lights[0].intensity, 1.0);
        assert_eq!(lighting.directional_lights[1].intensity, 2.0);
    }

    #[test]
    fn test_lighting_scenario_outdoor_scene() {
        let mut lighting = LightingUniforms::default();

        // Outdoor scene: Sun + sky light
        lighting.directional_lights[0] = DirectionalLightData {
            direction: [0.3, -0.8, 0.5, 0.0],
            color: [1.0, 0.95, 0.85, 0.0], // Warm sunlight
            intensity: 1.0,
            _padding: [0.0; 3],
        };

        lighting.directional_lights[1] = DirectionalLightData {
            direction: [0.0, 0.5, 0.0, 0.0],
            color: [0.4, 0.5, 0.7, 0.0], // Cool sky light
            intensity: 0.3,
            _padding: [0.0; 3],
        };

        lighting.directional_light_count = 2;
        lighting.ambient_color = [0.15, 0.15, 0.2, 0.0]; // Slight blue ambient

        assert_eq!(lighting.directional_light_count, 2);
        assert_eq!(lighting.directional_lights[0].intensity, 1.0);
        assert_eq!(lighting.directional_lights[1].intensity, 0.3);
    }

    #[test]
    fn test_lighting_scenario_indoor_scene() {
        let mut lighting = LightingUniforms::default();

        // Indoor scene: Multiple point lights
        let positions = [
            [5.0, 3.0, 5.0, 0.0],
            [-5.0, 3.0, 5.0, 0.0],
            [5.0, 3.0, -5.0, 0.0],
            [-5.0, 3.0, -5.0, 0.0],
        ];

        for (i, &pos) in positions.iter().enumerate() {
            lighting.point_lights[i] = PointLightData {
                position: pos,
                color: [1.0, 0.9, 0.7, 0.0], // Warm interior light
                intensity: 15.0,
                range: 10.0,
                _padding: [0.0; 2],
            };
        }

        lighting.point_light_count = 4;
        lighting.ambient_color = [0.05, 0.05, 0.05, 0.0]; // Dark ambient

        assert_eq!(lighting.point_light_count, 4);
        for i in 0..4 {
            assert_eq!(lighting.point_lights[i].intensity, 15.0);
            assert_eq!(lighting.point_lights[i].range, 10.0);
        }
    }

    #[test]
    fn test_lighting_scenario_mixed() {
        let mut lighting = LightingUniforms::default();

        // Mixed lighting: Directional + Point lights
        lighting.directional_lights[0] = DirectionalLightData {
            direction: [0.0, -1.0, 0.0, 0.0],
            color: [0.8, 0.8, 1.0, 0.0],
            intensity: 0.5,
            _padding: [0.0; 3],
        };
        lighting.directional_light_count = 1;

        lighting.point_lights[0] = PointLightData {
            position: [0.0, 5.0, 0.0, 0.0],
            color: [1.0, 0.3, 0.1, 0.0], // Orange/fire
            intensity: 20.0,
            range: 15.0,
            _padding: [0.0; 2],
        };

        lighting.point_lights[1] = PointLightData {
            position: [10.0, 2.0, 0.0, 0.0],
            color: [0.1, 0.3, 1.0, 0.0], // Blue
            intensity: 15.0,
            range: 12.0,
            _padding: [0.0; 2],
        };
        lighting.point_light_count = 2;

        assert_eq!(lighting.directional_light_count, 1);
        assert_eq!(lighting.point_light_count, 2);
    }

    #[test]
    fn test_constants_are_reasonable() {
        // Verify that our array size constants are reasonable
        assert!(
            MAX_DIRECTIONAL_LIGHTS >= 4,
            "Should support at least 4 directional lights"
        );
        assert!(
            MAX_POINT_LIGHTS >= 8,
            "Should support at least 8 point lights"
        );

        // Verify buffer size is under typical limits
        let buffer_size = std::mem::size_of::<LightingUniforms>();
        assert!(
            buffer_size < 16384,
            "Buffer should be under 16KB (typical UBO limit)"
        );
    }
}
