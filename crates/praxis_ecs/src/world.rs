//! World management for the Praxis ECS.
//!
//! This module provides a wrapper around bevy_ecs::World with additional
//! functionality specific to the Praxis engine.

use bevy_ecs::{
    bundle::Bundle,
    component::Component,
    entity::Entity,
    schedule::{Schedule, ScheduleLabel},
    system::Resource,
    world::World as BevyWorld,
};
use praxis_utils::{Result, debug, error, eyre, info, trace};

/// The main ECS world container.
///
/// The World holds all entities, components, and resources in the game.
/// It provides methods for spawning entities, adding/removing components,
/// managing resources, and running systems.
///
/// # Example
///
/// ```rust,no_run
/// use praxis_ecs::{World, Component};
///
/// #[derive(Component)]
/// struct Health(f32);
///
/// let mut world = World::new();
///
/// // Spawn an entity
/// let entity = world.spawn(Health(100.0));
///
/// // Spawn multiple entities
/// let entities = world.spawn_batch(vec![Health(100.0), Health(50.0)]);
/// ```
pub struct World {
    /// The underlying bevy_ecs World
    inner: BevyWorld,

    /// Statistics for debugging
    stats: WorldStats,
}

/// Statistics about the world for debugging and profiling.
#[derive(Debug, Default)]
pub struct WorldStats {
    /// Total number of entities spawned
    pub entities_spawned: u64,

    /// Total number of entities despawned
    pub entities_despawned: u64,

    /// Current number of active entities
    pub active_entities: u32,
}

impl World {
    /// Creates a new, empty World.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::World;
    ///
    /// let mut world = World::new();
    /// ```
    pub fn new() -> Self {
        debug!("Creating new ECS World");

        let inner = BevyWorld::new();

        Self {
            inner,
            stats: WorldStats::default(),
        }
    }

    /// Spawns a new entity with the given bundle of components.
    ///
    /// A bundle can be a single component or a tuple of components.
    ///
    /// # Arguments
    ///
    /// * `bundle` - The components to attach to the new entity
    ///
    /// # Returns
    ///
    /// The Entity ID of the newly spawned entity.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, Component};
    ///
    /// #[derive(Component)]
    /// struct Position { x: f32, y: f32, z: f32 }
    ///
    /// #[derive(Component)]
    /// struct Velocity { x: f32, y: f32, z: f32 }
    ///
    /// let mut world = World::new();
    ///
    /// // Spawn with a single component
    /// let entity1 = world.spawn(Position { x: 0.0, y: 0.0, z: 0.0 });
    ///
    /// // Spawn with multiple components
    /// let entity2 = world.spawn((
    ///     Position { x: 10.0, y: 0.0, z: 0.0 },
    ///     Velocity { x: 1.0, y: 0.0, z: 0.0 },
    /// ));
    /// ```
    pub fn spawn<B: Bundle>(&mut self, bundle: B) -> Entity {
        let entity = self.inner.spawn(bundle).id();

        self.stats.entities_spawned += 1;
        self.stats.active_entities += 1;

        trace!("Spawned entity {:?}", entity);
        entity
    }

    /// Spawns multiple entities with the same component types.
    ///
    /// This is more efficient than calling spawn() multiple times.
    ///
    /// # Arguments
    ///
    /// * `bundles` - An iterator of component bundles
    ///
    /// # Returns
    ///
    /// A vector of Entity IDs for the spawned entities.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, Component};
    ///
    /// #[derive(Component)]
    /// struct Enemy { health: f32 }
    ///
    /// let mut world = World::new();
    ///
    /// let enemies = world.spawn_batch(vec![
    ///     Enemy { health: 100.0 },
    ///     Enemy { health: 50.0 },
    ///     Enemy { health: 75.0 },
    /// ]);
    /// ```
    pub fn spawn_batch<B, I>(&mut self, bundles: I) -> Vec<Entity>
    where
        B: Bundle,
        I: IntoIterator<Item = B>,
    {
        let entities: Vec<Entity> = self.inner.spawn_batch(bundles).collect();

        let count = entities.len() as u64;
        self.stats.entities_spawned += count;
        self.stats.active_entities += count as u32;

        debug!("Spawned batch of {} entities", count);
        entities
    }

