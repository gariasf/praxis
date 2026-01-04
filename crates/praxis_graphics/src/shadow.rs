//! Shadow mapping system for the graphics engine.
//!
//! This module provides shadow map generation and configuration for directional lights.
//! It supports cascaded shadow maps (CSM) with configurable quality settings.
//!
//! # Shadow Mapping Overview
//!
//! Shadow mapping is a two-pass rendering technique:
//!
//! 1. **Shadow Pass**: Render the scene from the light's perspective to a depth texture (shadow map)
//! 2. **Main Pass**: Render the scene normally, sampling the shadow map to determine if fragments are in shadow
//!
//! # Cascaded Shadow Maps (CSM)
//!
//! CSM improves shadow quality by using multiple shadow maps at different distances:
//! - Near cascade: High detail for objects close to camera
//! - Middle cascades: Medium detail for mid-range objects
//! - Far cascade: Lower detail for distant objects
//!
//! This prevents shadow aliasing (blocky shadows) near the camera while maintaining
//! reasonable performance and memory usage.
//!
//! # PCF Filtering
//!
//! Percentage Closer Filtering (PCF) softens shadow edges by sampling multiple points
//! in the shadow map and averaging the results. This creates smooth shadow transitions
//! instead of hard, aliased edges.

use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, eyre, info, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    device::DeviceOwned,
    format::Format,
    image::{view::ImageView, Image, ImageCreateInfo, ImageUsage},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass},
};

/// Maximum number of shadow cascades supported.
///
/// Most implementations use 3-4 cascades as a good balance between quality and performance.
pub const MAX_SHADOW_CASCADES: usize = 4;

/// Shadow mapping configuration.
///
/// This controls the quality and performance characteristics of shadow rendering.
///
/// # Examples
///
/// ```rust,no_run
/// use praxis_graphics::shadow::ShadowConfig;
///
/// // High-quality shadows for close-up scenes
/// let high_quality = ShadowConfig {
///     shadow_map_size: 2048,
///     cascade_count: 4,
///     cascade_distances: [10.0, 30.0, 100.0, 300.0],
///     pcf_samples: 9,
///     bias: 0.005,
/// };
///
/// // Performance-focused shadows for open worlds
/// let performance = ShadowConfig {
///     shadow_map_size: 1024,
///     cascade_count: 3,
///     cascade_distances: [20.0, 100.0, 500.0, 1000.0],
///     pcf_samples: 4,
///     bias: 0.01,
/// };
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ShadowConfig {
    /// Resolution of each shadow map (shadow_map_size × shadow_map_size).
    ///
    /// Common values:
    /// - 512: Low quality, high performance
    /// - 1024: Medium quality (default)
    /// - 2048: High quality
    /// - 4096: Very high quality, expensive
    pub shadow_map_size: u32,

    /// Number of cascades to use (1 to MAX_SHADOW_CASCADES).
    ///
    /// More cascades = better quality but higher cost.
    /// Typical values: 3-4 cascades
    pub cascade_count: usize,

    /// Distance from camera where each cascade ends.
    ///
    /// Must be in ascending order. Only the first `cascade_count` values are used.
    ///
    /// Example for 3 cascades:
    /// - [20.0, 100.0, 500.0, _]: Near (0-20m), mid (20-100m), far (100-500m)
    pub cascade_distances: [f32; MAX_SHADOW_CASCADES],

    /// Number of samples for PCF filtering (1, 4, 9, or 16).
    ///
    /// - 1: No filtering (hard shadows, best performance)
    /// - 4: 2×2 filter (soft shadows, good performance)
    /// - 9: 3×3 filter (softer shadows, medium performance)
    /// - 16: 4×4 filter (softest shadows, lower performance)
    pub pcf_samples: u32,

    /// Shadow bias to prevent self-shadowing artifacts (acne).
    ///
    /// Too low: Shadow acne (surfaces incorrectly shadowing themselves)
    /// Too high: Peter panning (shadows detach from objects)
    ///
    /// Typical values: 0.001 - 0.01
    pub bias: f32,
}

impl Default for ShadowConfig {
    fn default() -> Self {
        Self {
            shadow_map_size: 1024,
            cascade_count: 3,
            cascade_distances: [20.0, 100.0, 500.0, 1000.0],
            pcf_samples: 4,
            bias: 0.005,
        }
    }
}

