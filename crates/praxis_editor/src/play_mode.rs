//! Play mode system for the Praxis editor.
//!
//! This module provides a complete play mode system that allows testing game functionality
//! within the editor with proper state isolation and restoration.
//!
//! # Features
//!
//! - **Edit/Play State Machine**: Transitions between Edit and Play modes with proper state management
//! - **Scene Snapshot/Restore**: Automatically captures scene state before entering play mode and
//!   restores it when exiting
//! - **Runtime ECS Isolation**: Play mode changes are isolated and don't affect the original scene
//! - **Input Routing Toggle**: Input can be routed to play mode or kept in edit mode
//! - **Visual Indicators**: Viewport border color changes and toolbar button state reflects current mode
//!
//! # Usage
//!
//! ```rust,no_run
//! use praxis_editor::PlayModeSystem;
//! use praxis_ecs::World;
//!
//! let mut world = World::new();
//! let mut play_mode = PlayModeSystem::new();
//!
//! // Enter play mode
//! play_mode.enter_play_mode(&mut world).unwrap();
//!
//! // While in play mode, the scene can be modified freely
//! // ...
//!
//! // Exit play mode and restore original scene
//! play_mode.exit_play_mode(&mut world).unwrap();
//! ```
//!
//! # State Machine
//!
//! The play mode system follows a simple state machine:
//!
//! ```text
//! Edit Mode --> (enter_play_mode) --> Play Mode
//!     ^                                    |
//!     |                                    |
//!     +--------- (exit_play_mode) <--------+
//! ```
//!
//! # Scene Snapshotting
//!
//! When entering play mode, the system:
//! 1. Serializes all scene entities to a `SceneSnapshot`
//! 2. Stores component data for all entities
//! 3. Preserves hierarchy relationships
//!
//! When exiting play mode, the system:
//! 1. Clears all runtime entities
//! 2. Restores entities from the snapshot
//! 3. Rebuilds the scene hierarchy
//!
//! # Visual Feedback
//!
//! - **Edit Mode**: Viewport has default border, Play button enabled
//! - **Play Mode**: Viewport has green border, Pause/Stop buttons enabled

use crate::EditorMode;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::Without;
use bevy_ecs::world::World as BevyWorld;
use praxis_ecs::{
    Active, Camera, Children, DirectionalLight, MaterialHandle, MeshHandle, Name, NoSave, Parent,
    PointLight, Transform, Visibility, World,
};
use praxis_scene::{SceneDefinition, SceneLoader, SceneManager};
use praxis_utils::{error, info, warn, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Play mode system managing Edit/Play state transitions and scene snapshots.
///
/// This system handles the complete lifecycle of play mode:
/// - Taking snapshots of the scene before play
/// - Restoring the scene when exiting play
/// - Managing input routing
/// - Providing visual state indicators
pub struct PlayModeSystem {
    /// Current play mode state
    state: PlayModeState,
    /// Scene snapshot taken when entering play mode
    snapshot: Option<SceneSnapshot>,
    /// Scene loader for serialization/deserialization
    #[allow(dead_code)]
    scene_loader: SceneLoader,
    /// Scene manager for spawning/despawning
    scene_manager: SceneManager,
    /// Whether to route input to play mode systems
    route_input_to_play: bool,
}

/// State of the play mode system
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayModeState {
    /// Editor is in edit mode
    Edit,
    /// Editor is in play mode
    Playing,
    /// Editor is paused (currently treated as edit mode)
    Paused,
}

impl PlayModeState {
    /// Returns true if in any play state (playing or paused)
    pub const fn is_play_mode(&self) -> bool {
        matches!(self, Self::Playing | Self::Paused)
    }

    /// Returns true if in edit mode
    pub const fn is_edit_mode(&self) -> bool {
        matches!(self, Self::Edit)
    }

    /// Converts to EditorMode
    pub const fn to_editor_mode(&self) -> EditorMode {
        match self {
            Self::Edit | Self::Paused => EditorMode::Edit,
            Self::Playing => EditorMode::Play,
        }
    }
}

/// Snapshot of the scene state for restoration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneSnapshot {
    /// Serialized scene definition
    scene_definition: SceneDefinition,
    /// Additional metadata
    metadata: SnapshotMetadata,
}

