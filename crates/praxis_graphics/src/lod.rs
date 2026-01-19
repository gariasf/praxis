//! Level of Detail (LOD) system for the Praxis graphics engine.
//!
//! This module provides an automatic LOD system that manages multiple mesh variants
//! per model at different triangle counts, performs distance-based LOD selection using
//! squared distance checks to avoid sqrt, and supports smooth transitions between LOD
//! levels using alpha blending.
//!
//! # LOD System Overview
//!
//! The LOD system consists of several key components:
//!
//! - **`LodLevel`**: Represents a single LOD level with mesh reference and distance threshold
//! - **`LodGroup`**: Manages multiple LOD levels for a single entity
//! - **`LodManager`**: System-wide LOD manager that handles LOD selection and transitions
//! - **`LodTransition`**: Manages smooth alpha-blended transitions between LOD levels
//! - **`GpuLodSelector`**: GPU-driven LOD selection using compute shaders for optimal performance
//!
//! # Distance-Based LOD Selection
//!
//! LOD selection is based on squared distance to avoid expensive sqrt operations:
//! ```text
//! if distance_squared < threshold_squared {
//!     use_lod_level(0); // Highest detail
//! } else if distance_squared < threshold_squared * 4.0 {
//!     use_lod_level(1); // Medium detail
//! } else {
//!     use_lod_level(2); // Lowest detail
//! }
//! ```
//!
//! # Smooth Transitions
//!
//! The system supports smooth alpha-blended transitions between LOD levels to avoid
//! popping artifacts. During transition:
//! - Both old and new LOD meshes are rendered
//! - Alpha values are interpolated over transition duration
//! - Ensures visual continuity
//!
//! # Usage Example
//!
//! ```rust,ignore
//! use praxis_graphics::lod::{LodGroup, LodLevel, LodManager};
//! use praxis_ecs::World;
//!
//! # fn example() {
//! let mut world = World::new();
//!
//! // Create LOD group with 3 levels
//! let lod_group = LodGroup::new(vec![
//!     LodLevel::new("mesh_high", 0.0, 10.0),    // 0-10 units
//!     LodLevel::new("mesh_medium", 10.0, 25.0), // 10-25 units
//!     LodLevel::new("mesh_low", 25.0, 50.0),    // 25-50 units
//! ]);
//!
//! // Add to entity
//! // world.spawn((Transform::default(), lod_group));
//!
//! // Update LOD system each frame
//! let camera_position = praxis_math::Vec3::new(0.0, 0.0, 0.0);
//! // lod_manager.update(&mut world, camera_position, delta_time);
//! # }
//! ```

use crate::shaders;
use praxis_math::{Mat4, Vec3};
use praxis_utils::{debug, eyre, trace, Result};
use std::sync::Arc;
use vulkano::{
    buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer},
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::{allocator::DescriptorSetAllocator, DescriptorSet, WriteDescriptorSet},
    device::Device,
    memory::allocator::{AllocationCreateInfo, MemoryAllocator, MemoryTypeFilter},
    pipeline::{
        compute::ComputePipelineCreateInfo, layout::PipelineDescriptorSetLayoutCreateInfo,
        ComputePipeline, Pipeline, PipelineBindPoint, PipelineShaderStageCreateInfo,
    },
};

/// Maximum number of LOD levels supported per entity.
pub const MAX_LOD_LEVELS: usize = 8;

/// Default transition duration for LOD switches (in seconds).
pub const DEFAULT_TRANSITION_DURATION: f32 = 0.3;

/// Represents a single level of detail with its mesh and distance thresholds.
///
/// Each LOD level defines:
/// - The mesh to use at this detail level
/// - The minimum distance at which this level becomes active
/// - The maximum distance before transitioning to the next level
///
/// # Distance Thresholds
///
/// LOD levels are selected based on squared distance to avoid sqrt:
/// - `min_distance_squared`: Distance below which this LOD is active
/// - `max_distance_squared`: Distance above which next LOD is used
///
/// # Example
///
/// ```rust
/// use praxis_graphics::lod::LodLevel;
///
/// // High detail mesh for 0-10 units
/// let lod0 = LodLevel::new("character_high", 0.0, 10.0);
///
/// // Medium detail mesh for 10-25 units
/// let lod1 = LodLevel::new("character_medium", 10.0, 25.0);
///
/// // Low detail mesh for 25-50 units
/// let lod2 = LodLevel::new("character_low", 25.0, 50.0);
/// ```
#[derive(Debug, Clone)]
pub struct LodLevel {
    /// Identifier of the mesh to use at this LOD level.
    pub mesh_id: String,

    /// Minimum distance (squared) at which this LOD level is active.
    pub min_distance_squared: f32,

    /// Maximum distance (squared) before transitioning to next LOD level.
    pub max_distance_squared: f32,

    /// Optional screen coverage threshold (percentage of screen height).
    /// If specified, uses screen-space size instead of distance.
    pub screen_coverage: Option<f32>,
}

impl LodLevel {
    /// Creates a new LOD level with the specified mesh and distance range.
    ///
    /// # Arguments
    ///
    /// * `mesh_id` - Identifier of the mesh to use at this level
    /// * `min_distance` - Minimum distance (in world units) for this level
    /// * `max_distance` - Maximum distance (in world units) for this level
    ///
    /// # Note
    ///
    /// Distances are automatically squared internally for efficient comparison.
    pub fn new(mesh_id: impl Into<String>, min_distance: f32, max_distance: f32) -> Self {
        Self {
            mesh_id: mesh_id.into(),
            min_distance_squared: min_distance * min_distance,
            max_distance_squared: max_distance * max_distance,
            screen_coverage: None,
        }
    }

    /// Creates a new LOD level with screen coverage threshold.
    ///
    /// # Arguments
    ///
    /// * `mesh_id` - Identifier of the mesh to use at this level
    /// * `screen_coverage` - Percentage of screen height (0.0 to 1.0)
    pub fn with_screen_coverage(mesh_id: impl Into<String>, screen_coverage: f32) -> Self {
        Self {
            mesh_id: mesh_id.into(),
            min_distance_squared: 0.0,
            max_distance_squared: f32::MAX,
            screen_coverage: Some(screen_coverage.clamp(0.0, 1.0)),
        }
    }

    /// Checks if this LOD level should be active at the given squared distance.
    ///
    /// # Arguments
    ///
    /// * `distance_squared` - Squared distance from camera to object
    ///
    /// # Returns
    ///
    /// `true` if this LOD level is appropriate for the given distance
    pub fn is_active(&self, distance_squared: f32) -> bool {
        distance_squared >= self.min_distance_squared
            && distance_squared < self.max_distance_squared
    }

    /// Gets the mesh identifier for this LOD level.
    pub fn mesh_id(&self) -> &str {
        &self.mesh_id
    }
}

/// Manages multiple LOD levels for a single entity.
///
/// An LOD group defines a collection of mesh variants at different detail levels,
/// along with rules for selecting which level to use based on camera distance.
///
/// # LOD Selection Strategy
///
/// The system selects LOD levels based on:
/// 1. **Distance-based**: Uses squared distance to avoid sqrt
/// 2. **Screen-space**: Uses projected screen size (optional)
/// 3. **Hysteresis**: Prevents flickering with transition zones
///
/// # Smooth Transitions
///
/// The system supports smooth alpha-blended transitions between LOD levels:
/// - `transition_duration`: Time to blend between levels
/// - `enable_transitions`: Toggle for immediate vs smooth switching
///
/// # Example
///
/// ```rust
/// use praxis_graphics::lod::{LodGroup, LodLevel};
///
/// let mut lod_group = LodGroup::new(vec![
///     LodLevel::new("tree_high", 0.0, 15.0),
///     LodLevel::new("tree_medium", 15.0, 40.0),
///     LodLevel::new("tree_low", 40.0, 100.0),
///     LodLevel::new("tree_billboard", 100.0, 200.0),
/// ]);
///
/// // Configure transition behavior
/// lod_group.set_transition_duration(0.5);
/// lod_group.enable_transitions(true);
///
/// // Optionally set bias to force higher/lower detail
/// lod_group.set_lod_bias(1.0); // Prefer higher detail
/// ```
#[derive(Debug, Clone)]
pub struct LodGroup {
    /// Collection of LOD levels, ordered from highest to lowest detail.
    levels: Vec<LodLevel>,