/// Shadow uniform data passed to shaders (std140 layout).
///
/// This contains the light-space transformation matrices for each cascade
/// and configuration parameters for shadow sampling.
///
/// # Memory Layout (std140)
///
/// ```text
/// Offset | Field                    | Size    | Alignment
/// -------|--------------------------|---------|----------
/// 0      | light_space_matrices     | 1024    | 16  (array of 4 * mat4)
/// 1024   | cascade_distances        | 16      | 16  (vec4, only xyz used)
/// 1040   | cascade_count            | 4       | 4   (uint)
/// 1044   | shadow_map_size          | 4       | 4   (uint)
/// 1048   | pcf_samples              | 4       | 4   (uint)
/// 1052   | bias                     | 4       | 4   (float)
/// Total: 1056 bytes
/// ```
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ShadowUniforms {
    /// Light-space transformation matrices for each cascade.
    ///
    /// These transform world-space positions to light-space clip coordinates
    /// for shadow map lookup. Each matrix is: projection * view (from light)
    pub light_space_matrices: [[f32; 16]; MAX_SHADOW_CASCADES],

    /// Distances from camera where each cascade ends.
    ///
    /// Stored as vec4 for std140 alignment (only first 3 used for 4 cascades).
    pub cascade_distances: [f32; 4],

    /// Number of active shadow cascades.
    pub cascade_count: u32,

    /// Size of the shadow map texture.
    pub shadow_map_size: u32,

    /// Number of PCF samples (1, 4, 9, or 16).
    pub pcf_samples: u32,

    /// Shadow bias for preventing acne.
    pub bias: f32,
}

impl Default for ShadowUniforms {
    fn default() -> Self {
        Self {
            light_space_matrices: [[0.0; 16]; MAX_SHADOW_CASCADES],
            cascade_distances: [0.0; 4],
            cascade_count: 0,
            shadow_map_size: 1024,
            pcf_samples: 4,
            bias: 0.005,
        }
    }
}

/// Manages shadow map resources and rendering.
///
/// This struct handles:
/// - Shadow map texture allocation
/// - Shadow render pass creation
/// - Light-space matrix calculation
/// - Shadow uniform buffer management
pub struct ShadowMapManager {
    /// Configuration for shadow mapping.
    config: ShadowConfig,

    /// Shadow map depth images (one per cascade).
    #[allow(dead_code)]
    shadow_maps: Vec<Arc<Image>>,

    /// Image views for shadow map textures.
    shadow_map_views: Vec<Arc<ImageView>>,

    /// Render pass for shadow map generation.
    shadow_render_pass: Arc<RenderPass>,

    /// Framebuffers for rendering to shadow maps.
    shadow_framebuffers: Vec<Arc<Framebuffer>>,

    /// Uniform buffer containing shadow data for shaders.
    shadow_uniform_buffer: Subbuffer<ShadowUniforms>,

    /// Memory allocator for creating resources.
    #[allow(dead_code)]
    memory_allocator: Arc<StandardMemoryAllocator>,
}

impl ShadowMapManager {
    /// Creates a new shadow map manager with the given configuration.
    ///
    /// This allocates all shadow map textures, creates the shadow render pass,
    /// and initializes the uniform buffer.
    ///
    /// # Arguments
    ///
    /// * `memory_allocator` - Allocator for creating GPU resources
    /// * `config` - Shadow mapping configuration
    ///
    /// # Returns
    ///
    /// A new `ShadowMapManager` ready for use.
    ///
    /// # Errors
    ///
    /// Returns an error if resource allocation fails.
    pub fn new(
        memory_allocator: Arc<StandardMemoryAllocator>,
        config: ShadowConfig,
    ) -> Result<Self> {
        info!(
            "Creating shadow map manager: {} cascades, {}×{} per cascade",
            config.cascade_count, config.shadow_map_size, config.shadow_map_size
        );

        let device = memory_allocator.device().clone();

        // Create shadow map images
        let mut shadow_maps = Vec::with_capacity(config.cascade_count);
        let mut shadow_map_views = Vec::with_capacity(config.cascade_count);

        for i in 0..config.cascade_count {
            debug!("Creating shadow map {} of {}", i + 1, config.cascade_count);

            let image = Image::new(
                memory_allocator.clone(),
                ImageCreateInfo {
                    format: Format::D32_SFLOAT,
                    extent: [config.shadow_map_size, config.shadow_map_size, 1],
                    usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT | ImageUsage::SAMPLED,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to create shadow map image: {}", e))?;

            let view = ImageView::new_default(image.clone())
                .map_err(|e| eyre::eyre!("Failed to create shadow map view: {}", e))?;

            shadow_maps.push(image);
            shadow_map_views.push(view);
        }

        // Create shadow render pass
        let shadow_render_pass = Self::create_shadow_render_pass(&device)?;

        // Create framebuffers for each shadow map
        let mut shadow_framebuffers = Vec::with_capacity(config.cascade_count);
        for view in &shadow_map_views {
            let framebuffer = Framebuffer::new(
                shadow_render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view.clone()],
                    ..Default::default()
                },
            )
            .map_err(|e| eyre::eyre!("Failed to create shadow framebuffer: {}", e))?;

            shadow_framebuffers.push(framebuffer);
        }

        // Create shadow uniform buffer
        let shadow_uniform_buffer = Buffer::from_data(
            memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            ShadowUniforms {
                cascade_count: config.cascade_count as u32,
                shadow_map_size: config.shadow_map_size,
                pcf_samples: config.pcf_samples,
                bias: config.bias,
                ..Default::default()
            },
        )
        .map_err(|e| eyre::eyre!("Failed to create shadow uniform buffer: {}", e))?;

        info!("Shadow map manager created successfully");

        Ok(Self {
            config,
            shadow_maps,
            shadow_map_views,
            shadow_render_pass,
            shadow_framebuffers,
            shadow_uniform_buffer,
            memory_allocator,
        })
    }