/// Metadata about the snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    /// Timestamp when snapshot was taken
    timestamp: u64,
    /// Number of entities in the snapshot
    entity_count: usize,
}

impl PlayModeSystem {
    /// Creates a new play mode system
    pub fn new() -> Self {
        Self {
            state: PlayModeState::Edit,
            snapshot: None,
            scene_loader: SceneLoader::new(),
            scene_manager: SceneManager::new(),
            route_input_to_play: true,
        }
    }

    /// Gets the current play mode state
    pub const fn state(&self) -> PlayModeState {
        self.state
    }

    /// Gets the current editor mode
    pub const fn editor_mode(&self) -> EditorMode {
        self.state.to_editor_mode()
    }

    /// Returns true if currently in play mode
    pub const fn is_playing(&self) -> bool {
        self.state.is_play_mode()
    }

    /// Returns true if currently in edit mode
    pub const fn is_editing(&self) -> bool {
        self.state.is_edit_mode()
    }

    /// Returns true if input should be routed to play mode systems
    pub const fn should_route_input_to_play(&self) -> bool {
        self.route_input_to_play && self.state.is_play_mode()
    }

    /// Sets whether to route input to play mode systems
    pub fn set_route_input_to_play(&mut self, route: bool) {
        self.route_input_to_play = route;
    }

    /// Gets the viewport border color based on current state
    pub fn viewport_border_color(&self) -> [f32; 3] {
        match self.state {
            PlayModeState::Edit => [0.3, 0.3, 0.35],   // Dark gray
            PlayModeState::Playing => [0.2, 0.8, 0.3], // Green
            PlayModeState::Paused => [0.9, 0.7, 0.2],  // Orange/Yellow
        }
    }

    /// Gets the viewport border color as egui Color32
    pub fn viewport_border_color_egui(&self) -> egui::Color32 {
        let [r, g, b] = self.viewport_border_color();
        egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
    }

    /// Enters play mode by taking a snapshot and transitioning state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Already in play mode
    /// - Scene snapshot fails
    pub fn enter_play_mode(&mut self, world: &mut World) -> Result<()> {
        if self.state.is_play_mode() {
            warn!("Already in play mode, ignoring enter_play_mode");
            return Ok(());
        }

        info!("Entering play mode");

        // Take snapshot of current scene state
        let snapshot = self.take_snapshot(world)?;
        self.snapshot = Some(snapshot);

        // Transition to playing state
        self.state = PlayModeState::Playing;

        info!("Play mode entered successfully");
        Ok(())
    }

    /// Exits play mode and restores the snapshot
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Not in play mode
    /// - Scene restoration fails
    pub fn exit_play_mode(&mut self, world: &mut World) -> Result<()> {
        if !self.state.is_play_mode() {
            warn!("Not in play mode, ignoring exit_play_mode");
            return Ok(());
        }

        info!("Exiting play mode");

        // Restore snapshot
        if let Some(snapshot) = self.snapshot.take() {
            self.restore_snapshot(world, snapshot)?;
        } else {
            error!("No snapshot available to restore");
        }

        // Transition to edit state
        self.state = PlayModeState::Edit;

        info!("Play mode exited successfully");
        Ok(())
    }

    /// Pauses play mode (currently transitions to edit mode)
    pub fn pause_play_mode(&mut self) {
        if self.state == PlayModeState::Playing {
            info!("Pausing play mode");
            self.state = PlayModeState::Paused;
        }
    }

    /// Resumes play mode from paused state
    pub fn resume_play_mode(&mut self) {
        if self.state == PlayModeState::Paused {
            info!("Resuming play mode");
            self.state = PlayModeState::Playing;
        }
    }