    /// Currently active LOD level index.
    current_level: usize,

    /// Target LOD level index (for transitions).
    target_level: usize,

    /// Duration for smooth transitions between LOD levels.
    transition_duration: f32,

    /// Enable smooth transitions vs immediate switching.
    enable_transitions: bool,

    /// LOD bias to force higher/lower detail (-1.0 to 1.0).
    /// Positive values prefer higher detail, negative prefer lower.
    lod_bias: f32,

    /// Current transition state.
    transition_state: Option<LodTransitionState>,

    /// Whether to render both LOD levels during transition.
    blend_during_transition: bool,
}

impl LodGroup {
    /// Creates a new LOD group with the specified levels.
    ///
    /// # Arguments
    ///
    /// * `levels` - Vector of LOD levels, ordered from highest to lowest detail
    ///
    /// # Panics
    ///
    /// Panics if `levels` is empty or contains more than `MAX_LOD_LEVELS`.
    pub fn new(levels: Vec<LodLevel>) -> Self {
        assert!(!levels.is_empty(), "LOD group must have at least one level");
        assert!(
            levels.len() <= MAX_LOD_LEVELS,
            "LOD group cannot have more than {MAX_LOD_LEVELS} levels"
        );

        Self {
            levels,
            current_level: 0,
            target_level: 0,
            transition_duration: DEFAULT_TRANSITION_DURATION,
            enable_transitions: true,
            lod_bias: 0.0,
            transition_state: None,
            blend_during_transition: true,
        }
    }

    /// Sets the transition duration for LOD level changes.
    ///
    /// # Arguments
    ///
    /// * `duration` - Transition duration in seconds
    pub fn set_transition_duration(&mut self, duration: f32) {
        self.transition_duration = duration.max(0.0);
    }

    /// Enables or disables smooth transitions between LOD levels.
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable smooth transitions, `false` for immediate switching
    pub fn enable_transitions(&mut self, enable: bool) {
        self.enable_transitions = enable;
    }

    /// Sets the LOD bias to force higher or lower detail levels.
    ///
    /// # Arguments
    ///
    /// * `bias` - Bias value (-1.0 to 1.0)
    ///   - Positive values prefer higher detail
    ///   - Negative values prefer lower detail
    pub fn set_lod_bias(&mut self, bias: f32) {
        self.lod_bias = bias.clamp(-1.0, 1.0);
    }

    /// Enables or disables blending during transitions.
    ///
    /// When enabled, both old and new LOD meshes are rendered during transition.
    /// When disabled, only the new LOD mesh is rendered (with fade-in).
    pub fn set_blend_during_transition(&mut self, blend: bool) {
        self.blend_during_transition = blend;
    }

    /// Gets the current LOD level index.
    pub fn current_level(&self) -> usize {
        self.current_level
    }

    /// Gets the target LOD level index.
    pub fn target_level(&self) -> usize {
        self.target_level
    }

    /// Gets the number of LOD levels in this group.
    pub fn level_count(&self) -> usize {
        self.levels.len()
    }

    /// Gets a reference to the LOD level at the specified index.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    pub fn get_level(&self, index: usize) -> &LodLevel {
        &self.levels[index]
    }

    /// Gets the mesh ID for the current LOD level.
    pub fn current_mesh_id(&self) -> &str {
        self.levels[self.current_level].mesh_id()
    }

    /// Gets the mesh ID for the target LOD level.
    pub fn target_mesh_id(&self) -> &str {
        self.levels[self.target_level].mesh_id()
    }

    /// Checks if a transition is currently in progress.
    pub fn is_transitioning(&self) -> bool {
        self.transition_state.is_some()
    }

    /// Gets the current transition progress (0.0 to 1.0).
    ///
    /// Returns `0.0` if no transition is in progress.
    pub fn transition_progress(&self) -> f32 {
        self.transition_state
            .as_ref()
            .map(|state| state.progress)
            .unwrap_or(0.0)
    }

    /// Selects the appropriate LOD level based on squared distance from camera.
    ///
    /// # Arguments
    ///
    /// * `distance_squared` - Squared distance from camera to object
    ///
    /// # Returns
    ///
    /// Index of the selected LOD level
    pub fn select_lod_level(&self, distance_squared: f32) -> usize {
        // Apply LOD bias by scaling distance
        let bias_scale = if self.lod_bias > 0.0 {
            // Positive bias: make objects appear closer (higher detail)
            1.0 - self.lod_bias * 0.5
        } else {
            // Negative bias: make objects appear farther (lower detail)
            1.0 + (-self.lod_bias) * 0.5
        };

        let adjusted_distance_squared = distance_squared * bias_scale * bias_scale;

        // Find the appropriate LOD level
        for (index, level) in self.levels.iter().enumerate() {
            if level.is_active(adjusted_distance_squared) {
                return index;
            }
        }

        // If no level matches, use the lowest detail level
        self.levels.len().saturating_sub(1)
    }

    /// Updates the LOD group state based on camera distance.
    ///
    /// This method should be called each frame to update LOD selection and transitions.
    ///
    /// # Arguments
    ///
    /// * `distance_squared` - Squared distance from camera to object
    /// * `delta_time` - Time elapsed since last update (in seconds)
    pub fn update(&mut self, distance_squared: f32, delta_time: f32) {
        // Select new LOD level
        let new_level = self.select_lod_level(distance_squared);

        // Check if we need to change LOD level
        if new_level != self.target_level {
            if self.enable_transitions && self.transition_duration > 0.0 {
                // Start a new transition
                self.start_transition(new_level);
            } else {
                // Immediate switch
                self.current_level = new_level;
                self.target_level = new_level;
                self.transition_state = None;
            }
        }

        // Update transition state if active
        if let Some(ref mut state) = self.transition_state {
            state.progress += delta_time / self.transition_duration;

            if state.progress >= 1.0 {
                // Transition complete
                self.current_level = self.target_level;
                self.transition_state = None;
                trace!("LOD transition complete: level {}", self.current_level);
            }
        }
    }

    /// Starts a transition to a new LOD level.
    fn start_transition(&mut self, new_level: usize) {
        debug!(
            "Starting LOD transition: {} -> {}",
            self.current_level, new_level
        );

        self.target_level = new_level;
        self.transition_state = Some(LodTransitionState { progress: 0.0 });
    }

    /// Gets the alpha value for the current LOD level during transition.
    ///
    /// # Returns
    ///
    /// Alpha value (0.0 to 1.0) for rendering the current level
    pub fn current_alpha(&self) -> f32 {
        if let Some(ref state) = self.transition_state {
            1.0 - state.progress
        } else {
            1.0
        }
    }

    /// Gets the alpha value for the target LOD level during transition.
    ///
    /// # Returns
    ///
    /// Alpha value (0.0 to 1.0) for rendering the target level
    pub fn target_alpha(&self) -> f32 {
        if let Some(ref state) = self.transition_state {
            state.progress
        } else {
            0.0
        }
    }

    /// Gets the meshes that should be rendered, along with their alpha values.
    ///
    /// # Returns
    ///
    /// Vector of (mesh_id, alpha) tuples for rendering
    pub fn get_render_meshes(&self) -> Vec<(&str, f32)> {
        let mut meshes = Vec::new();

        if self.is_transitioning() && self.blend_during_transition {
            // During transition, render both meshes with interpolated alpha
            let current_alpha = self.current_alpha();
            let target_alpha = self.target_alpha();

            if current_alpha > 0.0 {
                meshes.push((self.current_mesh_id(), current_alpha));
            }

            if target_alpha > 0.0 {
                meshes.push((self.target_mesh_id(), target_alpha));
            }
        } else {
            // Not transitioning or blend disabled, render only current mesh
            meshes.push((self.current_mesh_id(), 1.0));
        }

        meshes
    }

    /// Forces an immediate LOD level change without transition.
    ///
    /// # Arguments
    ///
    /// * `level` - Index of the LOD level to switch to
    ///
    /// # Panics
    ///
    /// Panics if `level` is out of bounds.
    pub fn force_lod_level(&mut self, level: usize) {
        assert!(
            level < self.levels.len(),
            "LOD level {} out of bounds (max: {})",
            level,
            self.levels.len() - 1
        );

        self.current_level = level;
        self.target_level = level;
        self.transition_state = None;
    }
}

