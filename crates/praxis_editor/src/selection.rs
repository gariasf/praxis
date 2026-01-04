//! Selection system for editor entity selection.
//!
//! This module provides a comprehensive selection system with support for:
//! - Multi-entity selection with add/remove/toggle modes
//! - Click-to-select with raycast picking in viewport
//! - Marquee (box) selection in viewport
//! - Keyboard shortcuts (Ctrl+A for select all, Ctrl+D for deselect all)
//! - Selection changed events for UI updates
//!
//! # Architecture
//!
//! The selection system uses ECS resources and components:
//! - **`SelectionSystem`**: Resource managing selected entities and selection state
//! - **`Selectable`**: Component marking entities that can be selected
//! - **`Selected`**: Component marking entities that are currently selected
//! - **`SelectionEvent`**: Events fired when selection changes
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_editor::{SelectionSystem, Selectable, update_selection_system};
//! use praxis_ecs::{World, Schedule, Transform};
//!
//! let mut world = World::new();
//! world.insert_resource(SelectionSystem::new());
//!
//! // Make an entity selectable
//! world.spawn((
//!     Transform::default(),
//!     Selectable,
//! ));
//!
//! // Add selection system to schedule
//! let mut schedule = Schedule::default();
//! schedule.add_systems(update_selection_system);
//! ```
//!
//! # Selection Modes
//!
//! - **Replace**: Clear existing selection and select new entities
//! - **Add**: Add entities to existing selection (Shift+Click)
//! - **Remove**: Remove entities from selection (Ctrl+Click)
//! - **Toggle**: Toggle entity selection state (Alt+Click)

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
use bevy_ecs::system::{Commands, Query, Res, ResMut, Resource};
use praxis_ecs::{CameraMatrices, GlobalTransform, Transform};
use praxis_input::InputState;
use praxis_math::{Vec2, Vec3, Vec4};
use std::collections::{HashSet, VecDeque};
use winit::keyboard::KeyCode;

/// Component marking an entity as selectable in the editor.
///
/// Only entities with this component can be selected by the selection system.
/// This allows you to filter which entities appear in the editor selection.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::Selectable;
/// use praxis_ecs::{World, Transform};
///
/// let mut world = World::new();
///
/// // Make an entity selectable
/// world.spawn((
///     Transform::default(),
///     Selectable,
/// ));
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selectable;

/// Component marking an entity as currently selected.
///
/// This component is automatically added/removed by the SelectionSystem
/// when entities are selected or deselected. You can query for this
/// component to implement selection-specific rendering or behavior.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::Selected;
/// use praxis_ecs::Query;
///
/// fn highlight_selected(query: Query<&Selected>) {
///     for _selected in query.iter() {
///         // Render selection highlight
///     }
/// }
/// ```
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selected;

/// Selection mode for multi-entity selection.
///
/// Determines how new selections interact with the existing selection set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionMode {
    /// Replace the entire selection with new entities.
    Replace,
    /// Add new entities to the existing selection.
    Add,
    /// Remove entities from the existing selection.
    Remove,
    /// Toggle the selection state of entities.
    Toggle,
}

/// Selection event fired when the selection changes.
///
/// These events are collected in the SelectionSystem and can be consumed
/// by UI panels or other systems that need to react to selection changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionEvent {
    /// One or more entities were selected.
    Selected(Vec<Entity>),
    /// One or more entities were deselected.
    Deselected(Vec<Entity>),
    /// The entire selection was cleared.
    Cleared,
    /// Selection was changed (generic change event).
    Changed,
}

/// Marquee selection state for box selection in viewport.
#[derive(Debug, Clone)]
struct MarqueeSelection {
    /// Starting position in screen space (pixels).
    start: Vec2,
    /// Current position in screen space (pixels).
    current: Vec2,
    /// Whether marquee selection is active.
    active: bool,
}

impl MarqueeSelection {
    fn new() -> Self {
        Self {
            start: Vec2::ZERO,
            current: Vec2::ZERO,
            active: false,
        }
    }

    /// Start marquee selection at the given screen position.
    fn start(&mut self, position: Vec2) {
        self.start = position;
        self.current = position;
        self.active = true;
    }

    /// Update the current marquee selection position.
    fn update(&mut self, position: Vec2) {
        self.current = position;
    }

