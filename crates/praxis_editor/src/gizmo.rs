//! Transform gizmo system for visual 3D manipulation of entities.
//!
//! This module provides visual gizmos for transforming entities in the editor viewport.
//! Gizmos render as colored lines and cones, allowing intuitive manipulation via ray-based
//! interaction with axis-constrained movement, rotation, and scaling.
//!
//! # Features
//!
//! - **Visual 3D Gizmos**: Rendered as colored lines and cones (X=red, Y=green, Z=blue)
//! - **Ray-based Interaction**: Click and drag axes to manipulate transforms
//! - **Axis Constraints**: Operations constrained to the selected axis
//! - **Local/World Space**: Toggle between local and world space transformation
//! - **Three Modes**: Translate, rotate, and scale
//! - **Undo/Redo**: All operations are undoable
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_editor::{GizmoSystem, GizmoMode, GizmoSpace};
//! use praxis_ecs::World;
//!
//! let mut world = World::new();
//! world.insert_resource(GizmoSystem::new());
//!
//! // Set gizmo mode
//! // gizmo_system.set_mode(GizmoMode::Translate);
//! // gizmo_system.set_space(GizmoSpace::World);
//! ```

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::system::Resource;
use praxis_ecs::{CameraMatrices, GlobalTransform, Transform};
use praxis_math::{Quat, Vec2, Vec3, Vec4};

/// Gizmo mode determining which transform property to manipulate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoMode {
    /// Translate (move) mode - manipulates position.
    Translate,
    /// Rotate mode - manipulates rotation.
    Rotate,
    /// Scale mode - manipulates scale.
    Scale,
}

impl Default for GizmoMode {
    fn default() -> Self {
        Self::Translate
    }
}

/// Coordinate space for gizmo transformations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoSpace {
    /// World space - gizmo axes align with world coordinates.
    World,
    /// Local space - gizmo axes align with entity's rotation.
    Local,
}

impl Default for GizmoSpace {
    fn default() -> Self {
        Self::World
    }
}

/// Gizmo axis identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GizmoAxis {
    /// X axis (red).
    X,
    /// Y axis (green).
    Y,
    /// Z axis (blue).
    Z,
}

impl GizmoAxis {
    /// Gets the color for this axis.
    pub fn color(&self) -> Vec3 {
        match self {
            GizmoAxis::X => Vec3::new(1.0, 0.0, 0.0), // Red
            GizmoAxis::Y => Vec3::new(0.0, 1.0, 0.0), // Green
            GizmoAxis::Z => Vec3::new(0.0, 0.0, 1.0), // Blue
        }
    }

    /// Gets the highlight color for this axis (brighter).
    pub fn highlight_color(&self) -> Vec3 {
        match self {
            GizmoAxis::X => Vec3::new(1.0, 0.5, 0.5),
            GizmoAxis::Y => Vec3::new(0.5, 1.0, 0.5),
            GizmoAxis::Z => Vec3::new(0.5, 0.5, 1.0),
        }
    }

    /// Gets the direction vector for this axis in local space.
    pub fn direction(&self) -> Vec3 {
        match self {
            GizmoAxis::X => Vec3::X,
            GizmoAxis::Y => Vec3::Y,
            GizmoAxis::Z => Vec3::Z,
        }
    }
}

/// Current interaction state with a gizmo.
#[derive(Debug, Clone)]
pub struct GizmoInteraction {
    /// The axis being interacted with.
    pub axis: GizmoAxis,
    /// Screen position where interaction started.
    pub start_screen_pos: Vec2,
    /// World position of gizmo when interaction started.
    pub start_gizmo_position: Vec3,
    /// Initial transforms of affected entities.
    pub initial_transforms: Vec<(Entity, Transform)>,
    /// Current drag delta in world space.
    pub drag_delta: f32,
}