    /// Takes a snapshot of the current scene state
    fn take_snapshot(&self, world: &mut World) -> Result<SceneSnapshot> {
        info!("Taking scene snapshot");

        let scene_definition = self.serialize_world_to_scene(world)?;
        let entity_count = scene_definition.total_entity_count();

        info!("Snapshot captured {} entities", entity_count);

        Ok(SceneSnapshot {
            scene_definition,
            metadata: SnapshotMetadata {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                entity_count,
            },
        })
    }

    /// Restores the scene from a snapshot
    fn restore_snapshot(&mut self, world: &mut World, snapshot: SceneSnapshot) -> Result<()> {
        info!(
            "Restoring scene snapshot with {} entities",
            snapshot.metadata.entity_count
        );

        // Clear all runtime entities (those without NoSave marker)
        self.clear_runtime_entities(world);

        // Restore entities from snapshot
        self.scene_manager
            .spawn_scene(world, &snapshot.scene_definition)?;

        info!("Scene snapshot restored successfully");
        Ok(())
    }

    /// Serializes the ECS world to a scene definition
    fn serialize_world_to_scene(&self, world: &mut World) -> Result<SceneDefinition> {
        use praxis_scene::EntityDefinition;

        let mut scene = SceneDefinition::new("PlayModeSnapshot");

        // Query all entities that should be saved (without NoSave component)
        let bevy_world = world.inner_mut();

        // Collect root entities (entities without Parent component)
        let mut root_entities = Vec::new();
        let mut entity_data: HashMap<Entity, EntityDefinition> = HashMap::new();

        // First pass: collect all entities and their data
        let mut query = bevy_world.query_filtered::<Entity, Without<NoSave>>();
        for entity in query.iter(bevy_world) {
            let def = self.serialize_entity(bevy_world, entity);
            entity_data.insert(entity, def);
        }

        // Second pass: identify root entities and build hierarchy
        for (entity, def) in &entity_data {
            // Check if entity has a parent
            if let Some(parent_comp) = bevy_world.get::<Parent>(*entity) {
                // This is a child entity - it will be added to its parent's children
                let parent_entity = parent_comp.0;
                if entity_data.contains_key(&parent_entity) {
                    // We'll handle children in the third pass
                    continue;
                }
            }

            // This is a root entity (no parent or parent not in saveable entities)
            root_entities.push((*entity, def.clone()));
        }

        // Third pass: add children to their parents
        let entity_data_clone = entity_data.clone();
        for (entity, def) in entity_data.iter_mut() {
            if let Some(children_comp) = bevy_world.get::<Children>(*entity) {
                for &child_entity in children_comp.iter() {
                    if let Some(child_def) = entity_data_clone.get(&child_entity) {
                        def.children.push(child_def.clone());
                    }
                }
            }
        }

        // Add root entities to scene
        for (_entity, def) in root_entities {
            scene.add_entity(def);
        }

        Ok(scene)
    }