    /// End marquee selection and return the selection rectangle.
    fn end(&mut self) -> Option<(Vec2, Vec2)> {
        if self.active {
            self.active = false;
            let min = Vec2::new(
                self.start.x.min(self.current.x),
                self.start.y.min(self.current.y),
            );
            let max = Vec2::new(
                self.start.x.max(self.current.x),
                self.start.y.max(self.current.y),
            );
            Some((min, max))
        } else {
            None
        }
    }

    /// Cancel marquee selection without selecting anything.
    fn cancel(&mut self) {
        self.active = false;
    }

    /// Check if marquee selection is active.
    fn is_active(&self) -> bool {
        self.active
    }

    /// Get the current marquee rectangle (min, max).
    fn get_rect(&self) -> (Vec2, Vec2) {
        let min = Vec2::new(
            self.start.x.min(self.current.x),
            self.start.y.min(self.current.y),
        );
        let max = Vec2::new(
            self.start.x.max(self.current.x),
            self.start.y.max(self.current.y),
        );
        (min, max)
    }
}

/// Selection system resource managing entity selection state.
///
/// This resource tracks which entities are selected, handles selection operations,
/// and fires selection events. It supports multiple selection modes and provides
/// both programmatic and input-driven selection.
///
/// # Features
///
/// - **Multi-entity selection**: Select multiple entities at once
/// - **Selection modes**: Replace, Add, Remove, Toggle
/// - **Raycast picking**: Click entities in viewport to select them
/// - **Marquee selection**: Drag a box to select multiple entities
/// - **Keyboard shortcuts**: Ctrl+A (select all), Ctrl+D (deselect all)
/// - **Selection events**: Track when selection changes
///
/// # Example
///
/// ```rust,ignore
/// use praxis_editor::SelectionSystem;
/// use bevy_ecs::world::World;
///
/// let mut world = World::new();
/// let mut selection = SelectionSystem::new();
///
/// // Programmatically select an entity
/// let entity = world.spawn_empty().id();
/// selection.select_entity(entity);
///
/// // Check if selected
/// assert!(selection.is_selected(entity));
///
/// // Get all selected entities
/// let selected = selection.selected_entities();
/// ```
#[derive(Resource, Debug, Clone)]
pub struct SelectionSystem {
    /// Set of currently selected entities.
    selected: HashSet<Entity>,
    /// Recent selection events (ring buffer).
    events: VecDeque<SelectionEvent>,
    /// Maximum number of events to keep in history.
    max_events: usize,
    /// Marquee selection state.
    marquee: MarqueeSelection,
    /// Last mouse position for raycast picking.
    #[allow(dead_code)]
    last_mouse_pos: Vec2,
    /// Whether selection input is enabled.
    input_enabled: bool,
}

impl Default for SelectionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionSystem {
    /// Creates a new selection system.
    pub fn new() -> Self {
        Self {
            selected: HashSet::new(),
            events: VecDeque::new(),
            max_events: 100,
            marquee: MarqueeSelection::new(),
            last_mouse_pos: Vec2::ZERO,
            input_enabled: true,
        }
    }

    /// Selects a single entity with the given mode.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to select
    /// * `mode` - How to modify the selection
    pub fn select_entity(&mut self, entity: Entity, mode: SelectionMode) {
        match mode {
            SelectionMode::Replace => {
                let was_empty = self.selected.is_empty();
                self.selected.clear();
                self.selected.insert(entity);
                if !was_empty {
                    self.push_event(SelectionEvent::Cleared);
                }
                self.push_event(SelectionEvent::Selected(vec![entity]));
                self.push_event(SelectionEvent::Changed);
            }
            SelectionMode::Add => {
                if self.selected.insert(entity) {
                    self.push_event(SelectionEvent::Selected(vec![entity]));
                    self.push_event(SelectionEvent::Changed);
                }
            }
            SelectionMode::Remove => {
                if self.selected.remove(&entity) {
                    self.push_event(SelectionEvent::Deselected(vec![entity]));
                    self.push_event(SelectionEvent::Changed);
                }
            }
            SelectionMode::Toggle => {
                if self.selected.contains(&entity) {
                    self.selected.remove(&entity);
                    self.push_event(SelectionEvent::Deselected(vec![entity]));
                } else {
                    self.selected.insert(entity);
                    self.push_event(SelectionEvent::Selected(vec![entity]));
                }
                self.push_event(SelectionEvent::Changed);
            }
        }
    }

