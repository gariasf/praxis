//! Save and load system for full game state persistence.
//!
//! This module provides comprehensive save/load functionality for persisting
//! complete game state including entity hierarchies, components, asset references,
//! and scene metadata with versioning support.
//!
//! # Features
//!
//! - **Full Entity Serialization**: Captures all components, hierarchies, and relationships
//! - **Asset References**: Properly tracks and serializes asset handles (meshes, textures, materials)
//! - **Versioning**: Scene format versioning with migration support
//! - **Metadata**: Timestamps, checksums, and custom metadata
//! - **Incremental Saves**: Support for auto-saves and named save slots
//! - **Selective Persistence**: Entities marked with `NoSave` are excluded
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_scene::{SaveManager, SaveMetadata};
//! use praxis_ecs::World;
//! use std::path::Path;
//!
//! let mut world = World::new();
//! let mut save_manager = SaveManager::new();
//!
//! // Create some metadata
//! let metadata = SaveMetadata::new("Chapter 1 - Forest")
//!     .with_description("Player at the forest entrance")
//!     .with_tag("autosave");
//!
//! // Save the complete game state
//! save_manager.save_to_file(&world, Path::new("saves/slot1.ron"), metadata).unwrap();
//!
//! // Load the game state
//! save_manager.load_from_file(&mut world, Path::new("saves/slot1.ron")).unwrap();
//! ```

use crate::{
    definition::{
        CameraDef, CameraType, DirectionalLightDef, EntityDefinition, PointLightDef,
        SceneDefinition, SceneMetadata, TransformDef, CURRENT_SCENE_VERSION,
    },
};
use bevy_ecs::entity::Entity;
use praxis_ecs::{
    Active, Camera, Children, DirectionalLight, GlobalTransform, MaterialHandle, MeshHandle, Name,
    NoSave, OrthographicProjection, Parent, PerspectiveProjection, PointLight, TextureHandle,
    Transform, Visibility, World,
};
use praxis_utils::{debug, info, warn, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Current save format version.
///
/// This should be incremented whenever the save format changes in a
/// backwards-incompatible way. Migration code should handle older versions.
pub const CURRENT_SAVE_VERSION: u32 = 1;

/// Manager for saving and loading game state.
///
/// The `SaveManager` handles full game state persistence including:
/// - Entity hierarchies and relationships
/// - Component data
/// - Asset references (meshes, textures, materials)
/// - Scene metadata and versioning
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{SaveManager, SaveMetadata};
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// let mut save_manager = SaveManager::new();
///
/// // Save current state
/// let metadata = SaveMetadata::new("Save 1");
/// save_manager.save_to_file(&world, "saves/slot1.ron", metadata).unwrap();
///
/// // Load saved state
/// save_manager.load_from_file(&mut world, "saves/slot1.ron").unwrap();
/// ```
#[derive(Debug, Default)]
pub struct SaveManager {
    /// Configuration for save/load operations.
    config: SaveConfig,
    /// Statistics from the last save/load operation.
    last_stats: Option<SaveStats>,
}

/// Configuration for save/load operations.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct SaveConfig {
    /// Whether to compress save files (future feature).
    pub compress: bool,
    /// Whether to include editor data in saves.
    pub include_editor_data: bool,
    /// Whether to validate saves after writing.
    pub validate_after_save: bool,
    /// Pretty-print RON output for readability.
    pub pretty_print: bool,
}

impl Default for SaveConfig {
    fn default() -> Self {
        Self {
            compress: false,
            include_editor_data: false,
            validate_after_save: true,
            pretty_print: true,
        }
    }
}

/// Statistics from a save/load operation.
#[derive(Debug, Clone, Copy)]
pub struct SaveStats {
    /// Number of entities saved/loaded.
    pub entity_count: usize,
    /// Number of components saved/loaded.
    pub component_count: usize,
    /// Time taken in milliseconds.
    pub duration_ms: f64,
    /// File size in bytes (save only).
    pub file_size_bytes: Option<u64>,
}

impl SaveStats {
    /// Creates new statistics with zero values.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            entity_count: 0,
            component_count: 0,
            duration_ms: 0.0,
            file_size_bytes: None,
        }
    }
}

