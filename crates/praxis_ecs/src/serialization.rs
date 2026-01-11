//! ECS component serialization system.
//!
//! This module provides a flexible system for serializing and deserializing ECS world state
//! to and from RON (Rusty Object Notation) format. It supports component registration,
//! custom serialization, and world snapshots.
//!
//! # Core Concepts
//!
//! - **`SerializableComponent`**: Trait for components that can be serialized
//! - **`ComponentRegistry`**: Registry for mapping component type names to serialization functions
//! - **`WorldSnapshot`**: Serializable representation of world state
//! - **`EntityData`**: Serializable representation of an entity and its components
//!
//! # Example
//!
//! ```rust,no_run
//! use praxis_ecs::{World, ComponentRegistry, Transform, Name};
//! use praxis_math::Vec3;
//!
//! // Create and populate world
//! let mut world = World::new();
//! world.spawn((
//!     Name::new("Player"),
//!     Transform::from_xyz(1.0, 2.0, 3.0),
//! ));
//!
//! // Create registry and register components
//! let mut registry = ComponentRegistry::new();
//! registry.register::<Name>();
//! registry.register::<Transform>();
//!
//! // Serialize world to RON
//! let ron_string = registry.serialize_world(&world).unwrap();
//!
//! // Deserialize into a new world
//! let mut new_world = World::new();
//! registry.deserialize_world(&ron_string, &mut new_world).unwrap();
//! ```

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::World as BevyWorld;
use praxis_utils::{error, info, Result};
use serde::{Deserialize, Serialize};
use std::any::TypeId;
use std::collections::HashMap;

type InsertComponentFn = Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>;
type SerializeFn =
    Box<dyn Fn(&BevyWorld, Entity) -> Result<Option<(String, String)>> + Send + Sync>;
type DeserializeFn =
    Box<dyn Fn(&str, Entity, &DeserializeContext) -> Result<InsertComponentFn> + Send + Sync>;
type EntityCallbacks = Vec<(Entity, Vec<InsertComponentFn>)>;

/// Trait for components that can be serialized.
///
/// This trait provides the mechanism for converting components to and from
/// a serialized form. Components must implement this trait to participate
/// in world serialization.
///
/// # Example
///
/// ```rust
/// use praxis_ecs::{Component, SerializableComponent, DeserializeContext};
/// use serde::{Serialize, Deserialize};
/// use bevy_ecs::entity::Entity;
/// use praxis_utils::Result;
///
/// #[derive(Component, Serialize, Deserialize, Clone)]
/// struct Health {
///     current: f32,
///     max: f32,
/// }
///
/// impl SerializableComponent for Health {
///     fn serialize_component(&self) -> Result<String> {
///         Ok(ron::to_string(self)?)
///     }
///
///     fn deserialize_component(
///         data: &str,
///         _entity: Entity,
///         _context: &DeserializeContext,
///     ) -> Result<InsertComponentFn>
///     where
///         Self: Sized + 'static,
///     {
///         let component: Health = ron::from_str(data)?;
///         Ok(Box::new(move |entity_mut| {
///             entity_mut.insert(component);
///         }))
///     }
///
///     fn type_name() -> &'static str
///     where
///         Self: Sized,
///     {
///         "Health"
///     }
/// }
/// ```
pub trait SerializableComponent: Component {
    /// Serializes this component to a RON string.
    fn serialize_component(&self) -> Result<String>;

    /// Deserializes a component from a RON string and returns a closure that inserts it.
    ///
    /// The closure approach allows for deferred insertion and custom deserialization logic.
    ///
    /// # Arguments
    ///
    /// * `data` - The serialized component data
    /// * `entity` - The entity this component will be attached to
    /// * `context` - Context for entity reference resolution
    fn deserialize_component(
        data: &str,
        entity: Entity,
        context: &DeserializeContext,
    ) -> Result<InsertComponentFn>
    where
        Self: Sized + 'static;