    /// Selects multiple entities with the given mode.
    ///
    /// # Arguments
    ///
    /// * `entities` - Iterator of entities to select
    /// * `mode` - How to modify the selection
    pub fn select_entities<I>(&mut self, entities: I, mode: SelectionMode)
    where
        I: IntoIterator<Item = Entity>,
    {
        let entities: Vec<Entity> = entities.into_iter().collect();
        if entities.is_empty() {
            return;
        }

        match mode {
            SelectionMode::Replace => {
                let was_empty = self.selected.is_empty();
                self.selected.clear();
                if !was_empty {
                    self.push_event(SelectionEvent::Cleared);
                }
                self.selected.extend(entities.iter());
                self.push_event(SelectionEvent::Selected(entities));
                self.push_event(SelectionEvent::Changed);
            }
            SelectionMode::Add => {
                let mut added = Vec::new();
                for entity in entities {
                    if self.selected.insert(entity) {
                        added.push(entity);
                    }
                }
                if !added.is_empty() {
                    self.push_event(SelectionEvent::Selected(added));
                    self.push_event(SelectionEvent::Changed);
                }
            }
            SelectionMode::Remove => {
                let mut removed = Vec::new();
                for entity in entities {
                    if self.selected.remove(&entity) {
                        removed.push(entity);
                    }
                }
                if !removed.is_empty() {
                    self.push_event(SelectionEvent::Deselected(removed));
                    self.push_event(SelectionEvent::Changed);
                }
            }
            SelectionMode::Toggle => {
                let mut added = Vec::new();
                let mut removed = Vec::new();
                for entity in entities {
                    if self.selected.contains(&entity) {
                        self.selected.remove(&entity);
                        removed.push(entity);
                    } else {
                        self.selected.insert(entity);
                        added.push(entity);
                    }
                }
                let has_added = !added.is_empty();
                let has_removed = !removed.is_empty();
                if has_removed {
                    self.push_event(SelectionEvent::Deselected(removed));
                }
                if has_added {
                    self.push_event(SelectionEvent::Selected(added));
                }
                if has_added || has_removed {
                    self.push_event(SelectionEvent::Changed);
                }
            }
        }
    }

    /// Deselects a single entity.
    pub fn deselect_entity(&mut self, entity: Entity) {
        if self.selected.remove(&entity) {
            self.push_event(SelectionEvent::Deselected(vec![entity]));
            self.push_event(SelectionEvent::Changed);
        }
    }

    /// Clears the entire selection.
    pub fn clear(&mut self) {
        if !self.selected.is_empty() {
            self.selected.clear();
            self.push_event(SelectionEvent::Cleared);
            self.push_event(SelectionEvent::Changed);
        }
    }

    /// Checks if an entity is selected.
    pub fn is_selected(&self, entity: Entity) -> bool {
        self.selected.contains(&entity)
    }

    /// Returns the number of selected entities.
    pub fn selected_count(&self) -> usize {
        self.selected.len()
    }