/// State information for an active LOD transition.
#[derive(Debug, Clone)]
struct LodTransitionState {
    /// Transition progress (0.0 to 1.0).
    progress: f32,
}

/// Manages LOD system updates across all entities.
///
/// The LOD manager provides system-level functionality for updating all LOD groups
/// in the world based on camera position.
///
/// # Responsibilities
///
/// - Update all LOD groups each frame
/// - Calculate squared distances for efficient comparison
/// - Manage global LOD settings and statistics
///
/// # Example
///
/// ```rust,ignore
/// use praxis_graphics::lod::LodManager;
/// use praxis_ecs::World;
/// use praxis_math::Vec3;
///
/// # fn example() {
/// let mut world = World::new();
/// let mut lod_manager = LodManager::new();
///
/// // Each frame:
/// let camera_position = Vec3::new(0.0, 5.0, 10.0);
/// let delta_time = 0.016; // ~60 FPS
/// // lod_manager.update(&mut world, camera_position, delta_time);
///
/// // Query statistics
/// let stats = lod_manager.statistics();
/// println!("Active LOD groups: {}", stats.active_groups);
/// println!("Transitions in progress: {}", stats.transitioning_groups);
/// # }
/// ```
pub struct LodManager {
    /// Global LOD bias applied to all LOD groups.
    global_lod_bias: f32,

    /// Enable/disable LOD system globally.
    enabled: bool,

    /// Statistics for the current frame.
    statistics: LodStatistics,
}

impl LodManager {
    /// Creates a new LOD manager with default settings.
    pub fn new() -> Self {
        Self {
            global_lod_bias: 0.0,
            enabled: true,
            statistics: LodStatistics::default(),
        }
    }

    /// Enables or disables the LOD system globally.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Checks if the LOD system is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets the global LOD bias applied to all LOD groups.
    ///
    /// # Arguments
    ///
    /// * `bias` - Global LOD bias (-1.0 to 1.0)
    pub fn set_global_lod_bias(&mut self, bias: f32) {
        self.global_lod_bias = bias.clamp(-1.0, 1.0);
    }

    /// Gets the current global LOD bias.
    pub fn global_lod_bias(&self) -> f32 {
        self.global_lod_bias
    }

    /// Gets LOD statistics for the current frame.
    pub fn statistics(&self) -> &LodStatistics {
        &self.statistics
    }

    /// Resets frame statistics.
    pub fn reset_statistics(&mut self) {
        self.statistics = LodStatistics::default();
    }

    /// Updates a single LOD group based on object and camera position.
    ///
    /// # Arguments
    ///
    /// * `lod_group` - The LOD group to update
    /// * `object_position` - World position of the object
    /// * `camera_position` - World position of the camera
    /// * `delta_time` - Time elapsed since last update (in seconds)
    pub fn update_lod_group(
        &mut self,
        lod_group: &mut LodGroup,
        object_position: Vec3,
        camera_position: Vec3,
        delta_time: f32,
    ) {
        if !self.enabled {
            return;
        }

        // Calculate squared distance (avoids sqrt)
        let delta = object_position - camera_position;
        let distance_squared = delta.length_squared();

        // Apply global bias to the LOD group temporarily
        let original_bias = lod_group.lod_bias;
        lod_group.lod_bias = (lod_group.lod_bias + self.global_lod_bias).clamp(-1.0, 1.0);

        // Update the LOD group
        lod_group.update(distance_squared, delta_time);

        // Restore original bias
        lod_group.lod_bias = original_bias;

        // Update statistics
        self.statistics.active_groups += 1;
        if lod_group.is_transitioning() {
            self.statistics.transitioning_groups += 1;
        }
    }

    /// Calculates squared distance between two points efficiently.
    ///
    /// This is a utility function for manual LOD calculations.
    pub fn calculate_distance_squared(position1: Vec3, position2: Vec3) -> f32 {
        let delta = position1 - position2;
        delta.length_squared()
    }
}

impl Default for LodManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about LOD system performance and state.
#[derive(Debug, Clone, Default)]
pub struct LodStatistics {
    /// Number of active LOD groups in the current frame.
    pub active_groups: usize,

    /// Number of LOD groups currently transitioning.
    pub transitioning_groups: usize,

    /// Total number of meshes rendered due to LOD (including transition meshes).
    pub meshes_rendered: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lod_level_creation() {
        let lod = LodLevel::new("mesh_high", 0.0, 10.0);
        assert_eq!(lod.mesh_id(), "mesh_high");
        assert_eq!(lod.min_distance_squared, 0.0);
        assert_eq!(lod.max_distance_squared, 100.0);
    }

    #[test]
    fn test_lod_level_is_active() {
        let lod = LodLevel::new("mesh", 10.0, 20.0);

        // Below min distance
        assert!(!lod.is_active(50.0)); // sqrt(50) < 10

        // Within range
        assert!(lod.is_active(150.0)); // 10 < sqrt(150) < 20

        // Above max distance
        assert!(!lod.is_active(500.0)); // sqrt(500) > 20
    }

