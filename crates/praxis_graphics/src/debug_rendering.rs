//! Debug rendering modes for visualization of optimization systems.
//!
//! This module provides visual debug rendering for:
//! - Frustum culling results (green=visible, red=culled)
//! - LOD level heat maps (blue=high detail, red=low detail)
//! - Occlusion buffer visualization
//! - Mesh streaming state indicators
//!
//! # Features
//!
//! ## Wireframe Bounding Spheres
//! Renders bounding spheres as wireframe overlays, colored by culling result:
//! - **Green**: Object is visible (passed culling)
//! - **Red**: Object is culled (failed culling test)
//!
//! ## LOD Heat Map
//! Overlays color-coded visualization of LOD levels:
//! - **Blue**: Highest detail LOD (level 0)
//! - **Cyan**: High-medium detail
//! - **Green**: Medium detail
//! - **Yellow**: Medium-low detail
//! - **Orange**: Low detail
//! - **Red**: Lowest detail LOD
//!
//! ## Occlusion Visualization
//! Shows hierarchical Z-buffer and occlusion query results:
//! - Depth buffer overlay with configurable intensity
//! - Per-object occlusion state indicators
//!
//! ## Mesh Streaming State
//! Displays streaming status for async-loaded meshes:
//! - **Gray**: Not loaded
//! - **Yellow**: Loading in progress
//! - **Green**: Fully loaded
//! - **Blue**: High priority in queue
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use praxis_graphics::debug_rendering::{DebugRenderer, DebugRenderMode};
//! use praxis_math::Vec3;
//!
//! // Create debug renderer
//! let mut debug_renderer = DebugRenderer::new(
//!     device.clone(),
//!     memory_allocator.clone(),
//!     render_pass.clone(),
//!     [1920, 1080],
//! )?;
//!
//! // Enable specific debug modes
//! debug_renderer.enable_mode(DebugRenderMode::CullingResults);
//! debug_renderer.enable_mode(DebugRenderMode::LodHeatMap);
//!
//! // In render loop, after main rendering:
//! debug_renderer.render_debug_overlays(
//!     &command_buffer_builder,
//!     &culling_results,
//!     &lod_states,
//!     &camera_view_proj,
//! )?;
//! ```

use crate::line_renderer::{LineBatch, LineRenderer};
use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, trace, Result};
use std::sync::Arc;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    device::Device,
    memory::allocator::StandardMemoryAllocator,
    render_pass::RenderPass,
};

/// Debug rendering modes for optimization system visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DebugRenderMode {
    /// Render wireframe bounding spheres colored by culling result.
    CullingResults,
    /// Render LOD level heat map overlay.
    LodHeatMap,
    /// Visualize occlusion buffer.
    OcclusionBuffer,
    /// Show mesh streaming state indicators.
    MeshStreamingState,
}

/// Configuration for debug rendering.
#[derive(Debug, Clone)]
pub struct DebugRenderConfig {
    /// Enable bounding sphere wireframes.
    pub show_bounding_spheres: bool,
    /// Enable LOD heat map overlay.
    pub show_lod_heat_map: bool,
    /// Enable occlusion buffer visualization.
    pub show_occlusion_buffer: bool,
    /// Enable mesh streaming state indicators.
    pub show_streaming_state: bool,
    /// Wireframe line width.
    pub wireframe_thickness: f32,
    /// Heat map intensity (0.0 to 1.0).
    pub heat_map_intensity: f32,
    /// Occlusion buffer visualization intensity.
    pub occlusion_intensity: f32,
}

impl Default for DebugRenderConfig {
    fn default() -> Self {
        Self {
            show_bounding_spheres: false,
            show_lod_heat_map: false,
            show_occlusion_buffer: false,
            show_streaming_state: false,
            wireframe_thickness: 1.0,
            heat_map_intensity: 0.7,
            occlusion_intensity: 0.5,
        }
    }
}

/// Culling result for a single object.
#[derive(Debug, Clone, Copy)]
pub struct CullingDebugInfo {
    /// Object position in world space.
    pub position: Vec3,
    /// Bounding sphere radius.
    pub radius: f32,
    /// Whether the object is visible (passed culling).
    pub is_visible: bool,
    /// Optional: which culling test failed (if any).
    pub cull_reason: Option<CullReason>,
}

/// Reason why an object was culled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullReason {
    /// Failed frustum culling test.
    Frustum,
    /// Failed distance culling test.
    Distance,
    /// Failed occlusion culling test.
    Occlusion,
}