    /// Returns true if no entities are selected.
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Returns an iterator over selected entities.
    pub fn selected_entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.selected.iter().copied()
    }

    /// Returns a slice of recent selection events.
    pub fn events(&self) -> &VecDeque<SelectionEvent> {
        &self.events
    }

    /// Consumes and returns all pending selection events.
    pub fn drain_events(&mut self) -> Vec<SelectionEvent> {
        self.events.drain(..).collect()
    }

    /// Pushes a selection event to the event queue.
    fn push_event(&mut self, event: SelectionEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Enables or disables selection input handling.
    pub fn set_input_enabled(&mut self, enabled: bool) {
        self.input_enabled = enabled;
    }

    /// Returns true if selection input is enabled.
    pub fn is_input_enabled(&self) -> bool {
        self.input_enabled
    }

    /// Starts marquee selection at the given screen position.
    pub fn start_marquee(&mut self, position: Vec2) {
        self.marquee.start(position);
    }

    /// Updates the marquee selection to the given screen position.
    pub fn update_marquee(&mut self, position: Vec2) {
        self.marquee.update(position);
    }

    /// Ends marquee selection and returns the selection rectangle.
    pub fn end_marquee(&mut self) -> Option<(Vec2, Vec2)> {
        self.marquee.end()
    }

    /// Cancels marquee selection without selecting anything.
    pub fn cancel_marquee(&mut self) {
        self.marquee.cancel();
    }

    /// Returns true if marquee selection is active.
    pub fn is_marquee_active(&self) -> bool {
        self.marquee.is_active()
    }

    /// Gets the current marquee rectangle (min, max).
    pub fn get_marquee_rect(&self) -> Option<(Vec2, Vec2)> {
        if self.marquee.is_active() {
            Some(self.marquee.get_rect())
        } else {
            None
        }
    }

    /// Performs raycast picking to find entity at screen position.
    ///
    /// # Arguments
    ///
    /// * `screen_pos` - Position in screen space (pixels)
    /// * `viewport_size` - Size of viewport in pixels
    /// * `camera_transform` - Camera transform
    /// * `camera_matrices` - Camera view/projection matrices
    /// * `selectable_query` - Query for selectable entities with transforms
    ///
    /// # Returns
    ///
    /// The entity that was picked, or None if nothing was hit.
    pub fn raycast_pick(
        &self,
        screen_pos: Vec2,
        viewport_size: Vec2,
        camera_transform: &Transform,
        camera_matrices: &CameraMatrices,
        selectable_query: &Query<(Entity, &GlobalTransform), With<Selectable>>,
    ) -> Option<Entity> {
        // Convert screen space to NDC (Normalized Device Coordinates)
        let ndc_x = (2.0 * screen_pos.x) / viewport_size.x - 1.0;
        let ndc_y = 1.0 - (2.0 * screen_pos.y) / viewport_size.y;

        // Unproject to get ray direction
        let ray = screen_to_ray(Vec2::new(ndc_x, ndc_y), camera_matrices);

        let ray_origin = camera_transform.translation;
        let ray_dir = camera_transform.rotation * ray.normalize();

        // Find closest entity intersecting the ray
        let mut closest_entity = None;
        let mut closest_distance = f32::MAX;

        for (entity, global_transform) in selectable_query.iter() {
            let entity_pos = global_transform.translation();

            // Simple sphere-based picking (radius = 1.0 for now)
            // In a real implementation, you'd use the actual entity bounds
            let to_entity = entity_pos - ray_origin;
            let projection = to_entity.dot(ray_dir);

            if projection < 0.0 {
                continue; // Behind camera
            }

            let closest_point = ray_origin + ray_dir * projection;
            let distance_to_ray = (entity_pos - closest_point).length();
            let pick_radius = 1.0; // TODO: Use actual entity bounds

            if distance_to_ray <= pick_radius && projection < closest_distance {
                closest_distance = projection;
                closest_entity = Some(entity);
            }
        }

        closest_entity
    }

    /// Performs marquee selection to find entities within screen rectangle.
    ///
    /// # Arguments
    ///
    /// * `rect_min` - Minimum corner of selection rectangle in screen space
    /// * `rect_max` - Maximum corner of selection rectangle in screen space  
    /// * `viewport_size` - Size of viewport in pixels
    /// * `camera_matrices` - Camera view/projection matrices
    /// * `selectable_query` - Query for selectable entities with transforms
    ///
    /// # Returns
    ///
    /// Vector of entities within the selection rectangle.
    pub fn marquee_pick(
        &self,
        rect_min: Vec2,
        rect_max: Vec2,
        viewport_size: Vec2,
        camera_matrices: &CameraMatrices,
        selectable_query: &Query<(Entity, &GlobalTransform), With<Selectable>>,
    ) -> Vec<Entity> {
        let mut selected = Vec::new();

        for (entity, global_transform) in selectable_query.iter() {
            let world_pos = global_transform.translation();

            // Project world position to screen space
            if let Some(screen_pos) = world_to_screen(world_pos, camera_matrices, viewport_size) {
                // Check if screen position is within the marquee rectangle
                if screen_pos.x >= rect_min.x
                    && screen_pos.x <= rect_max.x
                    && screen_pos.y >= rect_min.y
                    && screen_pos.y <= rect_max.y
                {
                    selected.push(entity);
                }
            }
        }

        selected
    }
}

/// Converts screen space coordinates to a ray in view space.
///
/// # Arguments
///
/// * `ndc` - Normalized device coordinates (-1 to 1)
/// * `camera_matrices` - Camera view/projection matrices
///
/// # Returns
///
/// Ray direction in view space
fn screen_to_ray(ndc: Vec2, camera_matrices: &CameraMatrices) -> Vec3 {
    // Compute inverse projection matrix
    let inv_projection = camera_matrices.projection.inverse();

    // Convert NDC to view space
    let clip = Vec4::new(ndc.x, ndc.y, -1.0, 1.0);
    let view = inv_projection * clip;
    let view = Vec3::new(view.x, view.y, view.z) / view.w;

    view.normalize()
}