    /// Returns the type name for this component.
    fn type_name() -> &'static str
    where
        Self: Sized;
}

/// Context for deserializing components with entity references.
///
/// This context maps serialized entity IDs to actual runtime entity IDs,
/// allowing components that reference other entities to maintain correct
/// relationships after deserialization.
#[derive(Debug, Clone)]
pub struct DeserializeContext {
    entity_map: HashMap<u64, Entity>,
}

impl DeserializeContext {
    /// Creates a new deserialization context.
    pub fn new() -> Self {
        Self {
            entity_map: HashMap::new(),
        }
    }

    /// Maps a serialized entity ID to a runtime entity.
    pub fn map_entity(&mut self, serialized_id: u64, entity: Entity) {
        self.entity_map.insert(serialized_id, entity);
    }

    /// Gets the runtime entity for a serialized entity ID.
    pub fn get_entity(&self, serialized_id: u64) -> Option<Entity> {
        self.entity_map.get(&serialized_id).copied()
    }
}

impl Default for DeserializeContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable representation of an entity and its components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityData {
    /// Serialized entity ID (for reference resolution).
    pub id: u64,

    /// Component type names mapped to their serialized data.
    pub components: HashMap<String, String>,
}

impl EntityData {
    /// Creates a new entity data container.
    pub fn new(id: u64) -> Self {
        Self {
            id,
            components: HashMap::new(),
        }
    }

    /// Adds a serialized component to this entity.
    pub fn add_component(&mut self, type_name: String, data: String) {
        self.components.insert(type_name, data);
    }
}

/// Serializable representation of world state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldSnapshot {
    /// All entities and their components.
    pub entities: Vec<EntityData>,
}

impl WorldSnapshot {
    /// Creates a new empty world snapshot.
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
        }
    }

    /// Adds an entity to this snapshot.
    pub fn add_entity(&mut self, entity: EntityData) {
        self.entities.push(entity);
    }

    /// Returns the number of entities in this snapshot.
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// Serializes this snapshot to a RON string.
    pub fn to_ron(&self) -> Result<String> {
        Ok(ron::ser::to_string_pretty(
            self,
            ron::ser::PrettyConfig::default(),
        )?)
    }

    /// Deserializes a snapshot from a RON string.
    pub fn from_ron(data: &str) -> Result<Self> {
        Ok(ron::from_str(data)?)
    }
}

impl Default for WorldSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

/// Component registry for mapping component types to serialization functions.
///
/// The registry maintains mappings between component type names and the functions
/// needed to serialize and deserialize them. It provides the main API for world
/// serialization.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{ComponentRegistry, Transform, Name};
///
/// let mut registry = ComponentRegistry::new();
/// registry.register::<Name>();
/// registry.register::<Transform>();
/// ```
pub struct ComponentRegistry {
    serialize_fns: HashMap<TypeId, SerializeFn>,
    deserialize_fns: HashMap<String, DeserializeFn>,
    type_names: HashMap<TypeId, String>,
}

impl ComponentRegistry {
    /// Creates a new component registry.
    pub fn new() -> Self {
        Self {
            serialize_fns: HashMap::new(),
            deserialize_fns: HashMap::new(),
            type_names: HashMap::new(),
        }
    }

    /// Registers a component type for serialization.
    ///
    /// This method sets up both serialization and deserialization functions
    /// for the given component type.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{ComponentRegistry, Transform};
    ///
    /// let mut registry = ComponentRegistry::new();
    /// registry.register::<Transform>();
    /// ```
    pub fn register<T: SerializableComponent + 'static>(&mut self) {
        let type_id = TypeId::of::<T>();
        let type_name = T::type_name().to_string();

        self.type_names.insert(type_id, type_name.clone());

        self.serialize_fns.insert(
            type_id,
            Box::new(move |world: &BevyWorld, entity: Entity| {
                if let Some(component) = world.get::<T>(entity) {
                    let serialized = component.serialize_component()?;
                    Ok(Some((T::type_name().to_string(), serialized)))
                } else {
                    Ok(None)
                }
            }),
        );