    /// Serializes a single entity to an entity definition
    fn serialize_entity(
        &self,
        world: &BevyWorld,
        entity: Entity,
    ) -> praxis_scene::EntityDefinition {
        use praxis_scene::{
            CameraDef, CameraType, DirectionalLightDef, EntityDefinition, PointLightDef,
            TransformDef,
        };

        let mut def = EntityDefinition::new();

        // Name
        if let Some(name) = world.get::<Name>(entity) {
            def.name = Some(name.as_str().to_string());
        }

        // Transform
        if let Some(transform) = world.get::<Transform>(entity) {
            def.transform = Some(TransformDef {
                translation: (
                    transform.translation.x,
                    transform.translation.y,
                    transform.translation.z,
                ),
                rotation: (
                    transform.rotation.x,
                    transform.rotation.y,
                    transform.rotation.z,
                    transform.rotation.w,
                ),
                scale: (transform.scale.x, transform.scale.y, transform.scale.z),
            });
        }

        // Mesh
        if let Some(mesh_handle) = world.get::<MeshHandle>(entity) {
            def.mesh = Some(mesh_handle.id().to_string());
        }

        // Material (texture)
        if let Some(material_handle) = world.get::<MaterialHandle>(entity) {
            def.texture = Some(material_handle.id().to_string());
        }

        // Camera
        if world.get::<Camera>(entity).is_some() {
            // Try perspective first
            if let Some(persp) = world.get::<praxis_ecs::PerspectiveProjection>(entity) {
                def.camera = Some(CameraDef {
                    camera_type: CameraType::Perspective,
                    fov: Some(persp.fov),
                    aspect_ratio: Some(persp.aspect_ratio),
                    left: None,
                    right: None,
                    bottom: None,
                    top: None,
                    near: persp.near,
                    far: persp.far,
                    is_active: world.get::<Camera>(entity).is_none_or(|c| c.is_active),
                    priority: world.get::<Camera>(entity).map_or(0, |c| c.priority),
                });
            }
            // Try orthographic
            else if let Some(ortho) = world.get::<praxis_ecs::OrthographicProjection>(entity) {
                def.camera = Some(CameraDef {
                    camera_type: CameraType::Orthographic,
                    fov: None,
                    aspect_ratio: None,
                    left: Some(ortho.left),
                    right: Some(ortho.right),
                    bottom: Some(ortho.bottom),
                    top: Some(ortho.top),
                    near: ortho.near,
                    far: ortho.far,
                    is_active: world.get::<Camera>(entity).is_none_or(|c| c.is_active),
                    priority: world.get::<Camera>(entity).map_or(0, |c| c.priority),
                });
            }
        }

        // Directional Light
        if let Some(dir_light) = world.get::<DirectionalLight>(entity) {
            def.directional_light = Some(DirectionalLightDef {
                direction: (
                    dir_light.direction.x,
                    dir_light.direction.y,
                    dir_light.direction.z,
                ),
                color: (dir_light.color.x, dir_light.color.y, dir_light.color.z),
                intensity: dir_light.intensity,
            });
        }

        // Point Light
        if let Some(point_light) = world.get::<PointLight>(entity) {
            def.point_light = Some(PointLightDef {
                color: (
                    point_light.color.x,
                    point_light.color.y,
                    point_light.color.z,
                ),
                intensity: point_light.intensity,
                range: point_light.range,
            });
        }

        // Visibility
        if let Some(visibility) = world.get::<Visibility>(entity) {
            def.visible = Some(visibility.is_visible());
        }

        // Active (marker component - just check if it exists)
        if world.get::<Active>(entity).is_some() {
            def.active = Some(true);
        }

        def
    }

    /// Clears all runtime entities (those without NoSave marker)
    fn clear_runtime_entities(&mut self, world: &mut World) {
        info!("Clearing runtime entities");

        let bevy_world = world.inner_mut();
        let mut entities_to_remove = Vec::new();

        // Collect entities without NoSave
        let mut query = bevy_world.query_filtered::<Entity, Without<NoSave>>();
        for entity in query.iter(bevy_world) {
            entities_to_remove.push(entity);
        }

        // Remove collected entities
        for entity in entities_to_remove {
            if bevy_world.despawn(entity) {
                // Entity despawned successfully
            }
        }

        info!("Runtime entities cleared");
    }
}