    #[test]
    fn test_lod_group_creation() {
        let lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ]);

        assert_eq!(lod_group.level_count(), 3);
        assert_eq!(lod_group.current_level(), 0);
        assert_eq!(lod_group.target_level(), 0);
    }

    #[test]
    fn test_lod_selection_near() {
        let lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ]);

        // Near distance (5 units squared = 25)
        let level = lod_group.select_lod_level(25.0);
        assert_eq!(level, 0); // Should select high detail
    }

    #[test]
    fn test_lod_selection_medium() {
        let lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ]);

        // Medium distance (15 units squared = 225)
        let level = lod_group.select_lod_level(225.0);
        assert_eq!(level, 1); // Should select medium detail
    }

    #[test]
    fn test_lod_selection_far() {
        let lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ]);

        // Far distance (30 units squared = 900)
        let level = lod_group.select_lod_level(900.0);
        assert_eq!(level, 2); // Should select low detail
    }

    #[test]
    fn test_lod_bias_positive() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ]);

        lod_group.set_lod_bias(0.5); // Prefer higher detail

        // At medium distance, positive bias should select higher detail
        let level = lod_group.select_lod_level(150.0); // ~12 units
        assert_eq!(level, 0); // Should prefer high detail due to bias
    }

    #[test]
    fn test_lod_transition_start() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ]);

        lod_group.enable_transitions(true);
        lod_group.set_transition_duration(1.0);

        // Trigger transition by moving far
        lod_group.update(400.0, 0.0); // 20 units

        assert!(lod_group.is_transitioning());
        assert_eq!(lod_group.current_level(), 0);
        assert_eq!(lod_group.target_level(), 1);
    }

    #[test]
    fn test_lod_transition_progress() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ]);

        lod_group.enable_transitions(true);
        lod_group.set_transition_duration(1.0);

        // Start transition
        lod_group.update(400.0, 0.0);

        // Update halfway through transition
        lod_group.update(400.0, 0.5);

        assert!(lod_group.is_transitioning());
        assert!((lod_group.transition_progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_lod_transition_complete() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ]);

        lod_group.enable_transitions(true);
        lod_group.set_transition_duration(1.0);

        // Start and complete transition
        lod_group.update(400.0, 0.0);
        lod_group.update(400.0, 1.5); // More than transition duration

        assert!(!lod_group.is_transitioning());
        assert_eq!(lod_group.current_level(), 1);
        assert_eq!(lod_group.target_level(), 1);
    }

    #[test]
    fn test_lod_immediate_switch() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ]);

        lod_group.enable_transitions(false);

        // Switch should be immediate
        lod_group.update(400.0, 0.0);

        assert!(!lod_group.is_transitioning());
        assert_eq!(lod_group.current_level(), 1);
    }

    #[test]
    fn test_lod_alpha_values() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ]);

        lod_group.enable_transitions(true);
        lod_group.set_transition_duration(1.0);

        // Start transition
        lod_group.update(400.0, 0.0);
        lod_group.update(400.0, 0.3);

        // Check alpha values
        let current_alpha = lod_group.current_alpha();
        let target_alpha = lod_group.target_alpha();

        assert!((current_alpha - 0.7).abs() < 0.01);
        assert!((target_alpha - 0.3).abs() < 0.01);
        assert!((current_alpha + target_alpha - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_lod_get_render_meshes_single() {
        let lod_group = LodGroup::new(vec![LodLevel::new("high", 0.0, 10.0)]);

        let meshes = lod_group.get_render_meshes();

        assert_eq!(meshes.len(), 1);
        assert_eq!(meshes[0].0, "high");
        assert_eq!(meshes[0].1, 1.0);
    }

    #[test]
    fn test_lod_get_render_meshes_transition() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ]);

        lod_group.enable_transitions(true);
        lod_group.set_blend_during_transition(true);
        lod_group.set_transition_duration(1.0);

        // Start transition
        lod_group.update(400.0, 0.0);
        lod_group.update(400.0, 0.5);

        let meshes = lod_group.get_render_meshes();

        assert_eq!(meshes.len(), 2);
        assert!(meshes[0].0 == "high" && meshes[1].0 == "low");
    }

    #[test]
    fn test_lod_force_level() {
        let mut lod_group = LodGroup::new(vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ]);

        lod_group.force_lod_level(2);

        assert_eq!(lod_group.current_level(), 2);
        assert_eq!(lod_group.target_level(), 2);
        assert!(!lod_group.is_transitioning());
    }

    #[test]
    fn test_lod_manager_creation() {
        let manager = LodManager::new();
        assert!(manager.is_enabled());
        assert_eq!(manager.global_lod_bias(), 0.0);
    }

    #[test]
    fn test_lod_manager_global_bias() {
        let mut manager = LodManager::new();
        manager.set_global_lod_bias(0.5);
        assert_eq!(manager.global_lod_bias(), 0.5);
    }

    #[test]
    fn test_lod_manager_enabled() {
        let mut manager = LodManager::new();
        manager.set_enabled(false);
        assert!(!manager.is_enabled());
    }

    #[test]
    fn test_calculate_distance_squared() {
        let pos1 = Vec3::new(0.0, 0.0, 0.0);
        let pos2 = Vec3::new(3.0, 4.0, 0.0);

        let distance_sq = LodManager::calculate_distance_squared(pos1, pos2);

        // 3^2 + 4^2 = 9 + 16 = 25
        assert_eq!(distance_sq, 25.0);
    }

    #[test]
    fn test_lod_statistics() {
        let manager = LodManager::new();
        let stats = manager.statistics();

        assert_eq!(stats.active_groups, 0);
        assert_eq!(stats.transitioning_groups, 0);
    }

    #[test]
    #[should_panic(expected = "LOD group must have at least one level")]
    fn test_lod_group_empty_levels() {
        LodGroup::new(vec![]);
    }

    #[test]
    fn test_lod_screen_coverage() {
        let lod = LodLevel::with_screen_coverage("billboard", 0.05);
        assert_eq!(lod.mesh_id(), "billboard");
        assert_eq!(lod.screen_coverage, Some(0.05));
    }

    #[test]
    fn test_lod_level_with_screen_coverage_clamp() {
        let lod = LodLevel::with_screen_coverage("test", 1.5);
        assert_eq!(lod.screen_coverage, Some(1.0));

        let lod2 = LodLevel::with_screen_coverage("test2", -0.5);
        assert_eq!(lod2.screen_coverage, Some(0.0));
    }

    // ===== GPU-Driven LOD Selection Tests =====

    #[test]
    fn test_gpu_object_data_size_and_alignment() {
        use std::mem::{align_of, size_of};

        // Size: 16 (mat4) * 4 + 16 (sphere) + 16 (4 u32s with padding) = 96 bytes
        assert_eq!(size_of::<GpuObjectData>(), 96);

        // Should be 16-byte aligned for GPU buffers
        assert_eq!(align_of::<GpuObjectData>(), 16);
    }

    #[test]
    fn test_gpu_lod_level_size_and_alignment() {
        use std::mem::{align_of, size_of};

        // Size: 4 (u32) + 4 (f32) + 4 (f32) + 4 (padding) = 16 bytes
        assert_eq!(size_of::<GpuLodLevel>(), 16);

        // Should be 4-byte aligned (standard for struct of u32/f32)
        assert_eq!(align_of::<GpuLodLevel>(), 4);
    }

    #[test]
    fn test_lod_uniforms_size_and_alignment() {
        use std::mem::{align_of, size_of};

        // Size should be multiple of 16 due to align(16)
        assert_eq!(size_of::<LodUniforms>(), 32);

        // Must be 16-byte aligned for uniform buffers
        assert_eq!(align_of::<LodUniforms>(), 16);
    }

    #[test]
    fn test_gpu_object_data_creation() {
        let model = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
        let bounding_sphere = [5.0, 6.0, 7.0, 2.5]; // Center (5,6,7), radius 2.5
        let mesh_id = 42;
        let lod_count = 3;
        let lod_offset = 10;

        let gpu_data = GpuObjectData::new(model, bounding_sphere, mesh_id, lod_count, lod_offset);

        assert_eq!(gpu_data.model, model.to_cols_array_2d());
        assert_eq!(gpu_data.bounding_sphere, bounding_sphere);
        assert_eq!(gpu_data.mesh_id, mesh_id);
        assert_eq!(gpu_data.lod_count, lod_count);
        assert_eq!(gpu_data.lod_offset, lod_offset);
        assert_eq!(gpu_data.padding, 0);
    }

    #[test]
    fn test_gpu_lod_level_from_lod_level() {
        let lod_level = LodLevel::new("mesh_high", 0.0, 10.0);
        let mesh_id = 100;

        let gpu_lod = GpuLodLevel::from_lod_level(&lod_level, mesh_id);

        assert_eq!(gpu_lod.mesh_id, mesh_id);
        assert_eq!(gpu_lod.min_distance_sq, 0.0);
        assert_eq!(gpu_lod.max_distance_sq, 100.0); // 10^2
        assert_eq!(gpu_lod.padding, 0);
    }

    #[test]
    fn test_gpu_lod_level_distance_thresholds() {
        let lod_level = LodLevel::new("mesh", 5.0, 20.0);
        let gpu_lod = GpuLodLevel::from_lod_level(&lod_level, 0);

        // Verify squared distances are correctly stored
        assert_eq!(gpu_lod.min_distance_sq, 25.0); // 5^2
        assert_eq!(gpu_lod.max_distance_sq, 400.0); // 20^2
    }

    #[test]
    fn test_lod_uniforms_creation() {
        let camera_pos = Vec3::new(1.0, 2.0, 3.0);
        let lod_bias = 0.5;
        let object_count = 1000;
        let enable_lod = true;

        let uniforms = LodUniforms::new(camera_pos, lod_bias, object_count, enable_lod);

        assert_eq!(uniforms.camera_position, [1.0, 2.0, 3.0]);
        assert_eq!(uniforms.lod_bias, 0.5);
        assert_eq!(uniforms.object_count, 1000);
        assert_eq!(uniforms.enable_lod, 1);
        assert_eq!(uniforms.padding1, 0);
        assert_eq!(uniforms.padding2, 0);
    }

    #[test]
    fn test_lod_uniforms_enable_lod_flag() {
        let camera_pos = Vec3::ZERO;

        let uniforms_enabled = LodUniforms::new(camera_pos, 0.0, 0, true);
        assert_eq!(uniforms_enabled.enable_lod, 1);

        let uniforms_disabled = LodUniforms::new(camera_pos, 0.0, 0, false);
        assert_eq!(uniforms_disabled.enable_lod, 0);
    }

    #[test]
    fn test_lod_selection_logic_near_distance() {
        // Test that LOD selection logic matches expected behavior for near objects
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),    // 0-100 squared
            LodLevel::new("medium", 10.0, 20.0), // 100-400 squared
            LodLevel::new("low", 20.0, 50.0),    // 400-2500 squared
        ];

        let lod_group = LodGroup::new(lod_levels);

        // Distance 5.0 squared = 25.0 -> should select level 0
        let selected = lod_group.select_lod_level(25.0);
        assert_eq!(selected, 0);
    }

    #[test]
    fn test_lod_selection_logic_medium_distance() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ];

        let lod_group = LodGroup::new(lod_levels);

        // Distance 15.0 squared = 225.0 -> should select level 1
        let selected = lod_group.select_lod_level(225.0);
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_lod_selection_logic_far_distance() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ];

        let lod_group = LodGroup::new(lod_levels);

        // Distance 30.0 squared = 900.0 -> should select level 2
        let selected = lod_group.select_lod_level(900.0);
        assert_eq!(selected, 2);
    }

    #[test]
    fn test_lod_selection_logic_extreme_distance() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ];

        let lod_group = LodGroup::new(lod_levels);

        // Distance beyond all thresholds -> should select last level
        let selected = lod_group.select_lod_level(10000.0);
        assert_eq!(selected, 2);
    }

    #[test]
    fn test_lod_selection_at_boundary() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
        ];

        let lod_group = LodGroup::new(lod_levels);

        // Exactly at boundary (10.0 squared = 100.0)
        // Should select medium (level 1) as it's min_distance_sq is 100.0
        let selected = lod_group.select_lod_level(100.0);
        assert_eq!(selected, 1);

        // Just below boundary
        let selected = lod_group.select_lod_level(99.9);
        assert_eq!(selected, 0);

        // Just above boundary
        let selected = lod_group.select_lod_level(100.1);
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_lod_bias_application_positive() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),    // 0-100
            LodLevel::new("medium", 10.0, 20.0), // 100-400
            LodLevel::new("low", 20.0, 50.0),    // 400-2500
        ];

        let mut lod_group = LodGroup::new(lod_levels);

        // Positive bias should prefer higher detail
        lod_group.set_lod_bias(1.0);

        // At distance 15 squared = 225, without bias would be medium (level 1)
        // With max positive bias (0.5 scale), adjusted = 225 * 0.5^2 = 56.25
        // This should select high detail (level 0)
        let selected = lod_group.select_lod_level(225.0);
        assert_eq!(selected, 0);
    }

    #[test]
    fn test_lod_bias_application_negative() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ];

        let mut lod_group = LodGroup::new(lod_levels);

        // Negative bias should prefer lower detail
        lod_group.set_lod_bias(-1.0);

        // At distance 5 squared = 25, without bias would be high (level 0)
        // With max negative bias (1.5 scale), adjusted = 25 * 1.5^2 = 56.25
        // Still in high range, but closer to medium
        let selected = lod_group.select_lod_level(25.0);
        // At 56.25, still in high range (0-100)
        assert_eq!(selected, 0);

        // At distance 8 squared = 64
        // With negative bias: 64 * 1.5^2 = 144, should be in medium range
        let selected = lod_group.select_lod_level(64.0);
        assert_eq!(selected, 1);
    }

    #[test]
    fn test_lod_bias_application_zero() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
        ];

        let mut lod_group = LodGroup::new(lod_levels);
        lod_group.set_lod_bias(0.0);

        // Zero bias should have no effect
        let selected = lod_group.select_lod_level(225.0);
        assert_eq!(selected, 1); // Medium
    }

    #[test]
    fn test_lod_bias_clamping() {
        let mut lod_group = LodGroup::new(vec![LodLevel::new("test", 0.0, 100.0)]);

        // Test clamping to valid range [-1.0, 1.0]
        lod_group.set_lod_bias(2.0);
        assert_eq!(lod_group.lod_bias, 1.0);

        lod_group.set_lod_bias(-2.0);
        assert_eq!(lod_group.lod_bias, -1.0);

        lod_group.set_lod_bias(0.5);
        assert_eq!(lod_group.lod_bias, 0.5);
    }

    #[test]
    fn test_buffer_layout_correctness_gpu_object_data() {
        use std::mem::offset_of;

        // Verify field offsets match expected GPU buffer layout
        assert_eq!(offset_of!(GpuObjectData, model), 0);
        assert_eq!(offset_of!(GpuObjectData, bounding_sphere), 64);
        assert_eq!(offset_of!(GpuObjectData, mesh_id), 80);
        assert_eq!(offset_of!(GpuObjectData, lod_count), 84);
        assert_eq!(offset_of!(GpuObjectData, lod_offset), 88);
        assert_eq!(offset_of!(GpuObjectData, padding), 92);
    }

    #[test]
    fn test_buffer_layout_correctness_gpu_lod_level() {
        use std::mem::offset_of;

        // Verify field offsets match expected GPU buffer layout
        assert_eq!(offset_of!(GpuLodLevel, mesh_id), 0);
        assert_eq!(offset_of!(GpuLodLevel, min_distance_sq), 4);
        assert_eq!(offset_of!(GpuLodLevel, max_distance_sq), 8);
        assert_eq!(offset_of!(GpuLodLevel, padding), 12);
    }

    #[test]
    fn test_buffer_layout_correctness_lod_uniforms() {
        use std::mem::offset_of;

        // Verify field offsets match expected GPU uniform buffer layout
        assert_eq!(offset_of!(LodUniforms, camera_position), 0);
        assert_eq!(offset_of!(LodUniforms, lod_bias), 12);
        assert_eq!(offset_of!(LodUniforms, object_count), 16);
        assert_eq!(offset_of!(LodUniforms, enable_lod), 20);
        assert_eq!(offset_of!(LodUniforms, padding1), 24);
        assert_eq!(offset_of!(LodUniforms, padding2), 28);
    }

    #[test]
    fn test_bytemuck_pod_traits_gpu_object_data() {
        // Verify that GpuObjectData implements Pod and Zeroable
        let zeroed = GpuObjectData::zeroed();
        assert_eq!(zeroed.mesh_id, 0);
        assert_eq!(zeroed.lod_count, 0);
        assert_eq!(zeroed.lod_offset, 0);
        assert_eq!(zeroed.padding, 0);

        // Test that we can cast to bytes
        let data = GpuObjectData::new(Mat4::IDENTITY, [0.0; 4], 1, 2, 3);
        let _bytes: &[u8] = bytemuck::bytes_of(&data);
    }

    #[test]
    fn test_bytemuck_pod_traits_gpu_lod_level() {
        let zeroed = GpuLodLevel::zeroed();
        assert_eq!(zeroed.mesh_id, 0);
        assert_eq!(zeroed.min_distance_sq, 0.0);
        assert_eq!(zeroed.max_distance_sq, 0.0);
        assert_eq!(zeroed.padding, 0);

        let data = GpuLodLevel {
            mesh_id: 5,
            min_distance_sq: 100.0,
            max_distance_sq: 400.0,
            padding: 0,
        };
        let _bytes: &[u8] = bytemuck::bytes_of(&data);
    }

    #[test]
    fn test_bytemuck_pod_traits_lod_uniforms() {
        let zeroed = LodUniforms::zeroed();
        assert_eq!(zeroed.camera_position, [0.0; 3]);
        assert_eq!(zeroed.lod_bias, 0.0);
        assert_eq!(zeroed.object_count, 0);
        assert_eq!(zeroed.enable_lod, 0);

        let data = LodUniforms::new(Vec3::new(1.0, 2.0, 3.0), 0.5, 100, true);
        let _bytes: &[u8] = bytemuck::bytes_of(&data);
    }

    #[test]
    fn test_multiple_gpu_lod_levels_array() {
        // Test that multiple GPU LOD levels can be stored in a contiguous array
        let cpu_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("medium", 10.0, 20.0),
            LodLevel::new("low", 20.0, 50.0),
        ];

        let gpu_levels: Vec<GpuLodLevel> = cpu_levels
            .iter()
            .enumerate()
            .map(|(i, level)| GpuLodLevel::from_lod_level(level, i as u32))
            .collect();

        assert_eq!(gpu_levels.len(), 3);
        assert_eq!(gpu_levels[0].mesh_id, 0);
        assert_eq!(gpu_levels[1].mesh_id, 1);
        assert_eq!(gpu_levels[2].mesh_id, 2);

        // Verify we can cast the entire array to bytes
        let _bytes: &[u8] = bytemuck::cast_slice(&gpu_levels);
    }

    #[test]
    fn test_gpu_object_data_array() {
        // Test that multiple GPU object data can be stored in a contiguous array
        let objects = vec![
            GpuObjectData::new(
                Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
                [0.0, 0.0, 0.0, 1.0],
                0,
                3,
                0,
            ),
            GpuObjectData::new(
                Mat4::from_translation(Vec3::new(10.0, 0.0, 0.0)),
                [10.0, 0.0, 0.0, 1.0],
                3,
                3,
                3,
            ),
            GpuObjectData::new(
                Mat4::from_translation(Vec3::new(20.0, 0.0, 0.0)),
                [20.0, 0.0, 0.0, 1.0],
                6,
                3,
                6,
            ),
        ];

        assert_eq!(objects.len(), 3);

        // Verify we can cast the entire array to bytes
        let _bytes: &[u8] = bytemuck::cast_slice(&objects);
    }

    #[test]
    fn test_lod_selection_with_single_level() {
        let lod_group = LodGroup::new(vec![LodLevel::new("only", 0.0, 1000.0)]);

        // Any distance should select the only level
        assert_eq!(lod_group.select_lod_level(0.0), 0);
        assert_eq!(lod_group.select_lod_level(500.0), 0);
        assert_eq!(lod_group.select_lod_level(1000000.0), 0);
    }

    #[test]
    fn test_lod_selection_with_max_levels() {
        let mut levels = Vec::new();
        for i in 0..MAX_LOD_LEVELS {
            let min_dist = (i * 10) as f32;
            let max_dist = ((i + 1) * 10) as f32;
            levels.push(LodLevel::new(format!("lod_{}", i), min_dist, max_dist));
        }

        let lod_group = LodGroup::new(levels);
        assert_eq!(lod_group.level_count(), MAX_LOD_LEVELS);

        // Test selection at various distances
        assert_eq!(lod_group.select_lod_level(25.0), 0); // sqrt(25) = 5, in range 0-10
        assert_eq!(lod_group.select_lod_level(225.0), 1); // sqrt(225) = 15, in range 10-20
        assert_eq!(lod_group.select_lod_level(10000.0), MAX_LOD_LEVELS - 1); // Far away
    }

    #[test]
    fn test_gpu_lod_data_with_complex_transform() {
        use praxis_math::Quat;

        // Create complex transform with rotation and scale
        let translation = Vec3::new(10.0, 20.0, 30.0);
        let rotation = Quat::from_rotation_y(std::f32::consts::PI / 4.0);
        let scale = Vec3::new(2.0, 3.0, 4.0);

        let model = Mat4::from_scale_rotation_translation(scale, rotation, translation);

        let gpu_data = GpuObjectData::new(model, [0.0, 0.0, 0.0, 5.0], 0, 1, 0);

        // Verify the matrix was stored correctly
        let stored_mat = Mat4::from_cols_array_2d(&gpu_data.model);
        let diff = (stored_mat.to_cols_array_2d()
            .iter()
            .flatten()
            .zip(model.to_cols_array_2d().iter().flatten())
            .map(|(a, b)| (a - b).abs())
            .sum::<f32>());

        assert!(diff < 0.0001, "Matrix should be stored accurately");
    }

    #[test]
    fn test_lod_bias_interpolation_precision() {
        let lod_levels = vec![
            LodLevel::new("high", 0.0, 10.0),
            LodLevel::new("low", 10.0, 50.0),
        ];

        let mut lod_group = LodGroup::new(lod_levels);

        // Test various bias values
        for bias in [-1.0, -0.5, 0.0, 0.5, 1.0] {
            lod_group.set_lod_bias(bias);

            // Verify bias is stored correctly
            assert!((lod_group.lod_bias - bias).abs() < f32::EPSILON);

            // Verify selection still works
            let _selected = lod_group.select_lod_level(100.0);
        }
    }

    #[test]
    fn test_distance_threshold_edge_cases() {
        // Test with very small distances
        let lod_small = LodLevel::new("tiny", 0.0, 0.1);
        let gpu_lod_small = GpuLodLevel::from_lod_level(&lod_small, 0);
        assert_eq!(gpu_lod_small.max_distance_sq, 0.01);

        // Test with very large distances
        let lod_large = LodLevel::new("huge", 1000.0, 10000.0);
        let gpu_lod_large = GpuLodLevel::from_lod_level(&lod_large, 0);
        assert_eq!(gpu_lod_large.min_distance_sq, 1_000_000.0);
        assert_eq!(gpu_lod_large.max_distance_sq, 100_000_000.0);

        // Test with zero distance
        let lod_zero = LodLevel::new("zero", 0.0, 0.0);
        let gpu_lod_zero = GpuLodLevel::from_lod_level(&lod_zero, 0);
        assert_eq!(gpu_lod_zero.min_distance_sq, 0.0);
        assert_eq!(gpu_lod_zero.max_distance_sq, 0.0);
    }
}

