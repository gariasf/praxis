//! Entity replication system.
//!
//! # Entity Replication Overview
//!
//! Entity replication synchronizes game objects across the network. When entities
//! move, change state, or perform actions, those changes need to be communicated
//! to all clients so everyone sees the same game world.
//!
//! # How Replication Works
//!
//! ## Server-Side (Authority)
//!
//! 1. **Mark entities for replication**: Add `Replicated` component
//! 2. **Register components**: Tell system which components to sync
//! 3. **Create snapshots**: Serialize entity state each frame
//! 4. **Send to clients**: Broadcast state updates via network
//!
//! ```text
//! Entity { Transform, Health, NetworkId }
//!         ↓
//! Snapshot { network_id: 42, components: { Transform, Health } }
//!         ↓
//! Network Message → All Clients
//! ```
//!
//! ## Client-Side (Receiver)
//!
//! 1. **Receive snapshot**: Get entity state from server
//! 2. **Find/spawn entity**: Look up by NetworkId or create new
//! 3. **Apply components**: Deserialize and update components
//! 4. **Render**: ECS systems render updated state
//!
//! # Component Registration
//!
//! Not all components need to be replicated. Common replicated components:
//! - **Transform**: Position, rotation, scale
//! - **Velocity**: Movement speed and direction
//! - **Health**: Current health points
//! - **Animation state**: Current animation and frame
//!
//! Non-replicated (local-only) components:
//! - **Rendering**: Mesh, materials (loaded from assets)
//! - **Input**: Player input state (only on owning client)
//! - **AI state**: Server-only game logic
//!
//! ## Registration Example
//!
//! ```rust,ignore
//! let mut registry = ReplicationRegistry::new();
//!
//! // Register built-in components
//! registry.register_transform();
//! registry.register_velocity();
//!
//! // Register custom component
//! registry.register::<Health>("Health");
//! ```
//!
//! # Serialization Flow
//!
//! Components must implement `Serialize` and `Deserialize`:
//!
//! ```text
//! Server:
//! Component → Serialize → Bytes → Network
//!
//! Client:
//! Network → Bytes → Deserialize → Component
//! ```
//!
//! The `ComponentSerializer` trait handles this automatically for types
//! that implement serde's traits.
//!
//! # NetworkId: Tracking Entities Across the Network
//!
//! Each replicated entity has a unique `NetworkId`:
//! - **Server assigns** on entity spawn
//! - **Clients use** to match snapshots to entities
//! - **Stable across** entity creation/destruction
//!
//! Example:
//! ```text
//! Server spawns entity with NetworkId(42)
//! Server sends snapshot { network_id: 42, transform: ... }
//! Client receives, finds or creates entity with NetworkId(42)
//! Client applies transform to that entity
//! ```
//!
//! # Replication Rate Control
//!
//! Not all entities need to be replicated every frame:
//!
//! - **rate_divisor**: Replicate every Nth tick
//!   - `rate_divisor = 1`: Every frame (default, high bandwidth)
//!   - `rate_divisor = 2`: Every other frame (50% bandwidth)
//!   - `rate_divisor = 4`: Every 4th frame (25% bandwidth)
//!
//! Use higher rate divisors for:
//! - Far away entities (less noticeable updates)
//! - Slow-moving objects (don't need frequent updates)
//! - Lower priority entities (background props)
//!
//! Use low rate divisors (1-2) for:
//! - Nearby players (need smooth movement)
//! - Fast-moving projectiles
//! - Important gameplay objects
//!
//! # Bandwidth Optimization
//!
//! Replication can consume significant bandwidth. Optimizations:
//!
//! 1. **Delta compression**: Only send changed components (not implemented yet)
//! 2. **Interest management**: Only replicate visible entities (not implemented yet)
//! 3. **Rate limiting**: Reduce update frequency for less important entities
//! 4. **Quantization**: Reduce precision for positions (e.g., 2 decimals instead of 6)
//!
//! # Example: Full Replication Flow
//!
//! ```rust,ignore
//! // Server setup
//! let mut world = World::new();
//! let mut system = ReplicationSystem::new();
//! system.registry().register_transform();
//!
//! // Spawn replicated entity
//! world.spawn((
//!     NetworkId::new(1),
//!     Replicated::new(),
//!     ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
//! ));
//!
//! // Each frame: create and send snapshot
//! let message = system.create_replication_message(&mut world);
//! server.broadcast(&message);
//!
//! // Client receives and applies
//! system.apply_replication_message(&mut world, &message);
//! ```

use crate::{
    ComponentData, EntitySnapshot, NetworkId, Replicated, ReplicatedTransform, ReplicatedVelocity,
    ReplicationMessage,
};
use bevy_ecs::prelude::*;
use parking_lot::RwLock;
use praxis_utils::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Component serialization trait.
pub trait ComponentSerializer: Send + Sync {
    /// Serializes a component to bytes.
    fn serialize(&self, world: &bevy_ecs::world::World, entity: Entity) -> Option<ComponentData>;

    /// Deserializes and applies component data to an entity.
    fn deserialize(
        &self,
        world: &mut bevy_ecs::world::World,
        entity: Entity,
        data: &ComponentData,
    ) -> Result<()>;
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

impl<T: Component + Serialize + for<'de> Deserialize<'de> + Clone> ComponentSerializer
    for GenericSerializer<T>
{
    fn serialize(&self, world: &bevy_ecs::world::World, entity: Entity) -> Option<ComponentData> {
        world
            .get::<T>(entity)
            .and_then(|component| ComponentData::from_serializable(component).ok())
    }

    fn deserialize(
        &self,
        world: &mut bevy_ecs::world::World,
        entity: Entity,
        data: &ComponentData,
    ) -> Result<()> {
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
    ///
    /// # Panics
    ///
    /// Panics if the system time is before the UNIX epoch. This should never happen
    /// on systems with correctly configured clocks.
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
        let mut query =
            world.query_filtered::<(Entity, &NetworkId, &Replicated), With<Replicated>>();

        for (entity, _network_id, replicated) in query.iter(world) {
            if !replicated.enabled {
                continue;
            }

            // Check rate divisor
            if replicated.rate_divisor > 1
                && self.tick_counter % replicated.rate_divisor as u64 != 0
            {
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
    use praxis_math::{Quat, Vec3};

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
        let entity = world
            .spawn((
                NetworkId::new(1),
                ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
            ))
            .id();

        let mut registry = ReplicationRegistry::new();
        registry.register_transform();

        let replicator = EntityReplicator::new(Arc::new(registry));
        let snapshot = replicator.snapshot_entity(&world, entity);

        assert!(snapshot.is_some());
        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.network_id, 1);
    }
}