/// Individual gizmo instance for an entity or group.
#[derive(Debug, Clone)]
pub struct Gizmo {
    /// Center position of the gizmo in world space.
    pub position: Vec3,
    /// Rotation of the gizmo (for local space mode).
    pub rotation: Quat,
    /// Size/scale of the gizmo visualization.
    pub size: f32,
    /// Currently hovered axis, if any.
    pub hovered_axis: Option<GizmoAxis>,
}

impl Gizmo {
    /// Creates a new gizmo at the given position.
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            size: 1.0,
            hovered_axis: None,
        }
    }

    /// Creates a gizmo from a transform.
    pub fn from_transform(transform: &Transform) -> Self {
        Self {
            position: transform.translation,
            rotation: transform.rotation,
            size: 1.0,
            hovered_axis: None,
        }
    }

    /// Creates a gizmo from a global transform.
    pub fn from_global_transform(global_transform: &GlobalTransform) -> Self {
        Self {
            position: global_transform.translation(),
            rotation: Quat::IDENTITY, // Extract rotation from matrix if needed
            size: 1.0,
            hovered_axis: None,
        }
    }

    /// Updates the gizmo to match a transform.
    pub fn update_from_transform(&mut self, transform: &Transform) {
        self.position = transform.translation;
        self.rotation = transform.rotation;
    }

    /// Performs ray-cast picking against gizmo axes.
    ///
    /// Returns the closest axis hit by the ray, if any.
    pub fn raycast(
        &self,
        ray_origin: Vec3,
        ray_direction: Vec3,
        mode: GizmoMode,
        space: GizmoSpace,
    ) -> Option<GizmoAxis> {
        let rotation = if space == GizmoSpace::Local {
            self.rotation
        } else {
            Quat::IDENTITY
        };

        let axes = [GizmoAxis::X, GizmoAxis::Y, GizmoAxis::Z];
        let mut closest_axis = None;
        let mut closest_distance = f32::MAX;

        for axis in &axes {
            let axis_dir = rotation * axis.direction();
            let axis_length = self.size
                * match mode {
                    GizmoMode::Translate => 2.0,
                    GizmoMode::Rotate => 1.5,
                    GizmoMode::Scale => 2.0,
                };

            // Ray-line distance calculation
            let to_gizmo = self.position - ray_origin;
            let ray_proj = to_gizmo.dot(ray_direction);

            if ray_proj < 0.0 {
                continue; // Behind camera
            }

            // Find closest points on ray and axis
            let axis_start = self.position;
            let axis_end = self.position + axis_dir * axis_length;

            let axis_vec = axis_end - axis_start;
            let axis_length_sq = axis_vec.length_squared();

            if axis_length_sq < 0.0001 {
                continue;
            }

            // Parametric closest point calculation
            let t_axis = (to_gizmo.dot(axis_vec)) / axis_length_sq;
            let t_axis = t_axis.clamp(0.0, 1.0);

            let point_on_axis = axis_start + axis_vec * t_axis;
            let point_on_ray = ray_origin + ray_direction * ray_proj;

            let distance = (point_on_axis - point_on_ray).length();

            // Threshold for picking (in world units)
            let pick_threshold = self.size * 0.2;

            if distance < pick_threshold && distance < closest_distance {
                closest_distance = distance;
                closest_axis = Some(*axis);
            }
        }

        closest_axis
    }

    /// Gets the lines to render for this gizmo.
    ///
    /// Returns a list of (start, end, color) tuples for rendering.
    pub fn get_lines(&self, mode: GizmoMode, space: GizmoSpace) -> Vec<(Vec3, Vec3, Vec3)> {
        let rotation = if space == GizmoSpace::Local {
            self.rotation
        } else {
            Quat::IDENTITY
        };

        let axes = [GizmoAxis::X, GizmoAxis::Y, GizmoAxis::Z];
        let mut lines = Vec::new();

        for axis in &axes {
            let axis_dir = rotation * axis.direction();
            let length = self.size
                * match mode {
                    GizmoMode::Translate => 2.0,
                    GizmoMode::Rotate => 1.5,
                    GizmoMode::Scale => 2.0,
                };

            let color = if self.hovered_axis == Some(*axis) {
                axis.highlight_color()
            } else {
                axis.color()
            };

            let start = self.position;
            let end = self.position + axis_dir * length;

            lines.push((start, end, color));

            // Add arrowhead for translate mode
            if mode == GizmoMode::Translate {
                let arrow_size = self.size * 0.3;
                let arrow_base = end - axis_dir * arrow_size;

                // Create perpendicular vectors for arrow
                let perp1 = if axis_dir.dot(Vec3::Y).abs() < 0.9 {
                    axis_dir.cross(Vec3::Y).normalize()
                } else {
                    axis_dir.cross(Vec3::X).normalize()
                };
                let perp2 = axis_dir.cross(perp1).normalize();

                // Arrow lines
                for i in 0..4 {
                    let angle = (i as f32) * std::f32::consts::PI * 0.5;
                    let offset = (perp1 * angle.cos() + perp2 * angle.sin()) * arrow_size * 0.5;
                    lines.push((arrow_base + offset, end, color));
                }
            }
        }

        lines
    }
}