/// Converts world space position to screen space coordinates.
///
/// # Arguments
///
/// * `world_pos` - Position in world space
/// * `camera_matrices` - Camera view/projection matrices
/// * `viewport_size` - Size of viewport in pixels
///
/// # Returns
///
/// Screen position in pixels, or None if behind camera
fn world_to_screen(
    world_pos: Vec3,
    camera_matrices: &CameraMatrices,
    viewport_size: Vec2,
) -> Option<Vec2> {
    // Transform to clip space
    let clip_pos =
        camera_matrices.view_projection * Vec4::new(world_pos.x, world_pos.y, world_pos.z, 1.0);

    // Check if behind camera
    if clip_pos.w <= 0.0 {
        return None;
    }

    // Perspective divide to get NDC
    let ndc = Vec3::new(
        clip_pos.x / clip_pos.w,
        clip_pos.y / clip_pos.w,
        clip_pos.z / clip_pos.w,
    );

    // Check if outside view frustum
    if ndc.x < -1.0 || ndc.x > 1.0 || ndc.y < -1.0 || ndc.y > 1.0 {
        return None;
    }

    // Convert NDC to screen space
    let screen_x = (ndc.x + 1.0) * 0.5 * viewport_size.x;
    let screen_y = (1.0 - ndc.y) * 0.5 * viewport_size.y;

    Some(Vec2::new(screen_x, screen_y))
}

/// System that synchronizes Selected components with SelectionSystem.
///
/// This system ensures that entities have the Selected component added when
/// they are selected and removed when they are deselected.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::update_selection_system;
/// use praxis_ecs::Schedule;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(update_selection_system);
/// ```
pub fn update_selection_system(
    mut commands: Commands,
    selection: Res<SelectionSystem>,
    selected_query: Query<Entity, With<Selected>>,
    selectable_query: Query<Entity, With<Selectable>>,
) {
    // Remove Selected component from entities no longer selected
    for entity in selected_query.iter() {
        if !selection.is_selected(entity) {
            commands.entity(entity).remove::<Selected>();
        }
    }

    // Add Selected component to newly selected entities
    for entity in selection.selected_entities() {
        // Only add if entity still exists and is selectable
        if selectable_query.get(entity).is_ok() {
            commands.entity(entity).insert(Selected);
        }
    }
}

