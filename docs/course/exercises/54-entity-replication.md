# Exercise 54: Network Entity Replication

**Difficulty**: 🔴 Advanced | **Estimated Time**: 6-8h | **Subsystem**: Networking

## Overview

Implement automatic replication of entities and components across network clients in a multiplayer game. Foundation for networked gameplay.

## Learning Objectives

- Understand client-server architecture
- Learn state synchronization patterns
- Implement delta compression
- Handle network ownership

## Requirements

### Functional Requirements

1. **Replication Registry**
   - Register which components should replicate
   - Mark entities as replicated
   - Assign unique network IDs

2. **State Serialization**
   - Serialize component state to bytes
   - Deserialize on remote client
   - Handle multiple component types

3. **Replication Modes**
   - Full state: Send complete state
   - Delta: Send only changed values
   - Relevancy: Only replicate to relevant clients

4. **Network Messages**
   - Entity spawn message
   - Entity destroy message
   - Component update message
   - Batch multiple updates

### Non-Functional Requirements

- **Performance**: Replicate 100 entities at 20 Hz
- **Bandwidth**: < 10 KB/s for 100 entities
- **Reliability**: No missed updates for important state

## API Design

```rust
pub struct ReplicationRegistry {
    replicated_components: HashMap<TypeId, ComponentReplicator>,
}

pub trait ComponentReplicator {
    fn serialize(&self, component: &dyn Any, writer: &mut dyn Write) -> Result<()>;
    fn deserialize(&self, reader: &mut dyn Read) -> Result<Box<dyn Any>>;
    fn compare(&self, old: &dyn Any, new: &dyn Any) -> bool; // For delta
}

pub struct NetworkServer {
    registry: ReplicationRegistry,
    client_connections: Vec<ClientConnection>,
}

impl NetworkServer {
    pub fn spawn_replicated_entity(&mut self, entity: Entity, components: Vec<Box<dyn Any>>);
    pub fn despawn_replicated_entity(&mut self, entity: Entity);
    pub fn replicate_updates(&mut self, world: &World);
}

pub struct NetworkClient {
    registry: ReplicationRegistry,
    server_connection: ServerConnection,
}

impl NetworkClient {
    pub fn process_replication_messages(&mut self, world: &mut World);
}
```

## Validation Criteria

### Correctness
- [ ] Entities spawn on all clients
- [ ] Component updates arrive and apply
- [ ] Entity destruction replicates
- [ ] No desyncs under normal conditions

### Performance
- [ ] 100 entities @ 20 Hz replication
- [ ] Bandwidth < 10 KB/s
- [ ] Latency < 100ms for updates

## Test Cases