/// Transform gizmo component attached to entities that should have gizmos.
#[derive(Debug, Clone, Copy, Component)]
pub struct TransformGizmo {
    /// Whether the gizmo is currently visible.
    pub visible: bool,
    /// Custom size multiplier for this gizmo.
    pub size_multiplier: f32,
}

impl Default for TransformGizmo {
    fn default() -> Self {
        Self {
            visible: true,
            size_multiplier: 1.0,
        }
    }
}

impl TransformGizmo {
    /// Creates a new transform gizmo component.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the visibility of the gizmo.
    pub fn set_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Sets a custom size multiplier for the gizmo.
    pub fn with_size(mut self, size: f32) -> Self {
        self.size_multiplier = size;
        self
    }
}

/// Gizmo system resource managing all gizmo state and interaction.
///
/// This resource tracks the current gizmo mode, space, active interactions,
/// and provides methods for updating and rendering gizmos.
#[derive(Resource)]
pub struct GizmoSystem {
    /// Current gizmo mode (translate/rotate/scale).
    mode: GizmoMode,
    /// Current coordinate space (world/local).
    space: GizmoSpace,
    /// Currently active gizmo for selected entities.
    active_gizmo: Option<Gizmo>,
    /// Current interaction state, if dragging.
    interaction: Option<GizmoInteraction>,
    /// Whether gizmos are enabled globally.
    enabled: bool,
}

impl Default for GizmoSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl GizmoSystem {
    /// Creates a new gizmo system.
    pub fn new() -> Self {
        Self {
            mode: GizmoMode::default(),
            space: GizmoSpace::default(),
            active_gizmo: None,
            interaction: None,
            enabled: true,
        }
    }

    /// Gets the current gizmo mode.
    pub fn mode(&self) -> GizmoMode {
        self.mode
    }

    /// Sets the gizmo mode.
    pub fn set_mode(&mut self, mode: GizmoMode) {
        if self.mode != mode {
            self.mode = mode;
            // Cancel any active interaction when mode changes
            self.interaction = None;
        }
    }

    /// Cycles to the next gizmo mode (translate -> rotate -> scale -> translate).
    pub fn cycle_mode(&mut self) {
        self.mode = match self.mode {
            GizmoMode::Translate => GizmoMode::Rotate,
            GizmoMode::Rotate => GizmoMode::Scale,
            GizmoMode::Scale => GizmoMode::Translate,
        };
        self.interaction = None;
    }

    /// Gets the current coordinate space.
    pub fn space(&self) -> GizmoSpace {
        self.space
    }

    /// Sets the coordinate space.
    pub fn set_space(&mut self, space: GizmoSpace) {
        if self.space != space {
            self.space = space;
            // Cancel any active interaction when space changes
            self.interaction = None;
        }
    }