/// System that handles selection input (keyboard shortcuts).
///
/// Handles:
/// - Ctrl+A: Select all selectable entities
/// - Ctrl+D: Deselect all entities
///
/// Note: Mouse input for raycast and marquee selection should be handled
/// separately in the editor viewport, as it requires viewport context.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_editor::handle_selection_input_system;
/// use praxis_ecs::Schedule;
///
/// let mut schedule = Schedule::default();
/// schedule.add_systems(handle_selection_input_system);
/// ```
pub fn handle_selection_input_system(
    mut selection: ResMut<SelectionSystem>,
    input: Res<InputState>,
    selectable_query: Query<Entity, With<Selectable>>,
) {
    if !selection.is_input_enabled() {
        return;
    }

    let ctrl =
        input.is_key_pressed(KeyCode::ControlLeft) || input.is_key_pressed(KeyCode::ControlRight);

    // Ctrl+A: Select all
    if ctrl && input.is_key_just_pressed(KeyCode::KeyA) {
        let all_entities: Vec<Entity> = selectable_query.iter().collect();
        selection.select_entities(all_entities, SelectionMode::Replace);
    }

    // Ctrl+D: Deselect all
    if ctrl && input.is_key_just_pressed(KeyCode::KeyD) {
        selection.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_system_creation() {
        let selection = SelectionSystem::new();
        assert!(selection.is_empty());
        assert_eq!(selection.selected_count(), 0);
    }

    #[test]
    fn test_select_single_entity_replace() {
        let mut selection = SelectionSystem::new();
        let entity = Entity::from_raw(1);

        selection.select_entity(entity, SelectionMode::Replace);
        assert!(selection.is_selected(entity));
        assert_eq!(selection.selected_count(), 1);
    }

    #[test]
    fn test_select_multiple_entities() {
        let mut selection = SelectionSystem::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        selection.select_entity(entity1, SelectionMode::Add);
        selection.select_entity(entity2, SelectionMode::Add);

        assert!(selection.is_selected(entity1));
        assert!(selection.is_selected(entity2));
        assert_eq!(selection.selected_count(), 2);
    }

    #[test]
    fn test_selection_mode_replace() {
        let mut selection = SelectionSystem::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        selection.select_entity(entity1, SelectionMode::Add);
        selection.select_entity(entity2, SelectionMode::Replace);

        assert!(!selection.is_selected(entity1));
        assert!(selection.is_selected(entity2));
        assert_eq!(selection.selected_count(), 1);
    }

    #[test]
    fn test_selection_mode_remove() {
        let mut selection = SelectionSystem::new();
        let entity = Entity::from_raw(1);

        selection.select_entity(entity, SelectionMode::Add);
        assert!(selection.is_selected(entity));

        selection.select_entity(entity, SelectionMode::Remove);
        assert!(!selection.is_selected(entity));
        assert!(selection.is_empty());
    }

    #[test]
    fn test_selection_mode_toggle() {
        let mut selection = SelectionSystem::new();
        let entity = Entity::from_raw(1);

        selection.select_entity(entity, SelectionMode::Toggle);
        assert!(selection.is_selected(entity));

        selection.select_entity(entity, SelectionMode::Toggle);
        assert!(!selection.is_selected(entity));
    }

    #[test]
    fn test_clear_selection() {
        let mut selection = SelectionSystem::new();
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        selection.select_entity(entity1, SelectionMode::Add);
        selection.select_entity(entity2, SelectionMode::Add);
        assert_eq!(selection.selected_count(), 2);

        selection.clear();
        assert!(selection.is_empty());
    }

    #[test]
    fn test_selection_events() {
        let mut selection = SelectionSystem::new();
        let entity = Entity::from_raw(1);

        selection.select_entity(entity, SelectionMode::Replace);

        let events: Vec<SelectionEvent> = selection.events().iter().cloned().collect();
        assert!(!events.is_empty());
        assert!(matches!(events[0], SelectionEvent::Selected(_)));
    }

    #[test]
    fn test_drain_events() {
        let mut selection = SelectionSystem::new();
        let entity = Entity::from_raw(1);

        selection.select_entity(entity, SelectionMode::Replace);
        assert!(!selection.events().is_empty());

        let events = selection.drain_events();
        assert!(!events.is_empty());
        assert!(selection.events().is_empty());
    }

    #[test]
    fn test_marquee_selection() {
        let mut selection = SelectionSystem::new();

        selection.start_marquee(Vec2::new(10.0, 10.0));
        assert!(selection.is_marquee_active());

        selection.update_marquee(Vec2::new(50.0, 50.0));
        assert!(selection.is_marquee_active());

        let rect = selection.end_marquee();
        assert!(rect.is_some());
        assert!(!selection.is_marquee_active());
    }

    #[test]
    fn test_marquee_cancel() {
        let mut selection = SelectionSystem::new();

        selection.start_marquee(Vec2::new(10.0, 10.0));
        assert!(selection.is_marquee_active());

        selection.cancel_marquee();
        assert!(!selection.is_marquee_active());
    }

    #[test]
    fn test_input_enabled() {
        let mut selection = SelectionSystem::new();
        assert!(selection.is_input_enabled());

        selection.set_input_enabled(false);
        assert!(!selection.is_input_enabled());

        selection.set_input_enabled(true);
        assert!(selection.is_input_enabled());
    }

    #[test]
    fn test_select_multiple_entities_batch() {
        let mut selection = SelectionSystem::new();
        let entities = vec![
            Entity::from_raw(1),
            Entity::from_raw(2),
            Entity::from_raw(3),
        ];

        selection.select_entities(entities.clone(), SelectionMode::Replace);
        assert_eq!(selection.selected_count(), 3);

        for entity in entities {
            assert!(selection.is_selected(entity));
        }
    }

    #[test]
    fn test_deselect_entity() {
        let mut selection = SelectionSystem::new();
        let entity = Entity::from_raw(1);

        selection.select_entity(entity, SelectionMode::Add);
        assert!(selection.is_selected(entity));

        selection.deselect_entity(entity);
        assert!(!selection.is_selected(entity));
    }
}