```rust
#[test]
fn test_entity_spawn_replication() {
    let mut server = NetworkServer::new();
    let mut client = NetworkClient::new();
    
    // Server spawns entity
    let entity = server.world.spawn();
    server.world.add_component(entity, Transform::default());
    server.spawn_replicated_entity(entity);
    
    // Process messages on client
    client.process_replication_messages();
    
    // Client should have the entity
    assert_eq!(client.world.entity_count(), 1);
}

#[test]
fn test_component_update_replication() {
    let mut server = NetworkServer::new();
    let mut client = NetworkClient::new();
    
    // Spawn and replicate entity
    let entity = server.spawn_entity_with_transform();
    
    // Update transform on server
    if let Some(transform) = server.world.get_component_mut::<Transform>(entity) {
        transform.position = Vec3::new(10.0, 0.0, 0.0);
    }
    
    server.replicate_updates();
    client.process_replication_messages();
    
    // Client should have updated position
    let client_entity = client.get_replicated_entity(entity);
    let transform = client.world.get_component::<Transform>(client_entity).unwrap();
    assert_eq!(transform.position, Vec3::new(10.0, 0.0, 0.0));
}

#[test]
fn test_delta_compression() {
    let mut replicator = TransformReplicator::new();
    
    let old = Transform {
        position: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let new = Transform {
        position: Vec3::new(1.0, 0.0, 0.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    };
    
    let mut full_bytes = Vec::new();
    replicator.serialize_full(&new, &mut full_bytes).unwrap();
    
    let mut delta_bytes = Vec::new();
    replicator.serialize_delta(&old, &new, &mut delta_bytes).unwrap();
    
    assert!(delta_bytes.len() < full_bytes.len());
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| 10 entities @ 60 Hz | < 5 KB/s |
| 100 entities @ 20 Hz | < 10 KB/s |
| 1000 entities @ 10 Hz | < 50 KB/s |
| Update latency | < 100ms |

## Hints & Guidance

### Network Messages
```rust
enum NetworkMessage {
    SpawnEntity {
        net_id: u32,
        entity_type: String,
        components: Vec<u8>,
    },
    DestroyEntity {
        net_id: u32,
    },
    UpdateComponents {
        net_id: u32,
        component_data: Vec<u8>,
    },
}
```

### Delta Compression
Only send fields that changed:
```rust
// Instead of sending all 12 floats of transform,
// send a bitmask + only changed fields:
// Bit 0: Position changed
// Bit 1: Rotation changed
// Bit 2: Scale changed
```

### Relevancy Filtering
Don't replicate to clients who can't see it:
```rust
fn is_relevant_to_client(entity_pos: Vec3, client_pos: Vec3) -> bool {
    entity_pos.distance(client_pos) < VIEW_DISTANCE
}
```

## Reference Implementation

### Rust (Primary)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use std::collections::HashMap;
use std::io::{Read, Write};
use glam::{Vec3, Quat};

pub type NetworkId = u32;
pub type EntityId = u32;

#[derive(Clone, Copy)]
pub struct Transform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        // Write position
        writer.write_all(&self.position.x.to_le_bytes())?;
        writer.write_all(&self.position.y.to_le_bytes())?;
        writer.write_all(&self.position.z.to_le_bytes())?;
        
        // Write rotation
        writer.write_all(&self.rotation.x.to_le_bytes())?;
        writer.write_all(&self.rotation.y.to_le_bytes())?;
        writer.write_all(&self.rotation.z.to_le_bytes())?;
        writer.write_all(&self.rotation.w.to_le_bytes())?;
        
        // Write scale
        writer.write_all(&self.scale.x.to_le_bytes())?;
        writer.write_all(&self.scale.y.to_le_bytes())?;
        writer.write_all(&self.scale.z.to_le_bytes())?;
        
        Ok(())
    }
    
    pub fn deserialize(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut buf = [0u8; 4];
        
        // Read position
        reader.read_exact(&mut buf)?;
        let px = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let py = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let pz = f32::from_le_bytes(buf);
        
        // Read rotation
        reader.read_exact(&mut buf)?;
        let rx = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let ry = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let rz = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let rw = f32::from_le_bytes(buf);
        
        // Read scale
        reader.read_exact(&mut buf)?;
        let sx = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let sy = f32::from_le_bytes(buf);
        reader.read_exact(&mut buf)?;
        let sz = f32::from_le_bytes(buf);
        
        Ok(Transform {
            position: Vec3::new(px, py, pz),
            rotation: Quat::from_xyzw(rx, ry, rz, rw),
            scale: Vec3::new(sx, sy, sz),
        })
    }
    
    pub fn serialize_delta(&self, old: &Transform, writer: &mut impl Write) 
        -> std::io::Result<()> 
    {
        let mut flags = 0u8;
        
        // Check what changed
        if self.position != old.position { flags |= 0b001; }
        if self.rotation != old.rotation { flags |= 0b010; }
        if self.scale != old.scale { flags |= 0b100; }
        
        writer.write_all(&[flags])?;
        
        // Write only changed fields
        if flags & 0b001 != 0 {
            writer.write_all(&self.position.x.to_le_bytes())?;
            writer.write_all(&self.position.y.to_le_bytes())?;
            writer.write_all(&self.position.z.to_le_bytes())?;
        }
        
        if flags & 0b010 != 0 {
            writer.write_all(&self.rotation.x.to_le_bytes())?;
            writer.write_all(&self.rotation.y.to_le_bytes())?;
            writer.write_all(&self.rotation.z.to_le_bytes())?;
            writer.write_all(&self.rotation.w.to_le_bytes())?;
        }
        
        if flags & 0b100 != 0 {
            writer.write_all(&self.scale.x.to_le_bytes())?;
            writer.write_all(&self.scale.y.to_le_bytes())?;
            writer.write_all(&self.scale.z.to_le_bytes())?;
        }
        
        Ok(())
    }
    
    pub fn deserialize_delta(base: &Transform, reader: &mut impl Read) 
        -> std::io::Result<Self> 
    {
        let mut flags_buf = [0u8; 1];
        reader.read_exact(&mut flags_buf)?;
        let flags = flags_buf[0];
        
        let mut result = *base;
        let mut buf = [0u8; 4];
        
        // Read changed fields
        if flags & 0b001 != 0 {
            reader.read_exact(&mut buf)?;
            result.position.x = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.position.y = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.position.z = f32::from_le_bytes(buf);
        }
        
        if flags & 0b010 != 0 {
            reader.read_exact(&mut buf)?;
            result.rotation.x = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.rotation.y = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.rotation.z = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.rotation.w = f32::from_le_bytes(buf);
        }
        
        if flags & 0b100 != 0 {
            reader.read_exact(&mut buf)?;
            result.scale.x = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.scale.y = f32::from_le_bytes(buf);
            reader.read_exact(&mut buf)?;
            result.scale.z = f32::from_le_bytes(buf);
        }
        
        Ok(result)
    }
}

pub enum ReplicationMessage {
    SpawnEntity {
        net_id: NetworkId,
        transform: Transform,
    },
    DestroyEntity {
        net_id: NetworkId,
    },
    UpdateTransform {
        net_id: NetworkId,
        transform: Transform,
    },
}

impl ReplicationMessage {
    pub fn serialize(&self, writer: &mut impl Write) -> std::io::Result<()> {
        match self {
            ReplicationMessage::SpawnEntity { net_id, transform } => {
                writer.write_all(&[0u8])?; // Type tag
                writer.write_all(&net_id.to_le_bytes())?;
                transform.serialize(writer)?;
            }
            ReplicationMessage::DestroyEntity { net_id } => {
                writer.write_all(&[1u8])?;
                writer.write_all(&net_id.to_le_bytes())?;
            }
            ReplicationMessage::UpdateTransform { net_id, transform } => {
                writer.write_all(&[2u8])?;
                writer.write_all(&net_id.to_le_bytes())?;
                transform.serialize(writer)?;
            }
        }
        Ok(())
    }
    
    pub fn deserialize(reader: &mut impl Read) -> std::io::Result<Self> {
        let mut type_buf = [0u8; 1];
        reader.read_exact(&mut type_buf)?;
        
        let mut id_buf = [0u8; 4];
        reader.read_exact(&mut id_buf)?;
        let net_id = u32::from_le_bytes(id_buf);
        
        match type_buf[0] {
            0 => {
                let transform = Transform::deserialize(reader)?;
                Ok(ReplicationMessage::SpawnEntity { net_id, transform })
            }
            1 => {
                Ok(ReplicationMessage::DestroyEntity { net_id })
            }
            2 => {
                let transform = Transform::deserialize(reader)?;
                Ok(ReplicationMessage::UpdateTransform { net_id, transform })
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid message type"
            ))
        }
    }
}

pub struct NetworkServer {
    entities: HashMap<NetworkId, (EntityId, Transform)>,
    next_net_id: NetworkId,
}

impl NetworkServer {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_net_id: 1,
        }
    }
    
    pub fn spawn_entity(&mut self, entity_id: EntityId, transform: Transform) -> NetworkId {
        let net_id = self.next_net_id;
        self.next_net_id += 1;
        
        self.entities.insert(net_id, (entity_id, transform));
        net_id
    }
    
    pub fn despawn_entity(&mut self, net_id: NetworkId) {
        self.entities.remove(&net_id);
    }
    
    pub fn update_entity(&mut self, net_id: NetworkId, transform: Transform) {
        if let Some(entry) = self.entities.get_mut(&net_id) {
            entry.1 = transform;
        }
    }
    
    pub fn collect_messages(&self) -> Vec<ReplicationMessage> {
        // In real implementation, track dirty entities and only send changes
        self.entities.iter().map(|(net_id, (_, transform))| {
            ReplicationMessage::UpdateTransform {
                net_id: *net_id,
                transform: *transform,
            }
        }).collect()
    }
}

pub struct NetworkClient {
    entities: HashMap<NetworkId, (EntityId, Transform)>,
}

impl NetworkClient {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
        }
    }
    
    pub fn process_message(&mut self, msg: ReplicationMessage) {
        match msg {
            ReplicationMessage::SpawnEntity { net_id, transform } => {
                // In real implementation, create entity in ECS
                let entity_id = net_id; // Placeholder
                self.entities.insert(net_id, (entity_id, transform));
            }
            ReplicationMessage::DestroyEntity { net_id } => {
                self.entities.remove(&net_id);
            }
            ReplicationMessage::UpdateTransform { net_id, transform } => {
                if let Some(entry) = self.entities.get_mut(&net_id) {
                    entry.1 = transform;
                }
            }
        }
    }
}
```

</details>

## Related Resources

- [Praxis Networking Documentation](../../reference/crates.md#praxis_networking)
- [Networked Physics (Gaffer on Games)](https://gafferongames.com/post/networked_physics_2004/)
- [Source Multiplayer Networking](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking)

## Next Steps

- Implement lag compensation (Exercise 55)
- Add client prediction
- Implement snapshot interpolation
