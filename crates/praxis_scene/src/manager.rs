//! Scene manager for spawning and managing scene instances.

#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::unused_self)]
#![allow(clippy::only_used_in_recursion)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::option_if_let_else)]

use crate::{
    components::{Scene, SceneHandle},
    definition::{CameraType, EntityDefinition, SceneDefinition},
};
use praxis_ecs::{
    Active, Camera, CameraMatrices, Children, DirectionalLight, Entity, GlobalTransform,
    MeshHandle, Name, OrthographicProjection, Parent, PerspectiveProjection, PointLight,
    TextureHandle, Transform, Visibility, World,
};
use praxis_utils::{debug, info, Result};
use std::collections::HashMap;

/// Scene manager for spawning and managing scene instances.
///
/// The scene manager maintains a registry of loaded scenes and provides
/// functionality to spawn entities from scene definitions and clean them up.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_scene::{SceneManager, SceneDefinition};
/// use praxis_ecs::World;
///
/// let mut world = World::new();
/// let mut manager = SceneManager::new();
///
/// let scene_def = SceneDefinition::new("TestScene");
/// let handle = manager.spawn_scene(&mut world, scene_def).unwrap();
///
/// // Later, unload the scene
/// manager.unload_scene(&mut world, &handle);
/// ```
#[derive(Debug, Default)]
pub struct SceneManager {
    /// Map of scene handles to their root entities.
    scenes: HashMap<SceneHandle, Vec<Entity>>,
}