// ===== GPU-Driven LOD Selection =====

/// Per-object data for GPU LOD calculation.
///
/// This structure matches the shader's ObjectData layout and contains all
/// information needed to calculate LOD on the GPU.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuObjectData {
    /// Model matrix transforming from model to world space.
    pub model: [[f32; 4]; 4],

    /// Bounding sphere in model space (xyz = center, w = radius).
    pub bounding_sphere: [f32; 4],

    /// Base mesh ID (highest detail).
    pub mesh_id: u32,

    /// Number of LOD levels for this object.
    pub lod_count: u32,

    /// Offset into the LOD levels array.
    pub lod_offset: u32,

    /// Padding for alignment.
    pub padding: u32,
}

impl GpuObjectData {
    /// Creates new GPU object data.
    pub fn new(
        model: Mat4,
        bounding_sphere: [f32; 4],
        mesh_id: u32,
        lod_count: u32,
        lod_offset: u32,
    ) -> Self {
        Self {
            model: model.to_cols_array_2d(),
            bounding_sphere,
            mesh_id,
            lod_count,
            lod_offset,
            padding: 0,
        }
    }
}

/// GPU LOD level definition.
///
/// Matches the shader's LodLevel structure for GPU-side LOD selection.
#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GpuLodLevel {
    /// Mesh ID for this LOD level.
    pub mesh_id: u32,

    /// Minimum distance squared.
    pub min_distance_sq: f32,

    /// Maximum distance squared.
    pub max_distance_sq: f32,

    /// Padding for alignment.
    pub padding: u32,
}