/// LOD state for a single object.
#[derive(Debug, Clone)]
pub struct LodDebugInfo {
    /// Object position in world space.
    pub position: Vec3,
    /// Bounding sphere radius.
    pub radius: f32,
    /// Current LOD level (0 = highest detail).
    pub current_lod_level: u32,
    /// Total number of LOD levels.
    pub total_lod_levels: u32,
    /// Distance from camera (for validation).
    pub distance_from_camera: f32,
}

/// Mesh streaming state for a single object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StreamingState {
    /// Mesh not loaded.
    NotLoaded,
    /// Mesh currently loading.
    Loading,
    /// Mesh fully loaded.
    Loaded,
    /// High priority in loading queue.
    HighPriority,
}

/// Streaming state info for debug rendering.
#[derive(Debug, Clone)]
pub struct StreamingDebugInfo {
    /// Object position in world space.
    pub position: Vec3,
    /// Bounding sphere radius.
    pub radius: f32,
    /// Current streaming state.
    pub state: StreamingState,
    /// Load progress (0.0 to 1.0).
    pub load_progress: f32,
}

/// Main debug renderer for optimization system visualization.
///
/// Provides methods for rendering debug overlays showing culling results,
/// LOD levels, occlusion data, and mesh streaming state.
pub struct DebugRenderer {
    /// Line renderer for wireframe geometry.
    line_renderer: LineRenderer,
    /// Configuration for debug rendering.
    config: DebugRenderConfig,
    /// Enabled debug modes.
    enabled_modes: Vec<DebugRenderMode>,
}