    /// Despawns an entity and all its components.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to despawn
    ///
    /// # Returns
    ///
    /// Ok(()) if the entity was despawned, Err if the entity doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::World;
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn(());
    ///
    /// // Later...
    /// world.despawn(entity).expect("Failed to despawn entity");
    /// ```
    pub fn despawn(&mut self, entity: Entity) -> Result<()> {
        if self.inner.despawn(entity) {
            self.stats.entities_despawned += 1;
            self.stats.active_entities = self.stats.active_entities.saturating_sub(1);
            trace!("Despawned entity {:?}", entity);
            Ok(())
        } else {
            error!("Failed to despawn entity {:?} - entity not found", entity);
            Err(eyre::eyre!("Entity {:?} does not exist", entity))
        }
    }

    /// Inserts a global resource into the world.
    ///
    /// Resources are globally accessible data that can be accessed by systems.
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to insert
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, Resource};
    ///
    /// #[derive(Resource)]
    /// struct GameTime {
    ///     elapsed: f32,
    ///     delta: f32,
    /// }
    ///
    /// let mut world = World::new();
    /// world.insert_resource(GameTime {
    ///     elapsed: 0.0,
    ///     delta: 0.016,
    /// });
    /// ```
    pub fn insert_resource<R: Resource>(&mut self, resource: R) {
        let type_name = std::any::type_name::<R>();
        trace!("Inserting resource: {}", type_name);
        self.inner.insert_resource(resource);
    }

    /// Removes a resource from the world.
    ///
    /// # Returns
    ///
    /// Some(resource) if it existed, None otherwise.
    pub fn remove_resource<R: Resource>(&mut self) -> Option<R> {
        let type_name = std::any::type_name::<R>();
        trace!("Removing resource: {}", type_name);
        self.inner.remove_resource::<R>()
    }

    /// Gets a reference to a resource.
    ///
    /// # Returns
    ///
    /// Some(&resource) if it exists, None otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, Resource};
    ///
    /// #[derive(Resource)]
    /// struct Score(u32);
    ///
    /// let mut world = World::new();
    /// world.insert_resource(Score(0));
    ///
    /// if let Some(score) = world.get_resource::<Score>() {
    ///     println!("Current score: {}", score.0);
    /// }
    /// ```
    pub fn get_resource<R: Resource>(&self) -> Option<&R> {
        self.inner.get_resource::<R>()
    }

    /// Gets a mutable reference to a resource.
    ///
    /// # Returns
    ///
    /// Some(&mut resource) if it exists, None otherwise.
    pub fn get_resource_mut<R: Resource>(&mut self) -> Option<&mut R> {
        self.inner
            .get_resource_mut::<R>()
            .map(|res| res.into_inner())
    }

    /// Adds a component to an existing entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to add the component to
    /// * `component` - The component to add
    ///
    /// # Returns
    ///
    /// Ok(()) if successful, Err if the entity doesn't exist.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, Component};
    ///
    /// #[derive(Component)]
    /// struct Name(String);
    ///
    /// let mut world = World::new();
    /// let entity = world.spawn(());
    ///
    /// world.insert_component(entity, Name("Player".to_string()))
    ///     .expect("Failed to add component");
    /// ```
    pub fn insert_component<C: Component>(&mut self, entity: Entity, component: C) -> Result<()> {
        if let Some(mut entity_mut) = self.inner.get_entity_mut(entity) {
            entity_mut.insert(component);
            trace!(
                "Added component {} to entity {:?}",
                std::any::type_name::<C>(),
                entity
            );
            Ok(())
        } else {
            error!(
                "Failed to add component to entity {:?} - entity not found",
                entity
            );
            Err(eyre::eyre!("Entity {:?} does not exist", entity))
        }
    }

    /// Removes a component from an entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - The entity to remove the component from
    ///
    /// # Returns
    ///
    /// Some(component) if it was removed, None if the entity doesn't exist or doesn't have the component.
    pub fn remove_component<C: Component>(&mut self, entity: Entity) -> Option<C> {
        let mut entity_mut = self.inner.get_entity_mut(entity)?;
        entity_mut.take::<C>()
    }

    /// Gets an immutable reference to the underlying bevy World.
    ///
    /// This is useful when you need to pass the world to bevy_ecs functions.
    pub fn inner(&self) -> &BevyWorld {
        &self.inner
    }

    /// Gets a mutable reference to the underlying bevy World.
    ///
    /// # Safety
    ///
    /// Be careful when using this as it bypasses the wrapper's statistics tracking.
    pub fn inner_mut(&mut self) -> &mut BevyWorld {
        &mut self.inner
    }

    /// Runs a schedule of systems.
    ///
    /// # Arguments
    ///
    /// * `label` - The schedule to run
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use praxis_ecs::{World, ScheduleLabel};
    ///
    /// #[derive(ScheduleLabel, Debug, Clone, PartialEq, Eq, Hash)]
    /// struct Update;
    ///
    /// let mut world = World::new();
    /// world.run_schedule(Update);
    /// ```
    pub fn run_schedule(&mut self, label: impl ScheduleLabel) {
        self.inner.run_schedule(label);
    }

    /// Adds a schedule to the world.
    ///
    /// # Arguments
    ///
    /// * `schedule` - The schedule to add (must already have a label)
    pub fn add_schedule(&mut self, schedule: Schedule) {
        self.inner.add_schedule(schedule);
    }

    /// Returns statistics about the world.
    pub fn stats(&self) -> &WorldStats {
        &self.stats
    }

    /// Returns the number of entities currently in the world.
    pub fn entity_count(&self) -> u32 {
        self.inner.entities().len() as u32
    }

    /// Clears all entities from the world.
    ///
    /// This is useful for level transitions or resets.
    pub fn clear_entities(&mut self) {
        info!("Clearing all entities from world");

        // We need to query all entities to collect them
        let mut all_entities = self.inner.query::<Entity>();
        let entities: Vec<Entity> = all_entities.iter(&self.inner).collect();

        // Then despawn them
        for entity in entities {
            let _ = self.despawn(entity);
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Component)]
    struct TestComponent(i32);

    #[derive(Resource)]
    struct TestResource(String);

    #[test]
    fn test_world_creation() {
        let world = World::new();
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.stats().active_entities, 0);
    }

    #[test]
    fn test_entity_spawning() {
        let mut world = World::new();

        let entity = world.spawn(TestComponent(42));
        assert_eq!(world.entity_count(), 1);
        assert_eq!(world.stats().entities_spawned, 1);
        assert_eq!(world.stats().active_entities, 1);

        // Despawn the entity
        world.despawn(entity).expect("Failed to despawn");
        assert_eq!(world.entity_count(), 0);
        assert_eq!(world.stats().entities_despawned, 1);
        assert_eq!(world.stats().active_entities, 0);
    }

    #[test]
    fn test_batch_spawning() {
        let mut world = World::new();

        let entities =
            world.spawn_batch(vec![TestComponent(1), TestComponent(2), TestComponent(3)]);

        assert_eq!(entities.len(), 3);
        assert_eq!(world.entity_count(), 3);
        assert_eq!(world.stats().entities_spawned, 3);
    }

    #[test]
    fn test_resource_management() {
        let mut world = World::new();

        // Insert resource
        world.insert_resource(TestResource("Hello".to_string()));

        // Get resource
        let resource = world.get_resource::<TestResource>();
        assert!(resource.is_some());
        assert_eq!(resource.unwrap().0, "Hello");

        // Remove resource
        let removed = world.remove_resource::<TestResource>();
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().0, "Hello");

        // Resource should be gone
        assert!(world.get_resource::<TestResource>().is_none());
    }
}
