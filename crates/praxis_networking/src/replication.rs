//! Entity replication system.

use crate::{
    NetworkId, Replicated, ReplicatedTransform, ReplicatedVelocity, 
    EntitySnapshot, ComponentData, ReplicationMessage,
};
use bevy_ecs::prelude::*;
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::sync::Arc;
use praxis_utils::Result;

/// Component serialization trait.
pub trait ComponentSerializer: Send + Sync {
    /// Serializes a component to bytes.
    fn serialize(&self, world: &bevy_ecs::world::World, entity: Entity) -> Option<ComponentData>;
    
    /// Deserializes and applies component data to an entity.
    fn deserialize(&self, world: &mut bevy_ecs::world::World, entity: Entity, data: &ComponentData) -> Result<()>;
}

/// Generic component serializer for types that implement Serialize/Deserialize.
struct GenericSerializer<T: Component + Serialize + for<'de> Deserialize<'de> + Clone> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T: Component + Serialize + for<'de> Deserialize<'de> + Clone> GenericSerializer<T> {
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Component + Serialize + for<'de> Deserialize<'de> + Clone> ComponentSerializer for GenericSerializer<T> {
    fn serialize(&self, world: &bevy_ecs::world::World, entity: Entity) -> Option<ComponentData> {
        world.get::<T>(entity).and_then(|component| {
            ComponentData::from_serializable(component).ok()
        })
    }
    
    fn deserialize(&self, world: &mut bevy_ecs::world::World, entity: Entity, data: &ComponentData) -> Result<()> {
        let component: T = data.deserialize()?;
        
        if let Some(mut entity_mut) = world.get_entity_mut(entity) {
            entity_mut.insert(component);
        }
        
        Ok(())
    }
}

/// Registry of replicated components.
pub struct ReplicationRegistry {
    /// Map of component names to serializers
    serializers: Arc<RwLock<HashMap<String, Arc<dyn ComponentSerializer>>>>,
}

impl ReplicationRegistry {
    /// Creates a new replication registry.
    pub fn new() -> Self {
        Self {
            serializers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Registers a component for replication.
    pub fn register<T: Component + Serialize + for<'de> Deserialize<'de> + Clone + 'static>(
        &mut self,
        name: impl Into<String>,
    ) {
        let serializer = Arc::new(GenericSerializer::<T>::new());
        self.serializers.write().insert(name.into(), serializer);
    }
    
    /// Registers common transform component.
    pub fn register_transform(&mut self) {
        self.register::<ReplicatedTransform>("Transform");
    }
    
    /// Registers velocity component.
    pub fn register_velocity(&mut self) {
        self.register::<ReplicatedVelocity>("Velocity");
    }
    
    /// Gets a component serializer by name.
    fn get_serializer(&self, name: &str) -> Option<Arc<dyn ComponentSerializer>> {
        self.serializers.read().get(name).cloned()
    }
    
    /// Gets all registered component names.
    pub fn component_names(&self) -> Vec<String> {
        self.serializers.read().keys().cloned().collect()
    }
}

impl Default for ReplicationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Entity replicator handles serialization of entities.
pub struct EntityReplicator {
    registry: Arc<ReplicationRegistry>,
}

impl EntityReplicator {
    /// Creates a new entity replicator.
    pub fn new(registry: Arc<ReplicationRegistry>) -> Self {
        Self { registry }
    }
    
    /// Creates a snapshot of an entity's state.
    pub fn snapshot_entity(
        &self,
        world: &bevy_ecs::world::World,
        entity: Entity,
    ) -> Option<EntitySnapshot> {
        // Get network ID
        let network_id = world.get::<NetworkId>(entity)?;
        
        // Create snapshot
        let mut snapshot = EntitySnapshot::new(network_id.get(), None);
        
        // Serialize all registered components
        for name in self.registry.component_names() {
            if let Some(serializer) = self.registry.get_serializer(&name) {
                if let Some(data) = serializer.serialize(world, entity) {
                    snapshot.add_component(name, data);
                }
            }
        }
        
        Some(snapshot)
    }
    