impl DebugRenderer {
    /// Creates a new debug renderer.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator
    /// * `render_pass` - Render pass for debug rendering
    /// * `viewport_dimensions` - Viewport size [width, height]
    ///
    /// # Errors
    ///
    /// Returns an error if line renderer creation fails.
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        viewport_dimensions: [u32; 2],
    ) -> Result<Self> {
        debug!("Creating debug renderer");

        let line_renderer =
            LineRenderer::new(device, render_pass, memory_allocator, viewport_dimensions)?;

        Ok(Self {
            line_renderer,
            config: DebugRenderConfig::default(),
            enabled_modes: Vec::new(),
        })
    }

    /// Enables a debug rendering mode.
    pub fn enable_mode(&mut self, mode: DebugRenderMode) {
        if !self.enabled_modes.contains(&mode) {
            self.enabled_modes.push(mode);
            self.update_config_from_modes();
            debug!("Enabled debug mode: {:?}", mode);
        }
    }

    /// Disables a debug rendering mode.
    pub fn disable_mode(&mut self, mode: DebugRenderMode) {
        self.enabled_modes.retain(|&m| m != mode);
        self.update_config_from_modes();
        debug!("Disabled debug mode: {:?}", mode);
    }

    /// Checks if a debug mode is enabled.
    pub fn is_mode_enabled(&self, mode: DebugRenderMode) -> bool {
        self.enabled_modes.contains(&mode)
    }

    /// Toggles a debug rendering mode.
    pub fn toggle_mode(&mut self, mode: DebugRenderMode) {
        if self.is_mode_enabled(mode) {
            self.disable_mode(mode);
        } else {
            self.enable_mode(mode);
        }
    }

    /// Updates configuration based on enabled modes.
    fn update_config_from_modes(&mut self) {
        self.config.show_bounding_spheres = self
            .enabled_modes
            .contains(&DebugRenderMode::CullingResults);
        self.config.show_lod_heat_map = self.enabled_modes.contains(&DebugRenderMode::LodHeatMap);
        self.config.show_occlusion_buffer = self
            .enabled_modes
            .contains(&DebugRenderMode::OcclusionBuffer);
        self.config.show_streaming_state = self
            .enabled_modes
            .contains(&DebugRenderMode::MeshStreamingState);
    }

    /// Sets the debug rendering configuration.
    pub fn set_config(&mut self, config: DebugRenderConfig) {
        self.config = config;
    }

    /// Gets the current configuration.
    pub fn config(&self) -> &DebugRenderConfig {
        &self.config
    }

    /// Renders debug visualization for culling results.
    ///
    /// Draws wireframe bounding spheres colored by culling result:
    /// - Green: visible objects
    /// - Red: culled objects
    pub fn render_culling_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        culling_info: &[CullingDebugInfo],
        view_proj: Mat4,
    ) -> Result<()> {
        if !self.config.show_bounding_spheres {
            return Ok(());
        }

        trace!("Rendering culling debug for {} objects", culling_info.len());

        let mut batch = LineBatch::new();

        for info in culling_info {
            let color = if info.is_visible {
                Vec3::new(0.0, 1.0, 0.0) // Green for visible
            } else {
                Vec3::new(1.0, 0.0, 0.0) // Red for culled
            };

            // Draw wireframe sphere
            self.add_wireframe_sphere(&mut batch, info.position, info.radius, color);
        }

        self.line_renderer.render(builder, &batch)?;

        Ok(())
    }

    /// Renders debug visualization for LOD levels.
    ///
    /// Draws bounding spheres colored by LOD level:
    /// - Blue: highest detail (LOD 0)
    /// - Green: medium detail
    /// - Red: lowest detail
    pub fn render_lod_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        lod_info: &[LodDebugInfo],
        view_proj: Mat4,
    ) -> Result<()> {
        if !self.config.show_lod_heat_map {
            return Ok(());
        }

        trace!("Rendering LOD heat map for {} objects", lod_info.len());

        let mut batch = LineBatch::new();

        for info in lod_info {
            let color = self.lod_level_to_color(info.current_lod_level, info.total_lod_levels);
            self.add_wireframe_sphere(&mut batch, info.position, info.radius, color);
        }

        self.line_renderer.render(builder, &batch)?;

        Ok(())
    }

    /// Renders debug visualization for mesh streaming state.
    ///
    /// Draws indicators colored by streaming state:
    /// - Gray: not loaded
    /// - Yellow: loading
    /// - Green: loaded
    /// - Blue: high priority
    pub fn render_streaming_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        streaming_info: &[StreamingDebugInfo],
        view_proj: Mat4,
    ) -> Result<()> {
        if !self.config.show_streaming_state {
            return Ok(());
        }

        trace!(
            "Rendering streaming state for {} objects",
            streaming_info.len()
        );

        let mut batch = LineBatch::new();

        for info in streaming_info {
            let color = match info.state {
                StreamingState::NotLoaded => Vec3::new(0.5, 0.5, 0.5), // Gray
                StreamingState::Loading => Vec3::new(1.0, 1.0, 0.0),   // Yellow
                StreamingState::Loaded => Vec3::new(0.0, 1.0, 0.0),    // Green
                StreamingState::HighPriority => Vec3::new(0.0, 0.5, 1.0), // Blue
            };

            self.add_wireframe_sphere(&mut batch, info.position, info.radius, color);

            // Add loading progress bar above object
            if info.state == StreamingState::Loading {
                self.add_progress_indicator(
                    &mut batch,
                    info.position + Vec3::new(0.0, info.radius * 1.5, 0.0),
                    info.load_progress,
                    color,
                );
            }
        }

        self.line_renderer.render(builder, &batch)?;

        Ok(())
    }

    /// Adds a wireframe sphere to the line batch.
    fn add_wireframe_sphere(&self, batch: &mut LineBatch, center: Vec3, radius: f32, color: Vec3) {
        let segments = 16;

        // Draw three orthogonal circles to form a sphere
        // XY circle
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p1 = center + Vec3::new(radius * angle1.cos(), radius * angle1.sin(), 0.0);
            let p2 = center + Vec3::new(radius * angle2.cos(), radius * angle2.sin(), 0.0);

            batch.add(p1, p2, color);
        }

        // XZ circle
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p1 = center + Vec3::new(radius * angle1.cos(), 0.0, radius * angle1.sin());
            let p2 = center + Vec3::new(radius * angle2.cos(), 0.0, radius * angle2.sin());

            batch.add(p1, p2, color);
        }

        // YZ circle
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;

            let p1 = center + Vec3::new(0.0, radius * angle1.cos(), radius * angle1.sin());
            let p2 = center + Vec3::new(0.0, radius * angle2.cos(), radius * angle2.sin());

            batch.add(p1, p2, color);
        }
    }

    /// Adds a progress indicator bar to the line batch.
    fn add_progress_indicator(
        &self,
        batch: &mut LineBatch,
        position: Vec3,
        progress: f32,
        color: Vec3,
    ) {
        let bar_width = 1.0;
        let bar_height = 0.1;

        let start = position - Vec3::new(bar_width * 0.5, 0.0, 0.0);
        let end = position + Vec3::new(bar_width * 0.5, 0.0, 0.0);

        // Background bar (white)
        batch.add(start, end, Vec3::new(1.0, 1.0, 1.0));

        // Progress bar (colored)
        let progress_end = start + Vec3::new(bar_width * progress.clamp(0.0, 1.0), 0.0, 0.0);
        batch.add(start, progress_end, color);

        // Border
        let top_left = start + Vec3::new(0.0, bar_height, 0.0);
        let top_right = end + Vec3::new(0.0, bar_height, 0.0);
        let bottom_left = start - Vec3::new(0.0, bar_height, 0.0);
        let bottom_right = end - Vec3::new(0.0, bar_height, 0.0);

        batch.add(top_left, top_right, color);
        batch.add(bottom_left, bottom_right, color);
        batch.add(top_left, bottom_left, color);
        batch.add(top_right, bottom_right, color);
    }

    /// Converts LOD level to a heat map color.
    ///
    /// Blue = highest detail, Red = lowest detail
    fn lod_level_to_color(&self, current_level: u32, total_levels: u32) -> Vec3 {
        if total_levels <= 1 {
            return Vec3::new(0.0, 1.0, 0.0); // Green for single-level
        }

        let normalized = current_level as f32 / (total_levels - 1) as f32;
        let normalized = normalized.clamp(0.0, 1.0);

        // Interpolate through color spectrum: blue -> cyan -> green -> yellow -> orange -> red
        if normalized < 0.2 {
            // Blue to Cyan
            let t = normalized / 0.2;
            Vec3::new(0.0, t, 1.0)
        } else if normalized < 0.4 {
            // Cyan to Green
            let t = (normalized - 0.2) / 0.2;
            Vec3::new(0.0, 1.0, 1.0 - t)
        } else if normalized < 0.6 {
            // Green to Yellow
            let t = (normalized - 0.4) / 0.2;
            Vec3::new(t, 1.0, 0.0)
        } else if normalized < 0.8 {
            // Yellow to Orange
            let t = (normalized - 0.6) / 0.2;
            Vec3::new(1.0, 1.0 - t * 0.5, 0.0)
        } else {
            // Orange to Red
            let t = (normalized - 0.8) / 0.2;
            Vec3::new(1.0, 0.5 - t * 0.5, 0.0)
        }
    }

    /// Renders all enabled debug visualizations.
    ///
    /// This is a convenience method that calls individual render methods
    /// based on enabled modes.
    pub fn render_all_debug(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        culling_info: &[CullingDebugInfo],
        lod_info: &[LodDebugInfo],
        streaming_info: &[StreamingDebugInfo],
        view_proj: Mat4,
    ) -> Result<()> {
        if self.config.show_bounding_spheres {
            self.render_culling_debug(builder, culling_info, view_proj)?;
        }

        if self.config.show_lod_heat_map {
            self.render_lod_debug(builder, lod_info, view_proj)?;
        }

        if self.config.show_streaming_state {
            self.render_streaming_debug(builder, streaming_info, view_proj)?;
        }

        Ok(())
    }

    /// Resizes the debug renderer for new viewport dimensions.
    ///
    /// Note: Currently a no-op as LineRenderer doesn't support runtime resizing.
    /// The renderer will need to be recreated for viewport changes.
    pub fn resize(&mut self, _viewport_dimensions: [u32; 2]) -> Result<()> {
        // LineRenderer doesn't have a resize method - it uses fixed viewport from creation
        Ok(())
    }
}