    /// Toggles between world and local space.
    pub fn toggle_space(&mut self) {
        self.space = match self.space {
            GizmoSpace::World => GizmoSpace::Local,
            GizmoSpace::Local => GizmoSpace::World,
        };
        self.interaction = None;
    }

    /// Returns whether gizmos are enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Sets whether gizmos are enabled.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.interaction = None;
        }
    }

    /// Gets the active gizmo, if any.
    pub fn active_gizmo(&self) -> Option<&Gizmo> {
        self.active_gizmo.as_ref()
    }

    /// Gets the active gizmo mutably.
    pub fn active_gizmo_mut(&mut self) -> Option<&mut Gizmo> {
        self.active_gizmo.as_mut()
    }

    /// Updates the active gizmo based on selected entities.
    ///
    /// Calculates the center position and average rotation of selected entities.
    pub fn update_gizmo_for_selection(&mut self, entities: &[(Entity, &Transform)]) {
        if entities.is_empty() {
            self.active_gizmo = None;
            return;
        }

        // Calculate center of selection
        let center = entities
            .iter()
            .map(|(_, t)| t.translation)
            .fold(Vec3::ZERO, |acc, pos| acc + pos)
            / entities.len() as f32;

        // For local space, use the first entity's rotation
        // For multiple entities, this is a simplification
        let rotation = if entities.len() == 1 {
            entities[0].1.rotation
        } else {
            Quat::IDENTITY
        };

        let mut gizmo = Gizmo::new(center);
        gizmo.rotation = rotation;
        gizmo.size = 1.0;

        self.active_gizmo = Some(gizmo);
    }

    /// Returns whether an interaction is currently active.
    pub fn is_interacting(&self) -> bool {
        self.interaction.is_some()
    }

    /// Gets the current interaction, if any.
    pub fn interaction(&self) -> Option<&GizmoInteraction> {
        self.interaction.as_ref()
    }

    /// Starts an interaction with the gizmo.
    ///
    /// # Arguments
    ///
    /// * `screen_pos` - Mouse position in screen space
    /// * `camera_matrices` - Camera view/projection matrices
    /// * `entities` - Entities to manipulate with their current transforms
    pub fn start_interaction(
        &mut self,
        screen_pos: Vec2,
        camera_matrices: &CameraMatrices,
        camera_position: Vec3,
        entities: Vec<(Entity, Transform)>,
    ) -> bool {
        if !self.enabled {
            return false;
        }

        let Some(gizmo) = &self.active_gizmo else {
            return false;
        };

        // Convert screen position to ray
        let (ray_origin, ray_direction) =
            screen_to_ray(screen_pos, camera_matrices, camera_position);

        // Check if ray hits any axis
        if let Some(axis) = gizmo.raycast(ray_origin, ray_direction, self.mode, self.space) {
            self.interaction = Some(GizmoInteraction {
                axis,
                start_screen_pos: screen_pos,
                start_gizmo_position: gizmo.position,
                initial_transforms: entities,
                drag_delta: 0.0,
            });
            return true;
        }

        false
    }

    /// Updates an active interaction with new mouse position.
    ///
    /// Returns the new transforms for affected entities, if any.
    pub fn update_interaction(
        &mut self,
        current_screen_pos: Vec2,
        _camera_matrices: &CameraMatrices,
        _camera_position: Vec3,
    ) -> Option<Vec<(Entity, Transform)>> {
        let interaction = self.interaction.as_mut()?;
        let gizmo = self.active_gizmo.as_ref()?;

        let rotation = if self.space == GizmoSpace::Local {
            gizmo.rotation
        } else {
            Quat::IDENTITY
        };

        let axis_dir = rotation * interaction.axis.direction();

        // Calculate drag delta along the axis
        let delta_screen = current_screen_pos - interaction.start_screen_pos;
        let drag_amount = delta_screen.length() * delta_screen.dot(Vec2::new(1.0, -1.0)).signum();

        // Scale factor for sensitivity
        let sensitivity = 0.01;
        interaction.drag_delta = drag_amount * sensitivity;

        // Apply transformation based on mode
        let mut new_transforms = Vec::new();

        for (entity, initial_transform) in &interaction.initial_transforms {
            let mut new_transform = *initial_transform;

            match self.mode {
                GizmoMode::Translate => {
                    let translation_offset = axis_dir * interaction.drag_delta;
                    new_transform.translation = initial_transform.translation + translation_offset;
                }
                GizmoMode::Rotate => {
                    let angle = interaction.drag_delta;
                    let rotation_delta = Quat::from_axis_angle(axis_dir, angle);
                    new_transform.rotation = rotation_delta * initial_transform.rotation;
                }
                GizmoMode::Scale => {
                    let scale_factor = 1.0 + interaction.drag_delta;
                    let scale_factor = scale_factor.max(0.01); // Prevent negative scale

                    match interaction.axis {
                        GizmoAxis::X => {
                            new_transform.scale.x = initial_transform.scale.x * scale_factor
                        }
                        GizmoAxis::Y => {
                            new_transform.scale.y = initial_transform.scale.y * scale_factor
                        }
                        GizmoAxis::Z => {
                            new_transform.scale.z = initial_transform.scale.z * scale_factor
                        }
                    }
                }
            }

            new_transforms.push((*entity, new_transform));
        }

        Some(new_transforms)
    }

    /// Ends the current interaction.
    ///
    /// Returns the interaction data for undo/redo, if any.
    pub fn end_interaction(&mut self) -> Option<GizmoInteraction> {
        self.interaction.take()
    }

    /// Updates gizmo hover state based on mouse position.
    pub fn update_hover(
        &mut self,
        screen_pos: Vec2,
        camera_matrices: &CameraMatrices,
        camera_position: Vec3,
    ) {
        if self.interaction.is_some() {
            return; // Don't update hover during interaction
        }

        let Some(gizmo) = &mut self.active_gizmo else {
            return;
        };

        let (ray_origin, ray_direction) =
            screen_to_ray(screen_pos, camera_matrices, camera_position);

        gizmo.hovered_axis = gizmo.raycast(ray_origin, ray_direction, self.mode, self.space);
    }
}