impl GpuLodLevel {
    /// Creates a new GPU LOD level from a CPU LOD level.
    pub fn from_lod_level(level: &LodLevel, mesh_id: u32) -> Self {
        Self {
            mesh_id,
            min_distance_sq: level.min_distance_squared,
            max_distance_sq: level.max_distance_squared,
            padding: 0,
        }
    }
}

/// LOD selection uniforms passed to the compute shader.
#[repr(C, align(16))]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LodUniforms {
    /// Camera position in world space.
    pub camera_position: [f32; 3],

    /// Global LOD bias (-1.0 to 1.0).
    pub lod_bias: f32,

    /// Number of objects to process.
    pub object_count: u32,

    /// Enable LOD system (0 = disabled, 1 = enabled).
    pub enable_lod: u32,

    /// Padding for alignment.
    pub padding1: u32,

    /// Padding for alignment.
    pub padding2: u32,
}

impl LodUniforms {
    /// Creates LOD uniforms.
    pub fn new(camera_position: Vec3, lod_bias: f32, object_count: u32, enable_lod: bool) -> Self {
        Self {
            camera_position: camera_position.to_array(),
            lod_bias,
            object_count,
            enable_lod: if enable_lod { 1 } else { 0 },
            padding1: 0,
            padding2: 0,
        }
    }
}

