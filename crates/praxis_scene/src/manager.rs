//! Scene manager for spawning and managing scene instances.

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
/// let handle = manager.spawn_scene(&mut world, &scene_def).unwrap();
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
    #[must_use]
    pub fn new() -> Self {
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
        scene_def: &SceneDefinition,
    ) -> Result<SceneHandle> {
        let handle = SceneHandle::generate();
        info!(
            "Spawning scene '{}' with handle '{}'",
            scene_def.name,
            handle.id()
        );

        let mut root_entities = Vec::new();

        for entity_def in &scene_def.entities {
            let entity = Self::spawn_entity_recursive(world, entity_def, &handle, None)?;
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
        world: &mut World,
        entity_def: &EntityDefinition,
        scene_handle: &SceneHandle,
        parent: Option<Entity>,
    ) -> Result<Entity> {
        let entity = Self::spawn_entity(world, entity_def, scene_handle, parent);

        for child_def in &entity_def.children {
            Self::spawn_entity_recursive(world, child_def, scene_handle, Some(entity))?;
        }

        Ok(entity)
    }

    /// Spawns a single entity from a definition.
    fn spawn_entity(
        world: &mut World,
        entity_def: &EntityDefinition,
        scene_handle: &SceneHandle,
        parent: Option<Entity>,
    ) -> Entity {
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

        let visibility = entity_def.visible.map_or(Visibility::Visible, |visible| {
            if visible {
                Visibility::Visible
            } else {
                Visibility::Hidden
            }
        });
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

        entity
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
        self.scenes.remove(handle).map_or_else(
            || {
                debug!("Scene '{}' not found in manager", handle.id());
                false
            },
            |root_entities| {
                info!("Unloading scene '{}'", handle.id());

                for entity in root_entities {
                    Self::despawn_recursive(world, entity);
                }

                debug!("Scene '{}' unloaded", handle.id());
                true
            },
        )
    }

    /// Despawns an entity and all its descendants recursively.
    fn despawn_recursive(world: &mut World, entity: Entity) {
        if let Some(children) = world.get::<Children>(entity) {
            let children_vec: Vec<Entity> = children.0.clone();
            for child in children_vec {
                Self::despawn_recursive(world, child);
            }
        }

        let _ = world.despawn(entity);
        debug!("Despawned entity {:?}", entity);
    }

    /// Checks if a scene is currently loaded.
    #[must_use]
    pub fn is_scene_loaded(&self, handle: &SceneHandle) -> bool {
        self.scenes.contains_key(handle)
    }

    /// Gets the number of currently loaded scenes.
    #[must_use]
    pub fn loaded_scene_count(&self) -> usize {
        self.scenes.len()
    }

    /// Gets the root entities for a loaded scene.
    #[must_use]
    pub fn get_scene_entities(&self, handle: &SceneHandle) -> Option<&[Entity]> {
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

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();

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

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();

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

        let handle1 = manager.spawn_scene(&mut world, &scene1).unwrap();
        let handle2 = manager.spawn_scene(&mut world, &scene2).unwrap();

        assert_eq!(manager.loaded_scene_count(), 2);
        assert!(manager.is_scene_loaded(&handle1));
        assert!(manager.is_scene_loaded(&handle2));

        manager.unload_all(&mut world);
        assert_eq!(manager.loaded_scene_count(), 0);
    }

    #[test]
    fn test_unload_scene_not_found() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let handle = SceneHandle::new("nonexistent");
        let result = manager.unload_scene(&mut world, &handle);
        assert!(!result);
    }

    #[test]
    fn test_get_scene_entities_valid() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let mut scene = SceneDefinition::new("Test Scene");
        scene.add_entity(
            EntityDefinition::new()
                .with_name("Entity1")
                .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0)),
        );

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();

        let entities = manager.get_scene_entities(&handle);
        assert!(entities.is_some());
        assert_eq!(entities.unwrap().len(), 1);
    }

    #[test]
    fn test_get_scene_entities_invalid() {
        let manager = SceneManager::new();
        let handle = SceneHandle::new("nonexistent");

        let entities = manager.get_scene_entities(&handle);
        assert!(entities.is_none());
    }

    #[test]
    fn test_spawn_empty_scene() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let scene = SceneDefinition::new("Empty Scene");
        let handle = manager.spawn_scene(&mut world, &scene).unwrap();

        assert!(manager.is_scene_loaded(&handle));
        let entities = manager.get_scene_entities(&handle).unwrap();
        assert_eq!(entities.len(), 0);
    }

    #[test]
    fn test_spawn_scene_with_components() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let mut scene = SceneDefinition::new("Test Scene");
        let mut entity = EntityDefinition::new()
            .with_name("TestEntity")
            .with_transform(TransformDef::from_translation(1.0, 2.0, 3.0))
            .with_mesh("test_mesh");
        entity.visible = Some(true);
        entity.active = Some(true);

        scene.add_entity(entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();

        let entities = manager.get_scene_entities(&handle).unwrap();
        assert_eq!(entities.len(), 1);

        let spawned_entity = entities[0];
        assert!(world.get::<Name>(spawned_entity).is_some());
        assert!(world.get::<Transform>(spawned_entity).is_some());
        assert!(world.get::<MeshHandle>(spawned_entity).is_some());
        assert!(world.get::<Visibility>(spawned_entity).is_some());
        assert!(world.get::<Active>(spawned_entity).is_some());
    }

    #[test]
    fn test_despawn_recursive() {
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

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        assert_eq!(entities.len(), 1);

        manager.unload_scene(&mut world, &handle);
        assert!(!manager.is_scene_loaded(&handle));
    }

    #[test]
    fn test_manager_default() {
        let manager = SceneManager::default();
        assert_eq!(manager.loaded_scene_count(), 0);
    }

    #[test]
    fn test_scene_with_camera() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let camera_entity =
            EntityDefinition::perspective_camera("MainCamera", (0.0, 5.0, 10.0), 1.22, 16.0 / 9.0);

        let mut scene = SceneDefinition::new("Camera Scene");
        scene.add_entity(camera_entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        assert!(world.get::<Camera>(spawned_entity).is_some());
        assert!(world.get::<PerspectiveProjection>(spawned_entity).is_some());
        assert!(world.get::<CameraMatrices>(spawned_entity).is_some());
    }

    #[test]
    fn test_scene_with_directional_light() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let light_entity =
            EntityDefinition::directional_light("Sun", (0.0, -1.0, 0.0), (1.0, 1.0, 1.0), 1.5);

        let mut scene = SceneDefinition::new("Light Scene");
        scene.add_entity(light_entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        assert!(world.get::<DirectionalLight>(spawned_entity).is_some());
    }

    #[test]
    fn test_scene_with_point_light() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let light_entity =
            EntityDefinition::point_light("Lamp", (0.0, 2.0, 0.0), (1.0, 0.8, 0.6), 2.0, 10.0);

        let mut scene = SceneDefinition::new("Point Light Scene");
        scene.add_entity(light_entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        assert!(world.get::<PointLight>(spawned_entity).is_some());
        assert!(world.get::<Transform>(spawned_entity).is_some());
    }

    #[test]
    fn test_scene_with_texture() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let entity = EntityDefinition::textured_mesh_entity(
            "TexturedCube",
            (0.0, 0.0, 0.0),
            "cube_mesh",
            "cube_texture",
        );

        let mut scene = SceneDefinition::new("Textured Scene");
        scene.add_entity(entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        assert!(world.get::<MeshHandle>(spawned_entity).is_some());
        assert!(world.get::<TextureHandle>(spawned_entity).is_some());
    }

    #[test]
    fn test_deep_hierarchy_spawning() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let great_grandchild = EntityDefinition::new()
            .with_name("GreatGrandchild")
            .with_transform(TransformDef::from_translation(3.0, 0.0, 0.0));

        let grandchild = EntityDefinition::new()
            .with_name("Grandchild")
            .with_transform(TransformDef::from_translation(2.0, 0.0, 0.0))
            .with_child(great_grandchild);

        let child = EntityDefinition::new()
            .with_name("Child")
            .with_transform(TransformDef::from_translation(1.0, 0.0, 0.0))
            .with_child(grandchild);

        let parent = EntityDefinition::new()
            .with_name("Parent")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0))
            .with_child(child);

        let mut scene = SceneDefinition::new("Deep Hierarchy");
        scene.add_entity(parent);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        assert!(manager.is_scene_loaded(&handle));

        let entities = manager.get_scene_entities(&handle).unwrap();
        assert_eq!(entities.len(), 1);

        manager.unload_scene(&mut world, &handle);
        assert!(!manager.is_scene_loaded(&handle));
    }

    #[test]
    fn test_multiple_root_entities() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let mut scene = SceneDefinition::new("Multiple Roots");
        scene.add_entity(
            EntityDefinition::new()
                .with_name("Root1")
                .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0)),
        );
        scene.add_entity(
            EntityDefinition::new()
                .with_name("Root2")
                .with_transform(TransformDef::from_translation(5.0, 0.0, 0.0)),
        );
        scene.add_entity(
            EntityDefinition::new()
                .with_name("Root3")
                .with_transform(TransformDef::from_translation(10.0, 0.0, 0.0)),
        );

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        assert_eq!(entities.len(), 3);

        manager.unload_scene(&mut world, &handle);
    }

    #[test]
    fn test_unload_all_empty() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        manager.unload_all(&mut world);
        assert_eq!(manager.loaded_scene_count(), 0);
    }

    #[test]
    fn test_visibility_hidden() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let mut entity = EntityDefinition::new()
            .with_name("HiddenEntity")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0));
        entity.visible = Some(false);

        let mut scene = SceneDefinition::new("Visibility Test");
        scene.add_entity(entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        let visibility = world.get::<Visibility>(spawned_entity).unwrap();
        assert_eq!(*visibility, Visibility::Hidden);
    }

    #[test]
    fn test_inactive_entity() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let mut entity = EntityDefinition::new()
            .with_name("InactiveEntity")
            .with_transform(TransformDef::from_translation(0.0, 0.0, 0.0));
        entity.active = Some(false);

        let mut scene = SceneDefinition::new("Active Test");
        scene.add_entity(entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        assert!(world.get::<Active>(spawned_entity).is_none());
    }

    #[test]
    fn test_orthographic_camera() {
        let mut world = World::new();
        let mut manager = SceneManager::new();

        let camera_entity =
            EntityDefinition::orthographic_camera("OrthoCamera", (0.0, 0.0, 10.0), (20.0, 15.0));

        let mut scene = SceneDefinition::new("Ortho Camera Scene");
        scene.add_entity(camera_entity);

        let handle = manager.spawn_scene(&mut world, &scene).unwrap();
        let entities = manager.get_scene_entities(&handle).unwrap();
        let spawned_entity = entities[0];

        assert!(world.get::<Camera>(spawned_entity).is_some());
        assert!(world
            .get::<OrthographicProjection>(spawned_entity)
            .is_some());
    }
}