    /// Applies a snapshot to an entity.
    pub fn apply_snapshot(
        &self,
        world: &mut bevy_ecs::world::World,
        entity: Entity,
        snapshot: &EntitySnapshot,
    ) -> Result<()> {
        for (name, data) in &snapshot.components {
            if let Some(serializer) = self.registry.get_serializer(name) {
                serializer.deserialize(world, entity, data)?;
            }
        }
        
        Ok(())
    }
}

/// Replication system manages entity synchronization.
pub struct ReplicationSystem {
    registry: Arc<ReplicationRegistry>,
    replicator: EntityReplicator,
    tick_counter: u64,
}

impl ReplicationSystem {
    /// Creates a new replication system.
    pub fn new() -> Self {
        let registry = Arc::new(ReplicationRegistry::new());
        let replicator = EntityReplicator::new(Arc::clone(&registry));
        
        Self {
            registry,
            replicator,
            tick_counter: 0,
        }
    }
    
    /// Gets the replication registry.
    pub fn registry(&self) -> Arc<ReplicationRegistry> {
        Arc::clone(&self.registry)
    }
    
    /// Creates a replication message for all replicated entities.
    pub fn create_replication_message(
        &mut self,
        world: &mut bevy_ecs::world::World,
    ) -> ReplicationMessage {
        self.tick_counter += 1;
        
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        
        let mut message = ReplicationMessage::new(self.tick_counter, timestamp);
        
        // Query all replicated entities
        let mut query = world.query_filtered::<(Entity, &NetworkId, &Replicated), With<Replicated>>();
        
        for (entity, _network_id, replicated) in query.iter(world) {
            if !replicated.enabled {
                continue;
            }
            
            // Check rate divisor
            if replicated.rate_divisor > 1 && self.tick_counter % replicated.rate_divisor as u64 != 0 {
                continue;
            }
            
            if let Some(snapshot) = self.replicator.snapshot_entity(world, entity) {
                message.add_entity(snapshot);
            }
        }
        
        message
    }
    
    /// Applies a replication message to the world.
    pub fn apply_replication_message(
        &self,
        world: &mut bevy_ecs::world::World,
        message: &ReplicationMessage,
    ) -> Result<()> {
        // Create a map of network IDs to entities
        let mut network_entities = HashMap::new();
        let mut query = world.query::<(Entity, &NetworkId)>();
        for (entity, network_id) in query.iter(world) {
            network_entities.insert(network_id.get(), entity);
        }
        
        // Apply entity snapshots
        for snapshot in &message.entities {
            let entity = if let Some(&entity) = network_entities.get(&snapshot.network_id) {
                entity
            } else {
                // Spawn new entity
                let new_entity = world.spawn(NetworkId::new(snapshot.network_id)).id();
                network_entities.insert(snapshot.network_id, new_entity);
                new_entity
            };
            
            self.replicator.apply_snapshot(world, entity, snapshot)?;
        }
        
        // Destroy entities
        for &network_id in &message.destroyed_entities {
            if let Some(&entity) = network_entities.get(&network_id) {
                if let Some(entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.despawn();
                }
            }
        }
        
        Ok(())
    }
}

impl Default for ReplicationSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use praxis_math::{Vec3, Quat};

    #[test]
    fn test_replication_registry() {
        let mut registry = ReplicationRegistry::new();
        registry.register_transform();
        registry.register_velocity();
        
        let names = registry.component_names();
        assert!(names.contains(&"Transform".to_string()));
        assert!(names.contains(&"Velocity".to_string()));
    }

    #[test]
    fn test_replication_system_creation() {
        let system = ReplicationSystem::new();
        assert_eq!(system.tick_counter, 0);
    }

    #[test]
    fn test_entity_snapshot() {
        let mut world = bevy_ecs::world::World::new();
        let entity = world.spawn((
            NetworkId::new(1),
            ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
        )).id();
        
        let mut registry = ReplicationRegistry::new();
        registry.register_transform();
        
        let replicator = EntityReplicator::new(Arc::new(registry));
        let snapshot = replicator.snapshot_entity(&world, entity);
        
        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.network_id, 1);
    }
}