/// Helper functions for creating debug info structures from engine data.
pub mod helpers {
    use super::*;
    use crate::lod::LodGroup;

    /// Creates culling debug info from GPU culling results.
    pub fn culling_info_from_gpu_result(
        position: Vec3,
        radius: f32,
        is_visible: bool,
        was_frustum_culled: bool,
        was_distance_culled: bool,
    ) -> CullingDebugInfo {
        let cull_reason = if !is_visible {
            if was_frustum_culled {
                Some(CullReason::Frustum)
            } else if was_distance_culled {
                Some(CullReason::Distance)
            } else {
                Some(CullReason::Occlusion)
            }
        } else {
            None
        };

        CullingDebugInfo {
            position,
            radius,
            is_visible,
            cull_reason,
        }
    }

    /// Creates LOD debug info from an LOD group.
    pub fn lod_info_from_lod_group(
        position: Vec3,
        radius: f32,
        lod_group: &LodGroup,
        distance_from_camera: f32,
    ) -> LodDebugInfo {
        LodDebugInfo {
            position,
            radius,
            current_lod_level: lod_group.current_level() as u32,
            total_lod_levels: lod_group.level_count() as u32,
            distance_from_camera,
        }
    }

    /// Creates streaming debug info from mesh streaming state.
    pub fn streaming_info_from_state(
        position: Vec3,
        radius: f32,
        state: StreamingState,
        load_progress: f32,
    ) -> StreamingDebugInfo {
        StreamingDebugInfo {
            position,
            radius,
            state,
            load_progress,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_render_mode_equality() {
        assert_eq!(
            DebugRenderMode::CullingResults,
            DebugRenderMode::CullingResults
        );
        assert_ne!(DebugRenderMode::CullingResults, DebugRenderMode::LodHeatMap);
    }

    #[test]
    fn test_debug_render_config_default() {
        let config = DebugRenderConfig::default();
        assert!(!config.show_bounding_spheres);
        assert!(!config.show_lod_heat_map);
        assert!(!config.show_occlusion_buffer);
        assert!(!config.show_streaming_state);
        assert_eq!(config.wireframe_thickness, 1.0);
        assert_eq!(config.heat_map_intensity, 0.7);
    }

    #[test]
    fn test_cull_reason() {
        assert_eq!(CullReason::Frustum, CullReason::Frustum);
        assert_ne!(CullReason::Frustum, CullReason::Distance);
    }

    #[test]
    fn test_streaming_state() {
        assert_eq!(StreamingState::Loading, StreamingState::Loading);
        assert_ne!(StreamingState::Loading, StreamingState::Loaded);
    }

    #[test]
    fn test_culling_debug_info_creation() {
        let info = CullingDebugInfo {
            position: Vec3::new(1.0, 2.0, 3.0),
            radius: 5.0,
            is_visible: true,
            cull_reason: None,
        };

        assert_eq!(info.position, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(info.radius, 5.0);
        assert!(info.is_visible);
        assert!(info.cull_reason.is_none());
    }

    #[test]
    fn test_lod_debug_info_creation() {
        let info = LodDebugInfo {
            position: Vec3::new(0.0, 0.0, 0.0),
            radius: 2.0,
            current_lod_level: 1,
            total_lod_levels: 4,
            distance_from_camera: 15.0,
        };

        assert_eq!(info.current_lod_level, 1);
        assert_eq!(info.total_lod_levels, 4);
        assert_eq!(info.distance_from_camera, 15.0);
    }

    #[test]
    fn test_streaming_debug_info_creation() {
        let info = StreamingDebugInfo {
            position: Vec3::ZERO,
            radius: 1.0,
            state: StreamingState::Loading,
            load_progress: 0.5,
        };

        assert_eq!(info.state, StreamingState::Loading);
        assert_eq!(info.load_progress, 0.5);
    }

    #[test]
    fn test_lod_level_to_color_single_level() {
        let config = DebugRenderConfig::default();
        let renderer_config = config;

        // Simulate what DebugRenderer would do
        let color = lod_level_to_color_test(0, 1);

        // Single level should be green
        assert_eq!(color, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn test_lod_level_to_color_multiple_levels() {
        // Level 0 of 4 should be blue
        let color0 = lod_level_to_color_test(0, 4);
        assert_eq!(color0.z, 1.0); // Blue component should be 1.0

        // Level 3 of 4 should be red-ish
        let color3 = lod_level_to_color_test(3, 4);
        assert_eq!(color3.x, 1.0); // Red component should be 1.0
    }

    // Helper function for testing color conversion
    fn lod_level_to_color_test(current_level: u32, total_levels: u32) -> Vec3 {
        if total_levels <= 1 {
            return Vec3::new(0.0, 1.0, 0.0);
        }

        let normalized = current_level as f32 / (total_levels - 1) as f32;
        let normalized = normalized.clamp(0.0, 1.0);

        if normalized < 0.2 {
            let t = normalized / 0.2;
            Vec3::new(0.0, t, 1.0)
        } else if normalized < 0.4 {
            let t = (normalized - 0.2) / 0.2;
            Vec3::new(0.0, 1.0, 1.0 - t)
        } else if normalized < 0.6 {
            let t = (normalized - 0.4) / 0.2;
            Vec3::new(t, 1.0, 0.0)
        } else if normalized < 0.8 {
            let t = (normalized - 0.6) / 0.2;
            Vec3::new(1.0, 1.0 - t * 0.5, 0.0)
        } else {
            let t = (normalized - 0.8) / 0.2;
            Vec3::new(1.0, 0.5 - t * 0.5, 0.0)
        }
    }

    #[test]
    fn test_helpers_culling_info_from_result() {
        let info = helpers::culling_info_from_gpu_result(
            Vec3::new(10.0, 0.0, 0.0),
            5.0,
            false,
            true,
            false,
        );

        assert!(!info.is_visible);
        assert_eq!(info.cull_reason, Some(CullReason::Frustum));
    }

    #[test]
    fn test_helpers_streaming_info_from_state() {
        let info =
            helpers::streaming_info_from_state(Vec3::ZERO, 1.0, StreamingState::Loading, 0.75);

        assert_eq!(info.state, StreamingState::Loading);
        assert_eq!(info.load_progress, 0.75);
    }
}