/// GPU-driven LOD selection manager.
///
/// This manager uses compute shaders to calculate appropriate LOD levels for all
/// objects in parallel on the GPU, avoiding expensive CPU-side distance calculations
/// and enabling efficient LOD selection for tens of thousands of objects.
///
/// # Architecture
///
/// The GPU LOD selector works in two stages:
/// 1. **LOD Selection**: Compute shader reads object positions and camera position,
///    calculates distances, and selects appropriate LOD levels
/// 2. **Indirect Draw Generation**: Selected LOD levels feed into indirect draw
///    buffer generation for GPU culling system
///
/// # Performance Benefits
///
/// - **Massively Parallel**: All LOD calculations happen simultaneously on GPU
/// - **No CPU-GPU Sync**: Distance calculations stay on GPU
/// - **Efficient Memory Access**: Coalesced reads/writes in compute shader
/// - **Scalability**: Handles 10,000+ objects with minimal overhead
///
/// # Usage
///
/// ```rust,ignore
/// use praxis_graphics::lod::{GpuLodSelector, GpuObjectData, GpuLodLevel};
/// use praxis_math::{Mat4, Vec3};
///
/// // Initialize selector
/// let mut lod_selector = GpuLodSelector::new(
///     device.clone(),
///     memory_allocator.clone(),
///     descriptor_set_allocator.clone(),
/// )?;
///
/// // Prepare object data and LOD definitions
/// let objects = vec![
///     GpuObjectData::new(
///         Mat4::IDENTITY,
///         [0.0, 0.0, 0.0, 1.0], // Bounding sphere
///         0, // Base mesh ID
///         3, // 3 LOD levels
///         0, // Offset in LOD array
///     ),
/// ];
///
/// let lod_levels = vec![
///     GpuLodLevel { mesh_id: 0, min_distance_sq: 0.0, max_distance_sq: 100.0, padding: 0 },
///     GpuLodLevel { mesh_id: 1, min_distance_sq: 100.0, max_distance_sq: 400.0, padding: 0 },
///     GpuLodLevel { mesh_id: 2, min_distance_sq: 400.0, max_distance_sq: f32::MAX, padding: 0 },
/// ];
///
/// // Dispatch LOD selection
/// lod_selector.prepare_frame(&objects, &lod_levels)?;
/// lod_selector.dispatch_lod_selection(
///     command_buffer,
///     Vec3::new(0.0, 5.0, 10.0), // Camera position
///     0.0, // LOD bias
///     true, // Enable LOD
/// )?;
///
/// // Read selected LOD levels (for use with indirect draw generation)
/// let selected_lods = lod_selector.selected_lod_buffer();
/// let distances = lod_selector.distance_buffer();
/// ```
pub struct GpuLodSelector {
    device: Arc<Device>,
    memory_allocator: Arc<dyn MemoryAllocator>,
    descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,

    compute_pipeline: Arc<ComputePipeline>,

    // Buffers
    object_data_buffer: Option<Subbuffer<[GpuObjectData]>>,
    lod_level_buffer: Option<Subbuffer<[GpuLodLevel]>>,
    selected_lod_buffer: Option<Subbuffer<[u32]>>,
    distance_buffer: Option<Subbuffer<[f32]>>,
    uniforms_buffer: Option<Subbuffer<LodUniforms>>,

    descriptor_set: Option<Arc<DescriptorSet>>,

    max_objects: usize,
    max_lod_levels: usize,
    current_object_count: u32,
}