/// Converts screen position to a ray in world space.
///
/// # Arguments
///
/// * `screen_pos` - Screen position in pixels
/// * `camera_matrices` - Camera matrices
/// * `camera_position` - Camera world position
///
/// # Returns
///
/// (ray_origin, ray_direction) in world space
fn screen_to_ray(
    screen_pos: Vec2,
    camera_matrices: &CameraMatrices,
    camera_position: Vec3,
) -> (Vec3, Vec3) {
    // Note: This is a simplified version. In practice, you'd need viewport size
    // to convert screen coordinates to NDC properly.

    // Assume normalized coordinates for now
    let ndc_x = screen_pos.x * 2.0 - 1.0;
    let ndc_y = 1.0 - screen_pos.y * 2.0;

    // Unproject through inverse view-projection
    let inv_vp = camera_matrices.view_projection.inverse();

    let near_point = inv_vp * Vec4::new(ndc_x, ndc_y, 0.0, 1.0);
    let near_point = near_point.truncate() / near_point.w;

    let far_point = inv_vp * Vec4::new(ndc_x, ndc_y, 1.0, 1.0);
    let far_point = far_point.truncate() / far_point.w;

    let ray_origin = camera_position;
    let ray_direction = (far_point - near_point).normalize();

    (ray_origin, ray_direction)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gizmo_system_creation() {
        let system = GizmoSystem::new();
        assert_eq!(system.mode(), GizmoMode::Translate);
        assert_eq!(system.space(), GizmoSpace::World);
        assert!(system.is_enabled());
        assert!(!system.is_interacting());
    }

    #[test]
    fn test_gizmo_mode_cycle() {
        let mut system = GizmoSystem::new();
        assert_eq!(system.mode(), GizmoMode::Translate);

        system.cycle_mode();
        assert_eq!(system.mode(), GizmoMode::Rotate);

        system.cycle_mode();
        assert_eq!(system.mode(), GizmoMode::Scale);

        system.cycle_mode();
        assert_eq!(system.mode(), GizmoMode::Translate);
    }

    #[test]
    fn test_gizmo_space_toggle() {
        let mut system = GizmoSystem::new();
        assert_eq!(system.space(), GizmoSpace::World);

        system.toggle_space();
        assert_eq!(system.space(), GizmoSpace::Local);

        system.toggle_space();
        assert_eq!(system.space(), GizmoSpace::World);
    }

    #[test]
    fn test_gizmo_creation() {
        let position = Vec3::new(1.0, 2.0, 3.0);
        let gizmo = Gizmo::new(position);

        assert_eq!(gizmo.position, position);
        assert_eq!(gizmo.rotation, Quat::IDENTITY);
        assert_eq!(gizmo.size, 1.0);
        assert_eq!(gizmo.hovered_axis, None);
    }

    #[test]
    fn test_gizmo_from_transform() {
        let transform = Transform {
            translation: Vec3::new(5.0, 10.0, 15.0),
            rotation: Quat::from_rotation_y(1.0),
            scale: Vec3::ONE,
        };

        let gizmo = Gizmo::from_transform(&transform);
        assert_eq!(gizmo.position, transform.translation);
        assert_eq!(gizmo.rotation, transform.rotation);
    }

    #[test]
    fn test_gizmo_axis_colors() {
        assert_eq!(GizmoAxis::X.color(), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(GizmoAxis::Y.color(), Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(GizmoAxis::Z.color(), Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_gizmo_axis_directions() {
        assert_eq!(GizmoAxis::X.direction(), Vec3::X);
        assert_eq!(GizmoAxis::Y.direction(), Vec3::Y);
        assert_eq!(GizmoAxis::Z.direction(), Vec3::Z);
    }

    #[test]
    fn test_transform_gizmo_component() {
        let gizmo = TransformGizmo::new();
        assert!(gizmo.visible);
        assert_eq!(gizmo.size_multiplier, 1.0);

        let gizmo = TransformGizmo::new().set_visible(false).with_size(2.0);
        assert!(!gizmo.visible);
        assert_eq!(gizmo.size_multiplier, 2.0);
    }

    #[test]
    fn test_gizmo_enable_disable() {
        let mut system = GizmoSystem::new();
        assert!(system.is_enabled());

        system.set_enabled(false);
        assert!(!system.is_enabled());

        system.set_enabled(true);
        assert!(system.is_enabled());
    }

    #[test]
    fn test_gizmo_mode_change_cancels_interaction() {
        let mut system = GizmoSystem::new();

        // Simulate an interaction
        system.interaction = Some(GizmoInteraction {
            axis: GizmoAxis::X,
            start_screen_pos: Vec2::ZERO,
            start_gizmo_position: Vec3::ZERO,
            initial_transforms: vec![],
            drag_delta: 0.0,
        });

        assert!(system.is_interacting());

        system.set_mode(GizmoMode::Rotate);
        assert!(!system.is_interacting());
    }

    #[test]
    fn test_gizmo_space_change_cancels_interaction() {
        let mut system = GizmoSystem::new();

        system.interaction = Some(GizmoInteraction {
            axis: GizmoAxis::X,
            start_screen_pos: Vec2::ZERO,
            start_gizmo_position: Vec3::ZERO,
            initial_transforms: vec![],
            drag_delta: 0.0,
        });

        assert!(system.is_interacting());

        system.set_space(GizmoSpace::Local);
        assert!(!system.is_interacting());
    }
}
