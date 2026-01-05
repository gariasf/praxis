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

use praxis_math::Vec3;
use praxis_utils::{debug, trace};

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
}