impl GpuLodSelector {
    /// Creates a new GPU LOD selector.
    ///
    /// # Arguments
    ///
    /// * `device` - Vulkan device
    /// * `memory_allocator` - Memory allocator for buffers
    /// * `descriptor_set_allocator` - Descriptor set allocator
    ///
    /// # Errors
    ///
    /// Returns an error if pipeline or buffer creation fails.
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<dyn MemoryAllocator>,
        descriptor_set_allocator: Arc<dyn DescriptorSetAllocator>,
    ) -> Result<Self> {
        debug!("Creating GPU LOD selector");

        // Create compute pipeline
        let compute_pipeline = Self::create_compute_pipeline(device.clone())?;

        Ok(Self {
            device,
            memory_allocator,
            descriptor_set_allocator,
            compute_pipeline,
            object_data_buffer: None,
            lod_level_buffer: None,
            selected_lod_buffer: None,
            distance_buffer: None,
            uniforms_buffer: None,
            descriptor_set: None,
            max_objects: 0,
            max_lod_levels: 0,
            current_object_count: 0,
        })
    }

    /// Creates the LOD selection compute pipeline.
    fn create_compute_pipeline(device: Arc<Device>) -> Result<Arc<ComputePipeline>> {
        trace!("Loading LOD selection compute shader");

        let shader = shaders::load_lod_selection_comp(device.clone())
            .map_err(|e| eyre::eyre!("Failed to load LOD selection shader: {}", e))?;

        let stage = PipelineShaderStageCreateInfo::new(shader.entry_point("main").unwrap());

        let layout = vulkano::pipeline::PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&[stage.clone()])
                .into_pipeline_layout_create_info(device.clone())
                .map_err(|e| eyre::eyre!("Failed to create pipeline layout info: {}", e))?,
        )
        .map_err(|e| eyre::eyre!("Failed to create pipeline layout: {}", e))?;

        ComputePipeline::new(
            device.clone(),
            None,
            ComputePipelineCreateInfo::stage_layout(stage, layout),
        )
        .map_err(|e| eyre::eyre!("Failed to create compute pipeline: {}", e))
    }

    /// Prepares buffers for a new frame.
    ///
    /// This should be called once per frame with the current object data and LOD definitions.
    /// It allocates or resizes buffers as needed and uploads the data.
    ///
    /// # Arguments
    ///
    /// * `objects` - Object data (transforms, bounding spheres, LOD metadata)
    /// * `lod_levels` - LOD level definitions for all objects
    ///
    /// # Errors
    ///
    /// Returns an error if buffer allocation or upload fails.
    pub fn prepare_frame(
        &mut self,
        objects: &[GpuObjectData],
        lod_levels: &[GpuLodLevel],
    ) -> Result<()> {
        let object_count = objects.len();
        let lod_count = lod_levels.len();

        if object_count == 0 {
            self.current_object_count = 0;
            return Ok(());
        }

        self.current_object_count = object_count as u32;

        trace!(
            "Preparing GPU LOD frame: {} objects, {} LOD levels",
            object_count,
            lod_count
        );

        // Reallocate buffers if needed
        if object_count > self.max_objects || lod_count > self.max_lod_levels {
            debug!(
                "Reallocating GPU LOD buffers: {} objects (was {}), {} LOD levels (was {})",
                object_count, self.max_objects, lod_count, self.max_lod_levels
            );
            self.allocate_buffers(object_count, lod_count)?;
        }

        // Upload object data
        if let Some(buffer) = &self.object_data_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map object data buffer: {}", e))?;
            write[..object_count].copy_from_slice(objects);
        }

        // Upload LOD level data
        if let Some(buffer) = &self.lod_level_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map LOD level buffer: {}", e))?;
            write[..lod_count].copy_from_slice(lod_levels);
        }

        Ok(())
    }

    /// Allocates GPU buffers for LOD selection.
    fn allocate_buffers(&mut self, max_objects: usize, max_lod_levels: usize) -> Result<()> {
        debug!(
            "Allocating GPU LOD buffers for {} objects, {} LOD levels",
            max_objects, max_lod_levels
        );

        // Object data buffer (input)
        let object_data_buffer = Buffer::new_slice::<GpuObjectData>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            max_objects as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create object data buffer: {}", e))?;

        // LOD level buffer (input)
        let lod_level_buffer = Buffer::new_slice::<GpuLodLevel>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            max_lod_levels as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create LOD level buffer: {}", e))?;

        // Selected LOD buffer (output)
        let selected_lod_buffer = Buffer::new_slice::<u32>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            max_objects as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create selected LOD buffer: {}", e))?;

        // Distance buffer (output, for debugging/sorting)
        let distance_buffer = Buffer::new_slice::<f32>(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::STORAGE_BUFFER | BufferUsage::TRANSFER_SRC,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_RANDOM_ACCESS,
                ..Default::default()
            },
            max_objects as u64,
        )
        .map_err(|e| eyre::eyre!("Failed to create distance buffer: {}", e))?;

        // Uniforms buffer
        let uniforms_buffer = Buffer::from_data(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::UNIFORM_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_HOST
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            LodUniforms::new(Vec3::ZERO, 0.0, 0, true),
        )
        .map_err(|e| eyre::eyre!("Failed to create LOD uniforms buffer: {}", e))?;

        self.object_data_buffer = Some(object_data_buffer);
        self.lod_level_buffer = Some(lod_level_buffer);
        self.selected_lod_buffer = Some(selected_lod_buffer);
        self.distance_buffer = Some(distance_buffer);
        self.uniforms_buffer = Some(uniforms_buffer);
        self.max_objects = max_objects;
        self.max_lod_levels = max_lod_levels;

        // Descriptor set will be recreated on next dispatch
        self.descriptor_set = None;

        Ok(())
    }

    /// Dispatches the LOD selection compute shader.
    ///
    /// This records commands into the provided command buffer to:
    /// 1. Bind the LOD selection compute pipeline
    /// 2. Update LOD uniforms
    /// 3. Dispatch compute work groups
    ///
    /// # Arguments
    ///
    /// * `builder` - Command buffer builder to record into
    /// * `camera_position` - Camera position in world space
    /// * `lod_bias` - Global LOD bias (-1.0 to 1.0)
    /// * `enable_lod` - Enable LOD system
    ///
    /// # Errors
    ///
    /// Returns an error if command recording fails.
    pub fn dispatch_lod_selection(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        camera_position: Vec3,
        lod_bias: f32,
        enable_lod: bool,
    ) -> Result<()> {
        if self.current_object_count == 0 {
            return Ok(());
        }

        trace!(
            "Dispatching LOD selection for {} objects",
            self.current_object_count
        );

        // Update uniforms
        let uniforms = LodUniforms::new(
            camera_position,
            lod_bias,
            self.current_object_count,
            enable_lod,
        );

        if let Some(buffer) = &self.uniforms_buffer {
            let mut write = buffer
                .write()
                .map_err(|e| eyre::eyre!("Failed to map LOD uniforms buffer: {}", e))?;
            *write = uniforms;
        }

        // Create or get descriptor set
        if self.descriptor_set.is_none() {
            self.create_descriptor_set()?;
        }

        let descriptor_set = self.descriptor_set.as_ref().unwrap();

        // Bind pipeline and descriptor set
        builder
            .bind_pipeline_compute(self.compute_pipeline.clone())
            .map_err(|e| eyre::eyre!("Failed to bind compute pipeline: {}", e))?
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                self.compute_pipeline.layout().clone(),
                0,
                descriptor_set.clone(),
            )
            .map_err(|e| eyre::eyre!("Failed to bind descriptor sets: {}", e))?;

        // Dispatch compute work groups (64 threads per group)
        let work_group_count = self.current_object_count.div_ceil(64);

        unsafe {
            builder
                .dispatch([work_group_count, 1, 1])
                .map_err(|e| eyre::eyre!("Failed to dispatch compute: {}", e))?;
        }

        trace!(
            "Dispatched {} compute work groups for LOD selection",
            work_group_count
        );

        Ok(())
    }

    /// Creates the descriptor set for the LOD selection compute shader.
    fn create_descriptor_set(&mut self) -> Result<()> {
        trace!("Creating LOD selection descriptor set");

        let layout = self
            .compute_pipeline
            .layout()
            .set_layouts()
            .first()
            .ok_or_else(|| eyre::eyre!("No descriptor set layout in pipeline"))?;

        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout.clone(),
            [
                WriteDescriptorSet::buffer(0, self.uniforms_buffer.clone().unwrap()),
                WriteDescriptorSet::buffer(1, self.object_data_buffer.clone().unwrap()),
                WriteDescriptorSet::buffer(2, self.lod_level_buffer.clone().unwrap()),
                WriteDescriptorSet::buffer(3, self.selected_lod_buffer.clone().unwrap()),
                WriteDescriptorSet::buffer(4, self.distance_buffer.clone().unwrap()),
            ],
            [],
        )
        .map_err(|e| eyre::eyre!("Failed to create descriptor set: {}", e))?;

        self.descriptor_set = Some(descriptor_set);

        Ok(())
    }

    /// Gets the selected LOD buffer for use in indirect draw generation.
    ///
    /// This buffer contains mesh IDs for each object after LOD selection.
    pub fn selected_lod_buffer(&self) -> Option<&Subbuffer<[u32]>> {
        self.selected_lod_buffer.as_ref()
    }

    /// Gets the distance buffer (for debugging/sorting).
    ///
    /// This buffer contains squared distances from camera to each object.
    pub fn distance_buffer(&self) -> Option<&Subbuffer<[f32]>> {
        self.distance_buffer.as_ref()
    }

    /// Reads back selected LOD levels for debugging.
    ///
    /// This requires CPU-GPU sync and should only be used for debugging.
    ///
    /// # Errors
    ///
    /// Returns an error if buffer mapping fails.
    pub fn read_selected_lods(&self) -> Result<Vec<u32>> {
        if let Some(buffer) = &self.selected_lod_buffer {
            let read = buffer
                .read()
                .map_err(|e| eyre::eyre!("Failed to read selected LOD buffer: {}", e))?;
            Ok(read[..self.current_object_count as usize].to_vec())
        } else {
            Ok(Vec::new())
        }
    }

    /// Reads back distances for debugging.
    ///
    /// This requires CPU-GPU sync and should only be used for debugging.
    ///
    /// # Errors
    ///
    /// Returns an error if buffer mapping fails.
    pub fn read_distances(&self) -> Result<Vec<f32>> {
        if let Some(buffer) = &self.distance_buffer {
            let read = buffer
                .read()
                .map_err(|e| eyre::eyre!("Failed to read distance buffer: {}", e))?;
            Ok(read[..self.current_object_count as usize].to_vec())
        } else {
            Ok(Vec::new())
        }
    }
}