impl SceneManager {
    /// Creates a new scene manager.
    #[must_use] pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
        }
    }

    /// Spawns a scene into the world from a scene definition.
    ///
    /// Returns a handle to the spawned scene that can be used to unload it later.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to spawn entities into
    /// * `scene_def` - The scene definition to spawn
    ///
    /// # Errors
    ///
    /// Returns an error if entity spawning fails.
    pub fn spawn_scene(
        &mut self,
        world: &mut World,
        scene_def: SceneDefinition,
    ) -> Result<SceneHandle> {
        let handle = SceneHandle::generate();
        info!(
            "Spawning scene '{}' with handle '{}'",
            scene_def.name,
            handle.id()
        );

        let mut root_entities = Vec::new();

        for entity_def in &scene_def.entities {
            let entity = self.spawn_entity_recursive(world, entity_def, &handle, None)?;
            root_entities.push(entity);
        }

        debug!(
            "Spawned {} root entities for scene '{}'",
            root_entities.len(),
            scene_def.name
        );

        self.scenes.insert(handle.clone(), root_entities);

        Ok(handle)
    }

    /// Spawns an entity and its children recursively.
    fn spawn_entity_recursive(
        &self,
        world: &mut World,
        entity_def: &EntityDefinition,
        scene_handle: &SceneHandle,
        parent: Option<Entity>,
    ) -> Result<Entity> {
        let entity = self.spawn_entity(world, entity_def, scene_handle, parent)?;

        for child_def in &entity_def.children {
            self.spawn_entity_recursive(world, child_def, scene_handle, Some(entity))?;
        }

        Ok(entity)
    }

    /// Spawns a single entity from a definition.
    fn spawn_entity(
        &self,
        world: &mut World,
        entity_def: &EntityDefinition,
        scene_handle: &SceneHandle,
        parent: Option<Entity>,
    ) -> Result<Entity> {
        let mut entity_builder = world.spawn_empty();

        entity_builder.insert(Scene(scene_handle.clone()));

        if let Some(ref name) = entity_def.name {
            entity_builder.insert(Name(name.clone()));
        }

        if let Some(ref transform_def) = entity_def.transform {
            let (translation, rotation, scale) = transform_def.to_components();
            entity_builder.insert(Transform {
                translation,
                rotation,
                scale,
            });
            entity_builder.insert(GlobalTransform::default());
        }

        if let Some(ref mesh_id) = entity_def.mesh {
            entity_builder.insert(MeshHandle::new(mesh_id));
        }

        if let Some(ref texture_id) = entity_def.texture {
            entity_builder.insert(TextureHandle::new(texture_id));
        }

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

            entity_builder.insert(CameraMatrices::default());
        }

        if let Some(ref light_def) = entity_def.directional_light {
            let (direction, color, intensity) = light_def.to_components();
            entity_builder.insert(DirectionalLight {
                direction,
                color,
                intensity,
            });
        }

        if let Some(ref light_def) = entity_def.point_light {
            let (color, intensity, range) = light_def.to_components();
            entity_builder.insert(PointLight {
                color,
                intensity,
                range,
            });
        }

        let visibility = if let Some(visible) = entity_def.visible {
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }
        } else {
            Visibility::Visible
        };
        entity_builder.insert(visibility);

        if entity_def.active.unwrap_or(true) {
            entity_builder.insert(Active);
        }

        if let Some(parent_entity) = parent {
            entity_builder.insert(Parent(parent_entity));
        }

        let entity = entity_builder.id();

        debug!(
            "Spawned entity {:?} with name '{}'",
            entity,
            entity_def.name.as_deref().unwrap_or("<unnamed>")
        );

        Ok(entity)
    }

    /// Unloads a scene, removing all its entities from the world.
    ///
    /// # Arguments
    ///
    /// * `world` - The ECS world to remove entities from
    /// * `handle` - Handle to the scene to unload
    ///
    /// # Returns
    ///
    /// Returns `true` if the scene was found and unloaded, `false` if the scene handle was not found.
    pub fn unload_scene(&mut self, world: &mut World, handle: &SceneHandle) -> bool {
        if let Some(root_entities) = self.scenes.remove(handle) {
            info!("Unloading scene '{}'", handle.id());

            for entity in root_entities {
                self.despawn_recursive(world, entity);
            }

            debug!("Scene '{}' unloaded", handle.id());
            true
        } else {
            debug!("Scene '{}' not found in manager", handle.id());
            false
        }
    }

    /// Despawns an entity and all its descendants recursively.
    fn despawn_recursive(&self, world: &mut World, entity: Entity) {
        if let Some(children) = world.get::<Children>(entity) {
            let children_vec: Vec<Entity> = children.0.clone();
            for child in children_vec {
                self.despawn_recursive(world, child);
            }
        }

        let _ = world.despawn(entity);
        debug!("Despawned entity {:?}", entity);
    }

    /// Checks if a scene is currently loaded.
    #[must_use] pub fn is_scene_loaded(&self, handle: &SceneHandle) -> bool {
        self.scenes.contains_key(handle)
    }

    /// Gets the number of currently loaded scenes.
    #[must_use] pub fn loaded_scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Gets the root entities for a loaded scene.
    #[must_use] pub fn get_scene_entities(&self, handle: &SceneHandle) -> Option<&[Entity]> {
        self.scenes.get(handle).map(std::vec::Vec::as_slice)
    }

    /// Unloads all scenes.
    pub fn unload_all(&mut self, world: &mut World) {
        info!("Unloading all scenes");
        let handles: Vec<SceneHandle> = self.scenes.keys().cloned().collect();
        for handle in handles {
            self.unload_scene(world, &handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition::{EntityDefinition, TransformDef};

    #[test]
    fn test_spawn_and_unload_scene() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let mut scene = SceneDefinition::new("Test Scene");
        scene.add_entity(
            EntityDefinition::new()
                .with_name("TestEntity")
                .with_transform(TransformDef::from_translation(1.0, 2.0, 3.0)),
        );

        let handle = manager.spawn_scene(&mut world, scene).unwrap();

        assert!(manager.is_scene_loaded(&handle));
        assert_eq!(manager.loaded_scene_count(), 1);

        let unloaded = manager.unload_scene(&mut world, &handle);
        assert!(unloaded);
        assert!(!manager.is_scene_loaded(&handle));
        assert_eq!(manager.loaded_scene_count(), 0);
    }

    #[test]
    fn test_spawn_hierarchical_scene() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let child = EntityDefinition::new()
            .with_name("Child")
            .with_transform(TransformDef::from_translation(1.0, 0.0, 0.0));

        let parent = EntityDefinition::new()
            .with_name("Parent")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0))
            .with_child(child);

        let mut scene = SceneDefinition::new("Hierarchy Scene");
        scene.add_entity(parent);

        let handle = manager.spawn_scene(&mut world, scene).unwrap();

        assert!(manager.is_scene_loaded(&handle));

        let entities = manager.get_scene_entities(&handle).unwrap();
        assert_eq!(entities.len(), 1);

        manager.unload_scene(&mut world, &handle);
        assert!(!manager.is_scene_loaded(&handle));
    }

    #[test]
    fn test_multiple_scenes() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let scene1 = SceneDefinition::new("Scene 1");
        let scene2 = SceneDefinition::new("Scene 2");

        let handle1 = manager.spawn_scene(&mut world, scene1).unwrap();
        let handle2 = manager.spawn_scene(&mut world, scene2).unwrap();

        assert_eq!(manager.loaded_scene_count(), 2);
        assert!(manager.is_scene_loaded(&handle1));
        assert!(manager.is_scene_loaded(&handle2));

        manager.unload_all(&mut world);
        assert_eq!(manager.loaded_scene_count(), 0);
    }
}