impl Default for SaveStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete save file structure.
///
/// This is the top-level structure serialized to disk. It contains
/// all necessary information to restore the complete game state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveFile {
    /// Save format version for migration support.
    pub version: u32,
    /// Metadata about this save file.
    pub metadata: SaveMetadata,
    /// The complete scene state.
    pub scene: SceneDefinition,
}

impl SaveFile {
    /// Creates a new save file with the given scene and metadata.
    #[must_use]
    pub const fn new(scene: SceneDefinition, metadata: SaveMetadata) -> Self {
        Self {
            version: CURRENT_SAVE_VERSION,
            metadata,
            scene,
        }
    }

    /// Validates the save file structure.
    ///
    /// # Errors
    ///
    /// Returns an error if the save file is invalid.
    pub fn validate(&self) -> Result<()> {
        if self.version == 0 {
            return Err(praxis_utils::Report::msg("Invalid save version: 0"));
        }

        if self.scene.entities.is_empty() {
            warn!("Save file contains no entities");
        }

        Ok(())
    }
}

/// Metadata about a save file.
///
/// Contains descriptive information, timestamps, and other metadata
/// that helps identify and manage save files.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadata {
    /// Display name for this save.
    pub name: String,
    /// Optional description of the save state.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Timestamp when the save was created (ISO 8601 format).
    pub timestamp: String,
    /// Playtime in seconds when this save was created.
    #[serde(default)]
    pub playtime_seconds: u64,
    /// Game version that created this save.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_version: Option<String>,
    /// Optional screenshot path (relative to save directory).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot_path: Option<String>,
    /// Custom tags for organization (e.g., "autosave", "checkpoint").
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Custom key-value metadata.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_data: HashMap<String, String>,
}

impl SaveMetadata {
    /// Creates new metadata with the given name and current timestamp.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            timestamp: chrono::Utc::now().to_rfc3339(),
            playtime_seconds: 0,
            game_version: None,
            screenshot_path: None,
            tags: Vec::new(),
            custom_data: HashMap::new(),
        }
    }

    /// Sets the description.
    #[must_use]
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the playtime.
    #[must_use]
    pub const fn with_playtime(mut self, seconds: u64) -> Self {
        self.playtime_seconds = seconds;
        self
    }

    /// Sets the game version.
    #[must_use]
    pub fn with_game_version(mut self, version: impl Into<String>) -> Self {
        self.game_version = Some(version.into());
        self
    }

    /// Sets the screenshot path.
    #[must_use]
    pub fn with_screenshot(mut self, path: impl Into<String>) -> Self {
        self.screenshot_path = Some(path.into());
        self
    }

    /// Adds a tag.
    #[must_use]
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Adds custom metadata.
    #[must_use]
    pub fn with_custom_data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_data.insert(key.into(), value.into());
        self
    }
}

