//! Network message protocol.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Network message types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Client connecting to server
    Connect {
        protocol_version: u32,
        client_name: String,
    },
    
    /// Server accepting client connection
    ConnectionAccepted {
        client_id: u64,
        server_tick: u64,
    },
    
    /// Connection rejected
    ConnectionRejected {
        reason: String,
    },
    
    /// Client disconnecting
    Disconnect {
        reason: String,
    },
    
    /// Heartbeat/keepalive
    Ping {
        timestamp: u64,
    },
    
    /// Response to ping
    Pong {
        timestamp: u64,
    },
    
    /// Entity replication update
    Replication(ReplicationMessage),
    
    /// Client input command
    ClientCommand {
        tick: u64,
        command_data: Vec<u8>,
    },
    
    /// Server acknowledgment of client command
    CommandAck {
        tick: u64,
    },
    
    /// Custom game message
    GameMessage {
        message_type: u32,
        data: Vec<u8>,
    },
}

/// Message type discriminator for bandwidth tracking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageType {
    /// Connection management
    Connection,
    /// Heartbeat
    Ping,
    /// Entity replication
    Replication,
    /// Client commands
    Command,
    /// Game-specific
    Game,
}

impl NetworkMessage {
    /// Gets the message type for categorization.
    pub fn message_type(&self) -> MessageType {
        match self {
            Self::Connect { .. }
            | Self::ConnectionAccepted { .. }
            | Self::ConnectionRejected { .. }
            | Self::Disconnect { .. } => MessageType::Connection,
            Self::Ping { .. } | Self::Pong { .. } => MessageType::Ping,
            Self::Replication(_) => MessageType::Replication,
            Self::ClientCommand { .. } | Self::CommandAck { .. } => MessageType::Command,
            Self::GameMessage { .. } => MessageType::Game,
        }
    }
    
    /// Serializes the message to bytes.
    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }
    
    /// Deserializes a message from bytes.
    pub fn deserialize(data: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(data)
    }
}

/// Replication message containing entity updates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicationMessage {
    /// Server tick when this snapshot was taken
    pub tick: u64,
    
    /// Timestamp in milliseconds
    pub timestamp: u64,
    
    /// Entity snapshots
    pub entities: Vec<EntitySnapshot>,
    
    /// IDs of entities that were destroyed
    pub destroyed_entities: Vec<u64>,
}

impl ReplicationMessage {
    /// Creates a new empty replication message.
    pub fn new(tick: u64, timestamp: u64) -> Self {
        Self {
            tick,
            timestamp,
            entities: Vec::new(),
            destroyed_entities: Vec::new(),
        }
    }
    
    /// Adds an entity snapshot.
    pub fn add_entity(&mut self, snapshot: EntitySnapshot) {
        self.entities.push(snapshot);
    }
    
    /// Marks an entity as destroyed.
    pub fn add_destroyed_entity(&mut self, entity_id: u64) {
        self.destroyed_entities.push(entity_id);
    }
}

/// Snapshot of an entity's state at a point in time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySnapshot {
    /// Network ID of the entity
    pub network_id: u64,
    
    /// Owner of the entity
    pub owner: Option<u64>,
    
    /// Component data
    pub components: HashMap<String, ComponentData>,
}

impl EntitySnapshot {
    /// Creates a new entity snapshot.
    pub fn new(network_id: u64, owner: Option<u64>) -> Self {
        Self {
            network_id,
            owner,
            components: HashMap::new(),
        }
    }
    
    /// Adds component data.
    pub fn add_component(&mut self, component_name: String, data: ComponentData) {
        self.components.insert(component_name, data);
    }
    
    /// Gets component data by name.
    pub fn get_component(&self, component_name: &str) -> Option<&ComponentData> {
        self.components.get(component_name)
    }
}

/// Serialized component data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentData {
    /// Serialized component bytes
    pub data: Vec<u8>,
}

impl ComponentData {
    /// Creates component data from serializable type.
    pub fn from_serializable<T: Serialize>(value: &T) -> Result<Self, bincode::Error> {
        Ok(Self {
            data: bincode::serialize(value)?,
        })
    }
    
    /// Deserializes component data.
    pub fn deserialize<T: for<'de> Deserialize<'de>>(&self) -> Result<T, bincode::Error> {
        bincode::deserialize(&self.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage::Connect {
            protocol_version: 1,
            client_name: "TestClient".to_string(),
        };
        
        let serialized = msg.serialize().unwrap();
        let deserialized = NetworkMessage::deserialize(&serialized).unwrap();
        
        match deserialized {
            NetworkMessage::Connect { protocol_version, client_name } => {
                assert_eq!(protocol_version, 1);
                assert_eq!(client_name, "TestClient");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_message_type() {
        let ping = NetworkMessage::Ping { timestamp: 100 };
        assert_eq!(ping.message_type(), MessageType::Ping);
        
        let replication = NetworkMessage::Replication(ReplicationMessage::new(1, 1000));
        assert_eq!(replication.message_type(), MessageType::Replication);
    }

    #[test]
    fn test_replication_message() {
        let mut replication = ReplicationMessage::new(42, 12345);
        assert_eq!(replication.tick, 42);
        assert_eq!(replication.timestamp, 12345);
        assert!(replication.entities.is_empty());
        
        let snapshot = EntitySnapshot::new(1, Some(5));
        replication.add_entity(snapshot);
        assert_eq!(replication.entities.len(), 1);
        
        replication.add_destroyed_entity(10);
        assert_eq!(replication.destroyed_entities.len(), 1);
    }

    #[test]
    fn test_entity_snapshot() {
        let mut snapshot = EntitySnapshot::new(123, Some(456));
        assert_eq!(snapshot.network_id, 123);
        assert_eq!(snapshot.owner, Some(456));
        
        let component_data = ComponentData::from_serializable(&42u32).unwrap();
        snapshot.add_component("health".to_string(), component_data);
        
        let retrieved = snapshot.get_component("health").unwrap();
        let value: u32 = retrieved.deserialize().unwrap();
        assert_eq!(value, 42);
    }

    #[test]
    fn test_component_data() {
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestComponent {
            value: i32,
            name: String,
        }
        
        let component = TestComponent {
            value: 42,
            name: "test".to_string(),
        };
        
        let data = ComponentData::from_serializable(&component).unwrap();
        let deserialized: TestComponent = data.deserialize().unwrap();
        
        assert_eq!(deserialized, component);
    }
}