impl Default for PlayModeSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_ecs::{GlobalTransform, World};

    #[test]
    fn test_play_mode_system_creation() {
        let system = PlayModeSystem::new();
        assert_eq!(system.state(), PlayModeState::Edit);
        assert!(system.is_editing());
        assert!(!system.is_playing());
    }

    #[test]
    fn test_editor_mode_conversion() {
        assert_eq!(PlayModeState::Edit.to_editor_mode(), EditorMode::Edit);
        assert_eq!(PlayModeState::Playing.to_editor_mode(), EditorMode::Play);
        assert_eq!(PlayModeState::Paused.to_editor_mode(), EditorMode::Edit);
    }

    #[test]
    fn test_viewport_border_colors() {
        let mut system = PlayModeSystem::new();

        // Edit mode - dark gray
        let color = system.viewport_border_color();
        assert_eq!(color, [0.3, 0.3, 0.35]);

        // Play mode - green
        system.state = PlayModeState::Playing;
        let color = system.viewport_border_color();
        assert_eq!(color, [0.2, 0.8, 0.3]);

        // Paused mode - orange/yellow
        system.state = PlayModeState::Paused;
        let color = system.viewport_border_color();
        assert_eq!(color, [0.9, 0.7, 0.2]);
    }

    #[test]
    fn test_input_routing() {
        let mut system = PlayModeSystem::new();

        // Edit mode - no routing
        assert!(!system.should_route_input_to_play());

        // Enter play mode - routing enabled
        system.state = PlayModeState::Playing;
        assert!(system.should_route_input_to_play());

        // Disable routing
        system.set_route_input_to_play(false);
        assert!(!system.should_route_input_to_play());

        // Re-enable routing
        system.set_route_input_to_play(true);
        assert!(system.should_route_input_to_play());
    }

    #[test]
    fn test_enter_play_mode() {
        let mut world = World::new();
        let mut system = PlayModeSystem::new();

        // Add some test entities
        world.spawn((
            Name::new("TestEntity"),
            Transform::from_xyz(1.0, 2.0, 3.0),
            GlobalTransform::default(),
        ));

        // Enter play mode
        let result = system.enter_play_mode(&mut world);
        assert!(result.is_ok());
        assert_eq!(system.state(), PlayModeState::Playing);
        assert!(system.snapshot.is_some());
    }

    #[test]
    fn test_exit_play_mode() {
        let mut world = World::new();
        let mut system = PlayModeSystem::new();

        // Add test entity
        let _original_entity = world.spawn((
            Name::new("OriginalEntity"),
            Transform::from_xyz(5.0, 10.0, 15.0),
            GlobalTransform::default(),
        ));

        // Enter play mode
        system.enter_play_mode(&mut world).unwrap();

        // Modify the scene in play mode
        world.spawn((
            Name::new("RuntimeEntity"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        // Exit play mode
        let result = system.exit_play_mode(&mut world);
        assert!(result.is_ok());
        assert_eq!(system.state(), PlayModeState::Edit);
        assert!(system.snapshot.is_none());
    }

    #[test]
    fn test_pause_and_resume() {
        let mut system = PlayModeSystem::new();

        // Start in edit mode
        assert_eq!(system.state(), PlayModeState::Edit);

        // Enter playing state
        system.state = PlayModeState::Playing;
        assert_eq!(system.state(), PlayModeState::Playing);

        // Pause
        system.pause_play_mode();
        assert_eq!(system.state(), PlayModeState::Paused);

        // Resume
        system.resume_play_mode();
        assert_eq!(system.state(), PlayModeState::Playing);
    }

    #[test]
    fn test_snapshot_metadata() {
        let mut world = World::new();
        let system = PlayModeSystem::new();

        // Add multiple entities
        for i in 0..5 {
            world.spawn((
                Name::new(format!("Entity{}", i)),
                Transform::default(),
                GlobalTransform::default(),
            ));
        }

        // Take snapshot
        let snapshot = system.take_snapshot(&mut world).unwrap();
        assert_eq!(snapshot.metadata.entity_count, 5);
        assert!(snapshot.metadata.timestamp > 0);
    }

    #[test]
    fn test_no_save_entities_excluded() {
        let mut world = World::new();
        let system = PlayModeSystem::new();

        // Add entity with NoSave marker
        world.spawn((
            Name::new("EditorOnlyEntity"),
            Transform::default(),
            GlobalTransform::default(),
            NoSave,
        ));

        // Add normal entity
        world.spawn((
            Name::new("NormalEntity"),
            Transform::default(),
            GlobalTransform::default(),
        ));

        // Take snapshot
        let snapshot = system.take_snapshot(&mut world).unwrap();

        // Only the normal entity should be in the snapshot
        assert_eq!(snapshot.metadata.entity_count, 1);
    }
}