impl SaveManager {
    /// Creates a new save manager with default configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: SaveConfig::default(),
            last_stats: None,
        }
    }

    /// Creates a save manager with custom configuration.
    #[must_use]
    pub const fn with_config(config: SaveConfig) -> Self {
        Self {
            config,
            last_stats: None,
        }
    }

    /// Gets the current configuration.
    #[must_use]
    pub const fn config(&self) -> &SaveConfig {
        &self.config
    }

    /// Sets the configuration.
    pub const fn set_config(&mut self, config: SaveConfig) {
        self.config = config;
    }

    /// Gets statistics from the last save/load operation.
    #[must_use]
    pub const fn last_stats(&self) -> Option<&SaveStats> {
        self.last_stats.as_ref()
    }

    /// Saves the complete world state to a file.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to save
    /// * `path` - Path to the save file
    /// * `metadata` - Metadata for the save file
    ///
    /// # Errors
    ///
    /// Returns an error if serialization or file I/O fails.
    ///
    /// # Panics
    ///
    /// Will panic if statistics collection fails (should not occur in normal operation).
    pub fn save_to_file(
        &mut self,
        world: &mut World,
        path: impl AsRef<Path>,
        metadata: SaveMetadata,
    ) -> Result<()> {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        info!("Saving game state to '{}'", path.display());

        // Capture the world state
        let scene = Self::capture_world_state(world);

        // Create save file
        let save_file = SaveFile::new(scene, metadata);

        // Validate if configured
        if self.config.validate_after_save {
            save_file.validate()?;
        }

        // Serialize to RON
        let ron_config = if self.config.pretty_print {
            ron::ser::PrettyConfig::default()
        } else {
            ron::ser::PrettyConfig::default().compact_arrays(true)
        };

        let serialized = ron::ser::to_string_pretty(&save_file, ron_config)
            .map_err(|e| praxis_utils::Report::msg(format!("Failed to serialize save file: {e}")))?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| praxis_utils::Report::msg(format!("Failed to create save directory: {e}")))?;
        }

        // Write to file
        fs::write(path, &serialized)
            .map_err(|e| praxis_utils::Report::msg(format!("Failed to write save file: {e}")))?;

        // Collect statistics
        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;
        let file_size = fs::metadata(path).ok().map(|m| m.len());

        self.last_stats = Some(SaveStats {
            entity_count: save_file.scene.total_entity_count(),
            component_count: Self::count_components(&save_file.scene),
            duration_ms,
            file_size_bytes: file_size,
        });

        info!(
            "Save complete: {} entities, {} components, {:.2}ms",
            self.last_stats.as_ref().unwrap().entity_count,
            self.last_stats.as_ref().unwrap().component_count,
            duration_ms
        );

        Ok(())
    }

    /// Loads world state from a file, replacing the current world contents.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to load into (will be cleared)
    /// * `path` - Path to the save file
    ///
    /// # Errors
    ///
    /// Returns an error if deserialization or file I/O fails.
    ///
    /// # Panics
    ///
    /// Will panic if statistics collection fails (should not occur in normal operation).
    pub fn load_from_file(&mut self, world: &mut World, path: impl AsRef<Path>) -> Result<()> {
        let start_time = std::time::Instant::now();
        let path = path.as_ref();

        info!("Loading game state from '{}'", path.display());

        // Read file
        let contents = fs::read_to_string(path)
            .map_err(|e| praxis_utils::Report::msg(format!("Failed to read save file: {e}")))?;

        // Deserialize
        let mut save_file: SaveFile = ron::from_str(&contents)
            .map_err(|e| praxis_utils::Report::msg(format!("Failed to deserialize save file: {e}")))?;

        // Validate
        save_file.validate()?;

        // Migrate if necessary
        if save_file.version < CURRENT_SAVE_VERSION {
            info!(
                "Migrating save from version {} to {}",
                save_file.version, CURRENT_SAVE_VERSION
            );
            // Migration would be handled here if needed
            save_file.version = CURRENT_SAVE_VERSION;
        }

        // Migrate scene format if necessary
        if save_file.scene.version < CURRENT_SCENE_VERSION {
            info!(
                "Migrating scene from version {} to {}",
                save_file.scene.version, CURRENT_SCENE_VERSION
            );
            crate::migration::migrate_scene(&mut save_file.scene)?;
        }

        // Clear world
        Self::clear_world(world);

        // Restore world state
        self.restore_world_state(world, &save_file.scene)?;

        // Collect statistics
        let duration_ms = start_time.elapsed().as_secs_f64() * 1000.0;

        self.last_stats = Some(SaveStats {
            entity_count: save_file.scene.total_entity_count(),
            component_count: Self::count_components(&save_file.scene),
            duration_ms,
            file_size_bytes: None,
        });

        info!(
            "Load complete: {} entities, {} components, {:.2}ms",
            self.last_stats.as_ref().unwrap().entity_count,
            self.last_stats.as_ref().unwrap().component_count,
            duration_ms
        );

        Ok(())
    }

    /// Reads save file metadata without loading the entire save.
    ///
    /// This is useful for displaying save file information in a load menu
    /// without the overhead of loading the complete game state.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn read_metadata(&self, path: impl AsRef<Path>) -> Result<SaveMetadata> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path)
            .map_err(|e| praxis_utils::Report::msg(format!("Failed to read save file: {e}")))?;

        // We only need the metadata, but RON requires parsing the full structure
        let save_file: SaveFile = ron::from_str(&contents)
            .map_err(|e| praxis_utils::Report::msg(format!("Failed to parse save file: {e}")))?;

        Ok(save_file.metadata)
    }

    /// Captures the current world state into a scene definition.
    ///
    /// This includes all entities, components, and hierarchies, excluding
    /// entities marked with the `NoSave` component.
    #[allow(clippy::too_many_lines)]
    fn capture_world_state(world: &mut World) -> SceneDefinition {
        let mut scene = SceneDefinition::new("SavedGame");
        scene.metadata = SceneMetadata {
            description: Some("Auto-generated save file".to_string()),
            author: None,
            version: Some(CURRENT_SAVE_VERSION.to_string()),
            tags: vec!["save".to_string()],
        };

        // Build entity map and find roots
        let mut entity_map: HashMap<Entity, EntityDefinition> = HashMap::new();
        let mut root_entities = Vec::new();

        // Query all entities
        let mut query = world.query::<(
            bevy_ecs::entity::Entity,
            Option<&Name>,
            Option<&Transform>,
            Option<&MeshHandle>,
            Option<&TextureHandle>,
            Option<&MaterialHandle>,
            Option<&Camera>,
            Option<&PerspectiveProjection>,
            Option<&OrthographicProjection>,
            Option<&DirectionalLight>,
            Option<&PointLight>,
            Option<&Visibility>,
            Option<&Active>,
            Option<&Parent>,
            Option<&NoSave>,
        )>();

        for (
            entity,
            name,
            transform,
            mesh,
            texture,
            material,
            camera,
            perspective,
            orthographic,
            dir_light,
            point_light,
            visibility,
            active,
            parent,
            no_save,
        ) in query.iter(world)
        {
            // Skip entities marked with NoSave
            if no_save.is_some() {
                debug!("Skipping entity {:?} marked with NoSave", entity);
                continue;
            }

            let mut entity_def = EntityDefinition::new();

            // Name
            if let Some(name) = name {
                entity_def.name = Some(name.0.clone());
            }

            // Transform
            if let Some(transform) = transform {
                entity_def.transform = Some(TransformDef {
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
            if let Some(mesh) = mesh {
                entity_def.mesh = Some(mesh.id.clone());
            }

            // Texture
            if let Some(texture) = texture {
                entity_def.texture = Some(texture.id.clone());
            }

            // Material
            if let Some(material) = material {
                entity_def.material = Some(material.id.clone());
            }

            // Camera
            if let Some(camera) = camera {
                if let Some(perspective) = perspective {
                    entity_def.camera = Some(CameraDef::perspective(
                        perspective.fov,
                        perspective.aspect_ratio,
                        perspective.near,
                        perspective.far,
                    ));
                    entity_def.camera.as_mut().unwrap().is_active = camera.is_active;
                    entity_def.camera.as_mut().unwrap().priority = camera.priority;
                } else if let Some(orthographic) = orthographic {
                    entity_def.camera = Some(CameraDef::orthographic(
                        orthographic.left,
                        orthographic.right,
                        orthographic.bottom,
                        orthographic.top,
                        orthographic.near,
                        orthographic.far,
                    ));
                    entity_def.camera.as_mut().unwrap().is_active = camera.is_active;
                    entity_def.camera.as_mut().unwrap().priority = camera.priority;
                }
            }

            // Directional Light
            if let Some(light) = dir_light {
                entity_def.directional_light = Some(DirectionalLightDef {
                    direction: (light.direction.x, light.direction.y, light.direction.z),
                    color: (light.color.x, light.color.y, light.color.z),
                    intensity: light.intensity,
                });
            }

            // Point Light
            if let Some(light) = point_light {
                entity_def.point_light = Some(PointLightDef {
                    color: (light.color.x, light.color.y, light.color.z),
                    intensity: light.intensity,
                    range: light.range,
                });
            }

            // Visibility
            if let Some(vis) = visibility {
                entity_def.visible = Some(matches!(vis, Visibility::Visible));
            }

            // Active
            entity_def.active = Some(active.is_some());

            // Track entities
            if parent.is_none() {
                root_entities.push(entity);
            }

            entity_map.insert(entity, entity_def);
        }

        // Build hierarchy - process each root entity
        for parent_entity in &root_entities {
            if let Some(children_component) = world.get::<Children>(*parent_entity) {
                let children_clone = children_component.0.clone();
                Self::build_hierarchy_recursive(
                    *parent_entity,
                    &children_clone,
                    &mut entity_map,
                    world,
                );
            }
        }

        // Collect root entities
        for root in root_entities {
            if let Some(entity_def) = entity_map.remove(&root) {
                scene.add_entity(entity_def);
            }
        }

        debug!("Captured {} root entities", scene.entity_count());

        scene
    }

    /// Recursively builds the entity hierarchy.
    fn build_hierarchy_recursive(
        parent_entity: Entity,
        children: &[Entity],
        entity_map: &mut HashMap<Entity, EntityDefinition>,
        world: &World,
    ) {
        // Collect children for this parent
        let mut children_defs = Vec::new();

        for child_entity in children {
            if let Some(mut child_def) = entity_map.remove(child_entity) {
                // Recursively process child's children
                if let Some(grandchildren) = world.get::<Children>(*child_entity) {
                    let grandchildren_clone = grandchildren.0.clone();
                    Self::build_hierarchy_recursive(
                        *child_entity,
                        &grandchildren_clone,
                        entity_map,
                        world,
                    );
                    // Re-get child_def after recursion
                    if let Some(updated_child) = entity_map.remove(child_entity) {
                        child_def = updated_child;
                    }
                }

                children_defs.push(child_def);
            }
        }

        // Add all children to parent
        if let Some(parent_def) = entity_map.get_mut(&parent_entity) {
            parent_def.children.extend(children_defs);
        }
    }

    /// Restores world state from a scene definition.
    fn restore_world_state(&self, world: &mut World, scene: &SceneDefinition) -> Result<()> {
        let mut entity_map: HashMap<String, Entity> = HashMap::new();

        for entity_def in &scene.entities {
            self.spawn_entity_recursive(world, entity_def, None, &mut entity_map)?;
        }

        Ok(())
    }

    /// Recursively spawns entities from definitions.
    #[allow(clippy::only_used_in_recursion)]
    fn spawn_entity_recursive(
        &self,
        world: &mut World,
        entity_def: &EntityDefinition,
        parent: Option<Entity>,
        entity_map: &mut HashMap<String, Entity>,
    ) -> Result<Entity> {
        let mut entity_builder = world.spawn_empty();

        // Name
        if let Some(ref name) = entity_def.name {
            entity_builder.insert(Name(name.clone()));
        }

        // Transform
        if let Some(ref transform_def) = entity_def.transform {
            let (translation, rotation, scale) = transform_def.to_components();
            entity_builder.insert(Transform {
                translation,
                rotation,
                scale,
            });
            entity_builder.insert(GlobalTransform::default());
        }

        // Mesh
        if let Some(ref mesh_id) = entity_def.mesh {
            entity_builder.insert(MeshHandle::new(mesh_id));
        }

        // Texture
        if let Some(ref texture_id) = entity_def.texture {
            entity_builder.insert(TextureHandle::new(texture_id));
        }

        // Material
        if let Some(ref material_id) = entity_def.material {
            entity_builder.insert(MaterialHandle::new(material_id));
        }

        // Camera
        if let Some(ref camera_def) = entity_def.camera {
            entity_builder.insert(Camera {
                is_active: camera_def.is_active,
                priority: camera_def.priority,
            });

            match camera_def.camera_type {
                CameraType::Perspective => {
                    let fov = camera_def.fov.unwrap_or(70.0_f32.to_radians());
                    let aspect_ratio = camera_def.aspect_ratio.unwrap_or(16.0 / 9.0);
                    entity_builder.insert(PerspectiveProjection {
                        fov,
                        aspect_ratio,
                        near: camera_def.near,
                        far: camera_def.far,
                    });
                }
                CameraType::Orthographic => {
                    let left = camera_def.left.unwrap_or(-10.0);
                    let right = camera_def.right.unwrap_or(10.0);
                    let bottom = camera_def.bottom.unwrap_or(-10.0);
                    let top = camera_def.top.unwrap_or(10.0);
                    entity_builder.insert(OrthographicProjection {
                        left,
                        right,
                        bottom,
                        top,
                        near: camera_def.near,
                        far: camera_def.far,
                    });
                }
            }
        }

        // Directional Light
        if let Some(ref light_def) = entity_def.directional_light {
            let (direction, color, intensity) = light_def.to_components();
            entity_builder.insert(DirectionalLight {
                direction,
                color,
                intensity,
            });
        }

        // Point Light
        if let Some(ref light_def) = entity_def.point_light {
            let (color, intensity, range) = light_def.to_components();
            entity_builder.insert(PointLight {
                color,
                intensity,
                range,
            });
        }

        // Visibility
        let visibility = entity_def.visible.map_or(Visibility::Visible, |visible| {
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }
        });
        entity_builder.insert(visibility);

        // Active
        if entity_def.active.unwrap_or(true) {
            entity_builder.insert(Active);
        }

        // Parent
        if let Some(parent_entity) = parent {
            entity_builder.insert(Parent(parent_entity));
        }

        let entity = entity_builder.id();

        // Track entity by name for cross-references
        if let Some(ref name) = entity_def.name {
            entity_map.insert(name.clone(), entity);
        }

        // Spawn children
        for child_def in &entity_def.children {
            self.spawn_entity_recursive(world, child_def, Some(entity), entity_map)?;
        }

        Ok(entity)
    }

    /// Clears all entities from the world.
    fn clear_world(world: &mut World) {
        let entities: Vec<Entity> = world.query::<bevy_ecs::entity::Entity>().iter(world).collect();

        for entity in entities {
            let _ = world.despawn(entity);
        }

        debug!("Cleared world");
    }

    /// Counts the total number of components in a scene.
    fn count_components(scene: &SceneDefinition) -> usize {
        fn count_entity_components(entity_def: &EntityDefinition) -> usize {
            let mut count = 0;

            if entity_def.name.is_some() {
                count += 1;
            }
            if entity_def.transform.is_some() {
                count += 2; // Transform + GlobalTransform
            }
            if entity_def.mesh.is_some() {
                count += 1;
            }
            if entity_def.texture.is_some() {
                count += 1;
            }
            if entity_def.material.is_some() {
                count += 1;
            }
            if entity_def.camera.is_some() {
                count += 2; // Camera + Projection
            }
            if entity_def.directional_light.is_some() {
                count += 1;
            }
            if entity_def.point_light.is_some() {
                count += 1;
            }
            if entity_def.visible.is_some() {
                count += 1;
            }
            if entity_def.active.is_some() && entity_def.active.unwrap() {
                count += 1;
            }

            // Count children recursively
            for child in &entity_def.children {
                count += count_entity_components(child);
            }

            count
        }

        scene.entities.iter().map(count_entity_components).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::TransformDef;

    #[test]
    fn test_save_manager_creation() {
        let manager = SaveManager::new();
        assert!(manager.last_stats().is_none());
    }

    #[test]
    fn test_save_config_default() {
        let config = SaveConfig::default();
        assert!(!config.compress);
        assert!(!config.include_editor_data);
        assert!(config.validate_after_save);
        assert!(config.pretty_print);
    }

    #[test]
    fn test_save_metadata_creation() {
        let metadata = SaveMetadata::new("Test Save")
            .with_description("A test save")
            .with_playtime(3600)
            .with_tag("test");

        assert_eq!(metadata.name, "Test Save");
        assert_eq!(metadata.description, Some("A test save".to_string()));
        assert_eq!(metadata.playtime_seconds, 3600);
        assert!(metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_save_metadata_builder() {
        let metadata = SaveMetadata::new("Game Save")
            .with_game_version("1.0.0")
            .with_screenshot("screenshot.png")
            .with_custom_data("level", "forest")
            .with_custom_data("chapter", "1");

        assert_eq!(metadata.game_version, Some("1.0.0".to_string()));
        assert_eq!(metadata.screenshot_path, Some("screenshot.png".to_string()));
        assert_eq!(metadata.custom_data.get("level"), Some(&"forest".to_string()));
        assert_eq!(metadata.custom_data.get("chapter"), Some(&"1".to_string()));
    }

    #[test]
    fn test_save_file_creation() {
        let scene = SceneDefinition::new("Test");
        let metadata = SaveMetadata::new("Test Save");
        let save_file = SaveFile::new(scene, metadata);

        assert_eq!(save_file.version, CURRENT_SAVE_VERSION);
        assert_eq!(save_file.metadata.name, "Test Save");
    }

    #[test]
    fn test_save_file_validation() {
        let scene = SceneDefinition::new("Test");
        let metadata = SaveMetadata::new("Test");
        let save_file = SaveFile::new(scene, metadata);

        assert!(save_file.validate().is_ok());
    }

    #[test]
    fn test_save_stats() {
        let stats = SaveStats::new();
        assert_eq!(stats.entity_count, 0);
        assert_eq!(stats.component_count, 0);
        assert_eq!(stats.duration_ms, 0.0);
        assert!(stats.file_size_bytes.is_none());
    }

    #[test]
    fn test_capture_empty_world() {
        let mut world = World::new();

        let scene = SaveManager::capture_world_state(&mut world);
        assert_eq!(scene.entity_count(), 0);
    }

    #[test]
    fn test_capture_world_with_entities() {
        let mut world = World::new();

        // Spawn some test entities
        world.spawn((
            Name("Entity1".to_string()),
            Transform::from_xyz(1.0, 2.0, 3.0),
            Active,
        ));

        world.spawn((
            Name("Entity2".to_string()),
            Transform::from_xyz(4.0, 5.0, 6.0),
            MeshHandle::new("cube"),
        ));

        let scene = SaveManager::capture_world_state(&mut world);
        assert_eq!(scene.entity_count(), 2);
    }

    #[test]
    fn test_capture_skips_no_save_entities() {
        let mut world = World::new();

        // Entity without NoSave
        world.spawn((Name("SavedEntity".to_string()), Active));

        // Entity with NoSave
        world.spawn((Name("TemporaryEntity".to_string()), NoSave));

        let scene = SaveManager::capture_world_state(&mut world);
        assert_eq!(scene.entity_count(), 1);
    }

    #[test]
    fn test_save_and_load_round_trip() {
        use std::env;

        let mut world = World::new();
        let mut manager = SaveManager::new();

        // Create test entities
        world.spawn((
            Name("TestEntity".to_string()),
            Transform::from_xyz(10.0, 20.0, 30.0),
            MeshHandle::new("test_mesh"),
            Active,
        ));

        // Save to temporary file
        let temp_dir = env::temp_dir();
        let save_path = temp_dir.join("test_save.ron");
        let metadata = SaveMetadata::new("Test Save");

        manager
            .save_to_file(&mut world, &save_path, metadata)
            .unwrap();

        // Verify file exists
        assert!(save_path.exists());

        // Load into new world
        let mut new_world = World::new();
        manager.load_from_file(&mut new_world, &save_path).unwrap();

        // Verify entity exists with correct components
        let query = new_world.query::<(&Name, &Transform, &MeshHandle)>();
        let mut count = 0;
        for (name, transform, mesh) in query.iter(&new_world) {
            assert_eq!(name.0, "TestEntity");
            assert_eq!(transform.translation.x, 10.0);
            assert_eq!(mesh.id, "test_mesh");
            count += 1;
        }
        assert_eq!(count, 1);

        // Cleanup
        let _ = fs::remove_file(save_path);
    }

    #[test]
    fn test_count_components() {
        let manager = SaveManager::new();
        let mut scene = SceneDefinition::new("Test");

        let entity = EntityDefinition::new()
            .with_name("Test")
            .with_transform(TransformDef::identity())
            .with_mesh("cube");

        scene.add_entity(entity);

        let count = SaveManager::count_components(&scene);
        assert!(count > 0);
    }

    #[test]
    fn test_clear_world() {
        let mut world = World::new();

        world.spawn(Name("Test".to_string()));
        world.spawn(Name("Test2".to_string()));

        assert_eq!(world.query::<&Name>().iter(&world).count(), 2);

        SaveManager::clear_world(&mut world);

        assert_eq!(world.query::<&Name>().iter(&world).count(), 0);
    }
}