        self.deserialize_fns.insert(
            type_name,
            Box::new(
                move |data: &str, entity: Entity, context: &DeserializeContext| {
                    T::deserialize_component(data, entity, context)
                },
            ),
        );
    }

    /// Serializes the entire world to a RON string.
    ///
    /// This method iterates through all entities in the world and serializes
    /// their registered components, excluding entities marked with `NoSave`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, ComponentRegistry, Transform};
    ///
    /// let world = World::new();
    /// let registry = ComponentRegistry::new();
    /// let ron_string = registry.serialize_world(&world).unwrap();
    /// ```
    pub fn serialize_world(&self, world: &crate::World) -> Result<String> {
        info!("Serializing world state");
        let mut snapshot = WorldSnapshot::new();

        for entity in world.inner().iter_entities() {
            if world.inner().get::<crate::NoSave>(entity.id()).is_some() {
                continue;
            }

            let mut entity_data = EntityData::new(entity.id().index() as u64);

            for serialize_fn in self.serialize_fns.values() {
                if let Ok(Some((type_name, data))) = serialize_fn(world.inner(), entity.id()) {
                    entity_data.add_component(type_name, data);
                }
            }

            if !entity_data.components.is_empty() {
                snapshot.add_entity(entity_data);
            }
        }

        info!(
            "Serialized {} entities with {} registered component types",
            snapshot.entity_count(),
            self.serialize_fns.len()
        );

        snapshot.to_ron()
    }

    /// Deserializes a world from a RON string.
    ///
    /// This method creates entities and components based on the serialized data.
    /// It builds an entity mapping to preserve entity references.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, ComponentRegistry};
    ///
    /// let mut world = World::new();
    /// let registry = ComponentRegistry::new();
    /// let ron_string = "(entities: [])"; // Empty world
    /// registry.deserialize_world(ron_string, &mut world).unwrap();
    /// ```
    pub fn deserialize_world(&self, data: &str, world: &mut crate::World) -> Result<()> {
        info!("Deserializing world state");
        let snapshot = WorldSnapshot::from_ron(data)?;
        let mut context = DeserializeContext::new();

        let mut entity_callbacks: EntityCallbacks = Vec::new();

        for entity_data in &snapshot.entities {
            let entity_mut = world.spawn_empty();
            let entity = entity_mut.id();
            context.map_entity(entity_data.id, entity);

            let mut callbacks = Vec::new();
            for (type_name, component_data) in &entity_data.components {
                if let Some(deserialize_fn) = self.deserialize_fns.get(type_name) {
                    match deserialize_fn(component_data, entity, &context) {
                        Ok(callback) => callbacks.push(callback),
                        Err(e) => {
                            error!(
                                "Failed to deserialize component {} for entity {:?}: {}",
                                type_name, entity, e
                            );
                        }
                    }
                } else {
                    error!(
                        "No deserializer registered for component type: {}",
                        type_name
                    );
                }
            }

            entity_callbacks.push((entity, callbacks));
        }

        for (entity, callbacks) in entity_callbacks {
            for callback in callbacks {
                if let Some(mut entity_mut) = world.inner_mut().get_entity_mut(entity) {
                    callback(&mut entity_mut);
                }
            }
        }

        info!(
            "Deserialized {} entities with {} registered component types",
            snapshot.entity_count(),
            self.deserialize_fns.len()
        );

        Ok(())
    }

    /// Returns the number of registered component types.
    pub fn registered_count(&self) -> usize {
        self.serialize_fns.len()
    }

    /// Checks if a component type is registered.
    pub fn is_registered<T: Component + 'static>(&self) -> bool {
        self.serialize_fns.contains_key(&TypeId::of::<T>())
    }

    /// Gets the registered type name for a component type.
    pub fn get_type_name<T: Component + 'static>(&self) -> Option<&str> {
        self.type_names.get(&TypeId::of::<T>()).map(|s| s.as_str())
    }

    /// Registers all common built-in component types.
    ///
    /// This is a convenience method that registers:
    /// - `Name`
    /// - `Transform`
    /// - `GlobalTransform`
    /// - `Parent`
    /// - `Children`
    /// - `MeshHandle`
    /// - `MaterialHandle`
    /// - `Visibility`
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::ComponentRegistry;
    ///
    /// let mut registry = ComponentRegistry::new();
    /// registry.register_common_types();
    /// ```
    pub fn register_common_types(&mut self) {
        self.register::<crate::Name>();
        self.register::<crate::Transform>();
        self.register::<crate::GlobalTransform>();
        self.register::<crate::Parent>();
        self.register::<crate::Children>();
        self.register::<crate::MeshHandle>();
        self.register::<crate::MaterialHandle>();
        self.register::<crate::Visibility>();
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// Component implementations for common types
mod impls {
    use super::*;
    use crate::{
        Children, GlobalTransform, MaterialHandle, MeshHandle, Name, Parent, Transform, Visibility,
    };

    impl SerializableComponent for Name {
        fn serialize_component(&self) -> Result<String> {
            Ok(ron::to_string(self)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let component: Name = ron::from_str(data)?;
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "Name"
        }
    }

    impl SerializableComponent for Transform {
        fn serialize_component(&self) -> Result<String> {
            Ok(ron::to_string(self)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let component: Transform = ron::from_str(data)?;
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "Transform"
        }
    }

    impl SerializableComponent for GlobalTransform {
        fn serialize_component(&self) -> Result<String> {
            Ok(ron::to_string(self)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let component: GlobalTransform = ron::from_str(data)?;
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "GlobalTransform"
        }
    }

    #[derive(Serialize, Deserialize)]
    struct SerializableParent {
        entity_id: u64,
    }

    impl SerializableComponent for Parent {
        fn serialize_component(&self) -> Result<String> {
            let serializable = SerializableParent {
                entity_id: self.0.index() as u64,
            };
            Ok(ron::to_string(&serializable)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let serializable: SerializableParent = ron::from_str(data)?;
            let parent_entity = context.get_entity(serializable.entity_id).ok_or_else(|| {
                praxis_utils::eyre::eyre!(
                    "Parent entity {} not found in deserialization context",
                    serializable.entity_id
                )
            })?;

            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(Parent(parent_entity));
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "Parent"
        }
    }

    #[derive(Serialize, Deserialize)]
    struct SerializableChildren {
        entity_ids: Vec<u64>,
    }

    impl SerializableComponent for Children {
        fn serialize_component(&self) -> Result<String> {
            let serializable = SerializableChildren {
                entity_ids: self.0.iter().map(|e| e.index() as u64).collect(),
            };
            Ok(ron::to_string(&serializable)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let serializable: SerializableChildren = ron::from_str(data)?;
            let mut entities = Vec::new();

            for entity_id in serializable.entity_ids {
                if let Some(entity) = context.get_entity(entity_id) {
                    entities.push(entity);
                }
            }

            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(Children(entities));
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "Children"
        }
    }

    impl SerializableComponent for MeshHandle {
        fn serialize_component(&self) -> Result<String> {
            Ok(ron::to_string(self)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let component: MeshHandle = ron::from_str(data)?;
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "MeshHandle"
        }
    }

    impl SerializableComponent for MaterialHandle {
        fn serialize_component(&self) -> Result<String> {
            Ok(ron::to_string(self)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let component: MaterialHandle = ron::from_str(data)?;
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "MaterialHandle"
        }
    }

    impl SerializableComponent for Visibility {
        fn serialize_component(&self) -> Result<String> {
            Ok(ron::to_string(self)?)
        }

        fn deserialize_component(
            data: &str,
            _entity: Entity,
            _context: &DeserializeContext,
        ) -> Result<Box<dyn FnOnce(&mut bevy_ecs::world::EntityWorldMut)>>
        where
            Self: Sized + 'static,
        {
            let component: Visibility = ron::from_str(data)?;
            Ok(Box::new(move |entity_mut| {
                entity_mut.insert(component);
            }))
        }

        fn type_name() -> &'static str
        where
            Self: Sized,
        {
            "Visibility"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Transform};

    #[test]
    fn test_component_registry_creation() {
        let registry = ComponentRegistry::new();
        assert_eq!(registry.registered_count(), 0);
    }

    #[test]
    fn test_world_snapshot_creation() {
        let snapshot = WorldSnapshot::new();
        assert_eq!(snapshot.entity_count(), 0);
    }

    #[test]
    fn test_entity_data_creation() {
        let mut entity_data = EntityData::new(1);
        assert_eq!(entity_data.id, 1);
        assert!(entity_data.components.is_empty());

        entity_data.add_component("TestComponent".to_string(), "test_data".to_string());
        assert_eq!(entity_data.components.len(), 1);
    }

    #[test]
    fn test_deserialize_context() {
        let mut context = DeserializeContext::new();
        let entity = Entity::from_raw(42);

        context.map_entity(1, entity);
        assert_eq!(context.get_entity(1), Some(entity));
        assert_eq!(context.get_entity(999), None);
    }

    #[test]
    fn test_world_snapshot_serialization() {
        let mut snapshot = WorldSnapshot::new();
        let mut entity_data = EntityData::new(1);
        entity_data.add_component("Transform".to_string(), "test_data".to_string());
        snapshot.add_entity(entity_data);

        let ron_string = snapshot.to_ron().unwrap();
        assert!(ron_string.contains("entities"));

        let deserialized = WorldSnapshot::from_ron(&ron_string).unwrap();
        assert_eq!(deserialized.entity_count(), 1);
    }

    #[test]
    fn test_component_registration() {
        let mut registry = ComponentRegistry::new();
        assert_eq!(registry.registered_count(), 0);

        registry.register::<Name>();
        assert_eq!(registry.registered_count(), 1);
        assert!(registry.is_registered::<Name>());

        registry.register::<Transform>();
        assert_eq!(registry.registered_count(), 2);
        assert!(registry.is_registered::<Transform>());
    }

    #[test]
    fn test_full_serialization_workflow() {
        use crate::{MaterialHandle, MeshHandle, Visibility, World};
        use praxis_math::Vec3;

        let mut world = World::new();

        let entity1 = world.spawn((
            Name::new("Player"),
            Transform::from_xyz(1.0, 2.0, 3.0),
            MeshHandle::new("player_mesh"),
            MaterialHandle::new("player_material"),
            Visibility::Visible,
        ));

        let entity2 = world.spawn((
            Name::new("Enemy"),
            Transform::from_xyz(10.0, 0.0, 5.0),
            Visibility::Hidden,
        ));

        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();
        registry.register::<MeshHandle>();
        registry.register::<MaterialHandle>();
        registry.register::<Visibility>();

        let ron_string = registry.serialize_world(&world).unwrap();
        assert!(!ron_string.is_empty());
        assert!(ron_string.contains("entities"));

        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut name_query = new_world.inner_mut().query::<&Name>();
        let names: Vec<String> = name_query
            .iter(new_world.inner())
            .map(|n| n.0.clone())
            .collect();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"Player".to_string()));
        assert!(names.contains(&"Enemy".to_string()));
    }

    #[test]
    fn test_parent_child_serialization() {
        use crate::{Children, Parent, World};

        let mut world = World::new();

        let parent = world.spawn((Name::new("Parent"), Transform::default()));

        let child1 = world.spawn((
            Name::new("Child1"),
            Transform::from_xyz(1.0, 0.0, 0.0),
            Parent(parent),
        ));

        let child2 = world.spawn((
            Name::new("Child2"),
            Transform::from_xyz(-1.0, 0.0, 0.0),
            Parent(parent),
        ));

        world.insert_component(parent, Children(vec![child1, child2]));

        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();
        registry.register::<Parent>();
        registry.register::<Children>();

        let ron_string = registry.serialize_world(&world).unwrap();

        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut parent_query = new_world.inner_mut().query::<(&Name, &Children)>();
        let parents: Vec<_> = parent_query.iter(new_world.inner()).collect();
        assert_eq!(parents.len(), 1);

        let (parent_name, children) = parents[0];
        assert_eq!(parent_name.0, "Parent");
        assert_eq!(children.0.len(), 2);
    }

    #[test]
    fn test_registry_type_name() {
        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();

        assert_eq!(registry.get_type_name::<Name>(), Some("Name"));
        assert_eq!(registry.get_type_name::<Transform>(), Some("Transform"));
    }

    #[test]
    fn test_register_common_types() {
        use crate::{Children, GlobalTransform, MaterialHandle, MeshHandle, Parent, Visibility};

        let mut registry = ComponentRegistry::new();
        assert_eq!(registry.registered_count(), 0);

        registry.register_common_types();

        assert!(registry.is_registered::<Name>());
        assert!(registry.is_registered::<Transform>());
        assert!(registry.is_registered::<GlobalTransform>());
        assert!(registry.is_registered::<Parent>());
        assert!(registry.is_registered::<Children>());
        assert!(registry.is_registered::<MeshHandle>());
        assert!(registry.is_registered::<MaterialHandle>());
        assert!(registry.is_registered::<Visibility>());

        assert_eq!(registry.registered_count(), 8);
    }

    #[test]
    fn test_nosave_entities_excluded() {
        use crate::{NoSave, World};

        let mut world = World::new();

        world.spawn((Name::new("Saved"), Transform::default()));

        world.spawn((Name::new("NotSaved"), Transform::default(), NoSave));

        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();

        let ron_string = registry.serialize_world(&world).unwrap();

        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut name_query = new_world.inner_mut().query::<&Name>();
        let names: Vec<String> = name_query
            .iter(new_world.inner())
            .map(|n| n.0.clone())
            .collect();

        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "Saved");
    }

    #[test]
    fn test_world_convenience_methods() {
        use crate::{MaterialHandle, MeshHandle, Visibility, World};

        let mut world = World::new();
        world.spawn((
            Name::new("TestEntity"),
            Transform::from_xyz(5.0, 10.0, 15.0),
            MeshHandle::new("test_mesh"),
            MaterialHandle::new("test_material"),
            Visibility::Visible,
        ));

        let mut registry = ComponentRegistry::new();
        registry.register_common_types();

        let ron_string = world.serialize(&registry).unwrap();

        let mut new_world = World::new();
        new_world.deserialize(&ron_string, &registry).unwrap();

        let mut query = new_world.inner_mut().query::<&Name>();
        let names: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].0, "TestEntity");
    }

    #[test]
    fn test_complex_hierarchy_serialization() {
        use crate::{Children, Parent, World};

        let mut world = World::new();

        let root = world.spawn((Name::new("Root"), Transform::default()));

        let child1 = world.spawn((
            Name::new("Child1"),
            Transform::from_xyz(1.0, 0.0, 0.0),
            Parent(root),
        ));

        let child2 = world.spawn((
            Name::new("Child2"),
            Transform::from_xyz(-1.0, 0.0, 0.0),
            Parent(root),
        ));

        let grandchild = world.spawn((
            Name::new("Grandchild"),
            Transform::from_xyz(0.0, 1.0, 0.0),
            Parent(child1),
        ));

        world.insert_component(root, Children(vec![child1, child2]));
        world.insert_component(child1, Children(vec![grandchild]));

        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();
        registry.register::<Parent>();
        registry.register::<Children>();

        let ron_string = registry.serialize_world(&world).unwrap();

        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut name_query = new_world.inner_mut().query::<&Name>();
        let names: Vec<_> = name_query
            .iter(new_world.inner())
            .map(|n| n.0.as_str())
            .collect();

        assert_eq!(names.len(), 4);
        assert!(names.contains(&"Root"));
        assert!(names.contains(&"Child1"));
        assert!(names.contains(&"Child2"));
        assert!(names.contains(&"Grandchild"));

        let mut parent_query = new_world.inner_mut().query::<(&Name, &Children)>();
        let mut child_counts: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        for (name, children) in parent_query.iter(new_world.inner()) {
            child_counts.insert(name.0.clone(), children.0.len());
        }

        assert_eq!(child_counts.get("Root"), Some(&2));
        assert_eq!(child_counts.get("Child1"), Some(&1));
    }

    #[test]
    fn test_empty_world_serialization() {
        use crate::World;

        let world = World::new();
        let registry = ComponentRegistry::new();

        let ron_string = registry.serialize_world(&world).unwrap();
        assert!(ron_string.contains("entities"));

        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        assert_eq!(new_world.entity_count(), 0);
    }

    #[test]
    fn test_ron_format_readability() {
        use crate::World;

        let mut world = World::new();
        world.spawn((Name::new("Player"), Transform::from_xyz(1.0, 2.0, 3.0)));

        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();

        let ron_string = registry.serialize_world(&world).unwrap();

        assert!(ron_string.contains("Player"));
        assert!(ron_string.contains("entities"));
        assert!(ron_string.contains("components"));
    }

    #[test]
    #[cfg(all(feature = "test_physics", feature = "test_audio"))]
    fn test_rigidbody_serialization_roundtrip() {
        use praxis_physics::RigidBody;

        let mut registry = ComponentRegistry::new();
        registry.register::<RigidBody>();

        // Test Dynamic
        let mut world = World::new();
        world.spawn(RigidBody::Dynamic);
        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&RigidBody>();
        let bodies: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(bodies.len(), 1);
        assert_eq!(*bodies[0], RigidBody::Dynamic);

        // Test Static
        let mut world = World::new();
        world.spawn(RigidBody::Static);
        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&RigidBody>();
        let bodies: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(bodies.len(), 1);
        assert_eq!(*bodies[0], RigidBody::Static);

        // Test Kinematic
        let mut world = World::new();
        world.spawn(RigidBody::Kinematic);
        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&RigidBody>();
        let bodies: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(bodies.len(), 1);
        assert_eq!(*bodies[0], RigidBody::Kinematic);
    }

    #[test]
    #[cfg(all(feature = "test_physics", feature = "test_audio"))]
    fn test_collider_serialization_roundtrip() {
        use praxis_physics::Collider;

        let mut registry = ComponentRegistry::new();
        registry.register::<Collider>();

        // Test Cuboid
        let mut world = World::new();
        world.spawn(Collider::cuboid(1.0, 2.0, 3.0));
        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&Collider>();
        let colliders: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(colliders.len(), 1);
        if let Collider::Cuboid { hx, hy, hz } = colliders[0] {
            assert_eq!(*hx, 1.0);
            assert_eq!(*hy, 2.0);
            assert_eq!(*hz, 3.0);
        } else {
            panic!("Expected Cuboid collider");
        }

        // Test Sphere
        let mut world = World::new();
        world.spawn(Collider::sphere(0.5));
        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&Collider>();
        let colliders: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(colliders.len(), 1);
        if let Collider::Sphere { radius } = colliders[0] {
            assert_eq!(*radius, 0.5);
        } else {
            panic!("Expected Sphere collider");
        }

        // Test CapsuleY
        let mut world = World::new();
        world.spawn(Collider::capsule_y(1.0, 0.25));
        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&Collider>();
        let colliders: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(colliders.len(), 1);
        if let Collider::CapsuleY {
            half_height,
            radius,
        } = colliders[0]
        {
            assert_eq!(*half_height, 1.0);
            assert_eq!(*radius, 0.25);
        } else {
            panic!("Expected CapsuleY collider");
        }
    }

    #[test]
    #[cfg(all(feature = "test_physics", feature = "test_audio"))]
    fn test_audiosource_serialization_roundtrip() {
        use praxis_audio::AudioSource;

        let mut registry = ComponentRegistry::new();
        registry.register::<AudioSource>();

        let mut world = World::new();
        let source = AudioSource::new("assets/sounds/test.ogg")
            .with_volume(0.75)
            .with_spatial(true)
            .with_looping(true)
            .with_max_distance(50.0)
            .with_reference_distance(2.0)
            .with_doppler(true)
            .with_doppler_scale(1.5);

        world.spawn(source);

        let ron_string = registry.serialize_world(&world).unwrap();
        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        let mut query = new_world.inner_mut().query::<&AudioSource>();
        let sources: Vec<_> = query.iter(new_world.inner()).collect();
        assert_eq!(sources.len(), 1);

        let deserialized = sources[0];
        assert_eq!(deserialized.path(), "assets/sounds/test.ogg");
        assert_eq!(deserialized.volume(), 0.75);
        assert_eq!(deserialized.is_spatial(), true);
        assert_eq!(deserialized.is_looping(), true);
        assert_eq!(deserialized.max_distance(), 50.0);
        assert_eq!(deserialized.reference_distance(), 2.0);
        assert_eq!(deserialized.doppler_enabled, true);
        assert_eq!(deserialized.doppler_scale, 1.5);

        // Verify internal fields are reset to None
        assert!(deserialized.sound_handle.is_none());
        assert!(deserialized.previous_position.is_none());
    }

    #[test]
    #[cfg(all(feature = "test_physics", feature = "test_audio"))]
    fn test_physics_and_audio_combined_serialization() {
        use praxis_audio::AudioSource;
        use praxis_physics::{Collider, RigidBody};

        let mut world = World::new();

        // Create an entity with physics and audio components
        world.spawn((
            Name::new("SoundEmitter"),
            Transform::from_xyz(5.0, 10.0, 15.0),
            RigidBody::Dynamic,
            Collider::sphere(1.0),
            AudioSource::new("assets/sounds/bounce.ogg")
                .with_volume(0.8)
                .with_spatial(true),
        ));

        let mut registry = ComponentRegistry::new();
        registry.register::<Name>();
        registry.register::<Transform>();
        registry.register::<RigidBody>();
        registry.register::<Collider>();
        registry.register::<AudioSource>();

        let ron_string = registry.serialize_world(&world).unwrap();

        let mut new_world = World::new();
        registry
            .deserialize_world(&ron_string, &mut new_world)
            .unwrap();

        // Verify all components were deserialized
        let mut name_query = new_world.inner_mut().query::<&Name>();
        let names: Vec<_> = name_query.iter(new_world.inner()).collect();
        assert_eq!(names.len(), 1);
        assert_eq!(names[0].0, "SoundEmitter");

        let mut body_query = new_world.inner_mut().query::<&RigidBody>();
        let bodies: Vec<_> = body_query.iter(new_world.inner()).collect();
        assert_eq!(bodies.len(), 1);
        assert_eq!(*bodies[0], RigidBody::Dynamic);

        let mut collider_query = new_world.inner_mut().query::<&Collider>();
        let colliders: Vec<_> = collider_query.iter(new_world.inner()).collect();
        assert_eq!(colliders.len(), 1);

        let mut audio_query = new_world.inner_mut().query::<&AudioSource>();
        let sources: Vec<_> = audio_query.iter(new_world.inner()).collect();
        assert_eq!(sources.len(), 1);
        assert_eq!(sources[0].path(), "assets/sounds/bounce.ogg");
    }
}