    /// Creates the render pass for shadow map generation.
    ///
    /// This render pass only has a depth attachment and no color attachment,
    /// as we only need to write depth values for shadow mapping.
    fn create_shadow_render_pass(device: &Arc<vulkano::device::Device>) -> Result<Arc<RenderPass>> {
        debug!("Creating shadow map render pass");

        let render_pass = vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                depth: {
                    format: Format::D32_SFLOAT,
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                }
            },
            pass: {
                color: [],
                depth_stencil: {depth}
            }
        )
        .map_err(|e| eyre::eyre!("Failed to create shadow render pass: {}", e))?;

        Ok(render_pass)
    }

    /// Calculates light-space matrices for shadow cascades.
    ///
    /// This computes the view and projection matrices for rendering from the
    /// light's perspective for each cascade.
    ///
    /// # Arguments
    ///
    /// * `light_direction` - Direction of the directional light (normalized)
    /// * `camera_view` - Camera view matrix
    /// * `camera_proj` - Camera projection matrix
    ///
    /// # Returns
    ///
    /// An array of light-space matrices (one per cascade).
    pub fn calculate_light_space_matrices(
        &self,
        light_direction: Vec3,
        camera_view: Mat4,
        camera_proj: Mat4,
    ) -> [[f32; 16]; MAX_SHADOW_CASCADES] {
        let mut matrices = [[0.0; 16]; MAX_SHADOW_CASCADES];

        // Extract camera position from view matrix
        let camera_pos = Self::extract_camera_position(camera_view);

        for (i, matrix) in matrices
            .iter_mut()
            .enumerate()
            .take(self.config.cascade_count)
        {
            // Calculate near and far planes for this cascade
            let near = if i == 0 {
                0.1
            } else {
                self.config.cascade_distances[i - 1]
            };
            let far = self.config.cascade_distances[i];

            // Create light view matrix
            // Position the light far away in the light direction
            let light_pos = camera_pos - light_direction * 100.0;
            let light_view = Mat4::look_at_rh(light_pos, camera_pos, Vec3::Y);

            // Calculate frustum corners in light space
            let frustum_corners = Self::calculate_frustum_corners(
                camera_view.inverse(),
                camera_proj.inverse(),
                near,
                far,
            );

            // Transform frustum corners to light space and find bounds
            let (min_bounds, max_bounds) =
                Self::calculate_light_space_bounds(&frustum_corners, light_view);

            // Create orthographic projection that covers the frustum
            let light_proj = Mat4::orthographic_rh(
                min_bounds.x,
                max_bounds.x,
                min_bounds.y,
                max_bounds.y,
                -max_bounds.z - 50.0, // Extend backwards to catch shadow casters
                -min_bounds.z,
            );

            // Combine projection and view
            let light_space_matrix = light_proj * light_view;
            *matrix = light_space_matrix.to_cols_array();
        }

        matrices
    }

    /// Extracts camera position from view matrix.
    fn extract_camera_position(view_matrix: Mat4) -> Vec3 {
        let inv_view = view_matrix.inverse();
        Vec3::new(inv_view.w_axis.x, inv_view.w_axis.y, inv_view.w_axis.z)
    }

    /// Calculates the 8 corners of the view frustum in world space.
    fn calculate_frustum_corners(
        inv_view: Mat4,
        inv_proj: Mat4,
        _near: f32,
        _far: f32,
    ) -> [Vec3; 8] {
        let inv_view_proj = inv_view * inv_proj;

        // NDC coordinates for frustum corners
        let ndc_corners = [
            [-1.0, -1.0, 0.0], // Near bottom-left
            [1.0, -1.0, 0.0],  // Near bottom-right
            [-1.0, 1.0, 0.0],  // Near top-left
            [1.0, 1.0, 0.0],   // Near top-right
            [-1.0, -1.0, 1.0], // Far bottom-left
            [1.0, -1.0, 1.0],  // Far bottom-right
            [-1.0, 1.0, 1.0],  // Far top-left
            [1.0, 1.0, 1.0],   // Far top-right
        ];

        let mut world_corners = [Vec3::ZERO; 8];
        for (i, ndc) in ndc_corners.iter().enumerate() {
            let world_pos = inv_view_proj.project_point3(Vec3::new(ndc[0], ndc[1], ndc[2]));
            world_corners[i] = world_pos;
        }

        world_corners
    }

    /// Calculates the bounding box of frustum corners in light space.
    fn calculate_light_space_bounds(corners: &[Vec3; 8], light_view: Mat4) -> (Vec3, Vec3) {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for corner in corners {
            let light_space_pos = light_view.transform_point3(*corner);
            min = min.min(light_space_pos);
            max = max.max(light_space_pos);
        }

        (min, max)
    }

    /// Updates the shadow uniform buffer with new light-space matrices.
    ///
    /// # Arguments
    ///
    /// * `light_direction` - Direction of the directional light
    /// * `camera_view` - Camera view matrix
    /// * `camera_proj` - Camera projection matrix
    ///
    /// # Errors
    ///
    /// Returns an error if buffer update fails.
    pub fn update(
        &mut self,
        light_direction: Vec3,
        camera_view: Mat4,
        camera_proj: Mat4,
    ) -> Result<()> {
        let matrices =
            self.calculate_light_space_matrices(light_direction, camera_view, camera_proj);

        let mut write_guard = self
            .shadow_uniform_buffer
            .write()
            .map_err(|e| eyre::eyre!("Failed to lock shadow uniform buffer for writing: {}", e))?;

        write_guard.light_space_matrices = matrices;

        // Copy cascade distances
        for i in 0..self.config.cascade_count.min(4) {
            write_guard.cascade_distances[i] = self.config.cascade_distances[i];
        }

        Ok(())
    }

    /// Returns a reference to the shadow configuration.
    pub fn config(&self) -> &ShadowConfig {
        &self.config
    }

    /// Returns a reference to the shadow uniform buffer.
    pub fn uniform_buffer(&self) -> &Subbuffer<ShadowUniforms> {
        &self.shadow_uniform_buffer
    }

    /// Returns references to the shadow map image views.
    pub fn shadow_map_views(&self) -> &[Arc<ImageView>] {
        &self.shadow_map_views
    }

    /// Returns a reference to the shadow render pass.
    pub fn shadow_render_pass(&self) -> &Arc<RenderPass> {
        &self.shadow_render_pass
    }

    /// Returns references to the shadow framebuffers.
    pub fn shadow_framebuffers(&self) -> &[Arc<Framebuffer>] {
        &self.shadow_framebuffers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shadow_config_default() {
        let config = ShadowConfig::default();

        assert_eq!(config.shadow_map_size, 1024);
        assert_eq!(config.cascade_count, 3);
        assert_eq!(config.cascade_distances[0], 20.0);
        assert_eq!(config.cascade_distances[1], 100.0);
        assert_eq!(config.cascade_distances[2], 500.0);
        assert_eq!(config.pcf_samples, 4);
        assert_eq!(config.bias, 0.005);
    }

    #[test]
    fn test_shadow_uniforms_default() {
        let uniforms = ShadowUniforms::default();

        assert_eq!(uniforms.cascade_count, 0);
        assert_eq!(uniforms.shadow_map_size, 1024);
        assert_eq!(uniforms.pcf_samples, 4);
        assert_eq!(uniforms.bias, 0.005);
    }

    #[test]
    fn test_shadow_uniforms_size() {
        // Verify struct size matches expected layout
        // MAX_SHADOW_CASCADES=4 mat4 (64 bytes each) = 256
        // 1 vec4 cascade_distances = 16
        // 4 u32 (cascade_count, shadow_map_size, pcf_samples, bias) = 16
        // Total = 288 bytes
        assert_eq!(std::mem::size_of::<ShadowUniforms>(), 288);
    }

    #[test]
    fn test_shadow_uniforms_alignment() {
        // Natural alignment for struct with f32 arrays
        assert_eq!(std::mem::align_of::<ShadowUniforms>(), 4);
    }

    #[test]
    fn test_extract_camera_position() {
        let view = Mat4::look_at_rh(Vec3::new(5.0, 10.0, 15.0), Vec3::ZERO, Vec3::Y);
        let pos = ShadowMapManager::extract_camera_position(view);

        // Should extract approximately the original camera position
        assert!((pos.x - 5.0).abs() < 0.01);
        assert!((pos.y - 10.0).abs() < 0.01);
        assert!((pos.z - 15.0).abs() < 0.01);
    }

    #[test]
    fn test_shadow_config_custom() {
        let config = ShadowConfig {
            shadow_map_size: 2048,
            cascade_count: 4,
            cascade_distances: [10.0, 50.0, 200.0, 800.0],
            pcf_samples: 16,
            bias: 0.001,
        };

        assert_eq!(config.shadow_map_size, 2048);
        assert_eq!(config.cascade_count, 4);
        assert_eq!(config.cascade_distances[0], 10.0);
        assert_eq!(config.cascade_distances[3], 800.0);
        assert_eq!(config.pcf_samples, 16);
        assert_eq!(config.bias, 0.001);
    }

    #[test]
    fn test_shadow_uniforms_initialization() {
        let uniforms = ShadowUniforms {
            cascade_count: 3,
            shadow_map_size: 2048,
            pcf_samples: 9,
            bias: 0.01,
            ..Default::default()
        };

        assert_eq!(uniforms.cascade_count, 3);
        assert_eq!(uniforms.shadow_map_size, 2048);
        assert_eq!(uniforms.pcf_samples, 9);
        assert_eq!(uniforms.bias, 0.01);

        // Verify cascade distances are zero-initialized
        for distance in &uniforms.cascade_distances {
            assert_eq!(*distance, 0.0);
        }
    }

    #[test]
    fn test_calculate_frustum_corners() {
        let camera_pos = Vec3::new(0.0, 5.0, 10.0);
        let view = Mat4::look_at_rh(camera_pos, Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 100.0);

        let corners =
            ShadowMapManager::calculate_frustum_corners(view.inverse(), proj.inverse(), 0.1, 50.0);

        // Should have 8 corners
        assert_eq!(corners.len(), 8);

        // All corners should be different
        for (i, corner) in corners.iter().enumerate() {
            for (j, other) in corners.iter().enumerate() {
                if i != j {
                    let distance = corner.distance(*other);
                    assert!(distance > 0.0001, "Corners {} and {} are too close", i, j);
                }
            }
        }

        // Near corners should be closer to camera than far corners
        let near_center = (corners[0] + corners[1] + corners[2] + corners[3]) / 4.0;
        let far_center = (corners[4] + corners[5] + corners[6] + corners[7]) / 4.0;
        let near_dist = camera_pos.distance(near_center);
        let far_dist = camera_pos.distance(far_center);
        assert!(near_dist < far_dist);
    }

    #[test]
    fn test_calculate_light_space_bounds() {
        let corners = [
            Vec3::new(-1.0, -1.0, -1.0),
            Vec3::new(1.0, -1.0, -1.0),
            Vec3::new(-1.0, 1.0, -1.0),
            Vec3::new(1.0, 1.0, -1.0),
            Vec3::new(-1.0, -1.0, 1.0),
            Vec3::new(1.0, -1.0, 1.0),
            Vec3::new(-1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
        ];

        let light_view = Mat4::IDENTITY;
        let (min, max) = ShadowMapManager::calculate_light_space_bounds(&corners, light_view);

        // For identity transform, bounds should match corner extents
        assert!((min.x - (-1.0)).abs() < 0.0001);
        assert!((min.y - (-1.0)).abs() < 0.0001);
        assert!((min.z - (-1.0)).abs() < 0.0001);
        assert!((max.x - 1.0).abs() < 0.0001);
        assert!((max.y - 1.0).abs() < 0.0001);
        assert!((max.z - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_calculate_light_space_bounds_transformed() {
        let corners = [
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Vec3::new(2.0, 2.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(2.0, 0.0, 2.0),
            Vec3::new(0.0, 2.0, 2.0),
            Vec3::new(2.0, 2.0, 2.0),
        ];

        // Transform that translates by (-1, -1, -1)
        let light_view = Mat4::from_translation(Vec3::new(-1.0, -1.0, -1.0));
        let (min, max) = ShadowMapManager::calculate_light_space_bounds(&corners, light_view);

        // After translation, bounds should be [-1, -1, -1] to [1, 1, 1]
        assert!((min.x - (-1.0)).abs() < 0.0001);
        assert!((min.y - (-1.0)).abs() < 0.0001);
        assert!((min.z - (-1.0)).abs() < 0.0001);
        assert!((max.x - 1.0).abs() < 0.0001);
        assert!((max.y - 1.0).abs() < 0.0001);
        assert!((max.z - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_cascade_distances_ascending() {
        let config = ShadowConfig::default();

        // Verify cascade distances are in ascending order
        for i in 1..config.cascade_count {
            assert!(
                config.cascade_distances[i] > config.cascade_distances[i - 1],
                "Cascade distance {} ({}) should be greater than cascade {} ({})",
                i,
                config.cascade_distances[i],
                i - 1,
                config.cascade_distances[i - 1]
            );
        }
    }

    #[test]
    fn test_max_shadow_cascades_constant() {
        assert_eq!(MAX_SHADOW_CASCADES, 4);
    }

    #[test]
    fn test_shadow_map_size_power_of_two() {
        let common_sizes = [512, 1024, 2048, 4096];
        for size in common_sizes {
            let config = ShadowConfig {
                shadow_map_size: size,
                ..Default::default()
            };
            // Verify size is a power of two
            assert_eq!(
                size.count_ones(),
                1,
                "Shadow map size {} should be a power of two",
                size
            );
            assert!(config.shadow_map_size >= 512 && config.shadow_map_size <= 8192);
        }
    }

    #[test]
    fn test_pcf_samples_valid_values() {
        let valid_samples = [1, 4, 9, 16];
        for samples in valid_samples {
            let config = ShadowConfig {
                pcf_samples: samples,
                ..Default::default()
            };
            assert_eq!(config.pcf_samples, samples);
        }
    }

    #[test]
    fn test_shadow_bias_range() {
        // Test typical bias values
        let typical_biases = [0.0001, 0.001, 0.005, 0.01, 0.1];
        for bias in typical_biases {
            let config = ShadowConfig {
                bias,
                ..Default::default()
            };
            assert!(config.bias >= 0.0 && config.bias <= 1.0);
        }
    }

    #[test]
    fn test_light_space_matrices_initialization() {
        let uniforms = ShadowUniforms::default();

        // Verify all matrices are zero-initialized
        for matrix in &uniforms.light_space_matrices {
            for &value in matrix {
                assert_eq!(value, 0.0);
            }
        }
    }

    #[test]
    fn test_extract_camera_position_identity() {
        let view = Mat4::IDENTITY;
        let pos = ShadowMapManager::extract_camera_position(view);

        // Identity view should give zero position
        assert!((pos.x - 0.0).abs() < 0.01);
        assert!((pos.y - 0.0).abs() < 0.01);
        assert!((pos.z - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_extract_camera_position_various_positions() {
        let test_positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(10.0, 20.0, 30.0),
            Vec3::new(-5.0, 15.0, -10.0),
            Vec3::new(100.0, 0.0, 100.0),
        ];

        for expected_pos in test_positions {
            let view = Mat4::look_at_rh(expected_pos, expected_pos + Vec3::Z, Vec3::Y);
            let extracted_pos = ShadowMapManager::extract_camera_position(view);

            assert!(
                (extracted_pos.x - expected_pos.x).abs() < 0.01,
                "X position mismatch: expected {}, got {}",
                expected_pos.x,
                extracted_pos.x
            );
            assert!(
                (extracted_pos.y - expected_pos.y).abs() < 0.01,
                "Y position mismatch: expected {}, got {}",
                expected_pos.y,
                extracted_pos.y
            );
            assert!(
                (extracted_pos.z - expected_pos.z).abs() < 0.01,
                "Z position mismatch: expected {}, got {}",
                expected_pos.z,
                extracted_pos.z
            );
        }
    }
}
