//! Network-specific ECS components.

use bevy_ecs::prelude::*;
use praxis_math::{Quat, Vec3};
use serde::{Deserialize, Serialize};

/// Unique network identifier for entities.
///
/// This component marks an entity as networked and provides a stable
/// identifier for synchronization across clients and server.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkId(pub u64);

impl NetworkId {
    /// Creates a new network ID.
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// Gets the underlying ID value.
    pub fn get(&self) -> u64 {
        self.0
    }
}

/// Marks the owner of a networked entity.
///
/// For client-owned entities, this stores the client ID.
/// For server-owned entities, this is None.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NetworkOwner(pub Option<u64>);

impl NetworkOwner {
    /// Creates a server-owned entity marker.
    pub fn server() -> Self {
        Self(None)
    }

    /// Creates a client-owned entity marker.
    pub fn client(client_id: u64) -> Self {
        Self(Some(client_id))
    }

    /// Returns true if this entity is owned by the server.
    pub fn is_server(&self) -> bool {
        self.0.is_none()
    }

    /// Returns true if this entity is owned by a client.
    pub fn is_client(&self) -> bool {
        self.0.is_some()
    }

    /// Gets the client ID if this entity is client-owned.
    pub fn client_id(&self) -> Option<u64> {
        self.0
    }
}

/// Marks an entity as replicated across the network.
///
/// Entities with this component will have their state synchronized
/// to all clients.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Replicated {
    /// Whether this entity should be replicated
    pub enabled: bool,

    /// Priority for bandwidth management (higher = more important)
    pub priority: u8,

    /// Replication rate divisor (1 = every tick, 2 = every other tick, etc.)
    pub rate_divisor: u8,
}

impl Replicated {
    /// Creates a new replicated marker with default settings.
    pub fn new() -> Self {
        Self {
            enabled: true,
            priority: 128,
            rate_divisor: 1,
        }
    }

    /// Sets the replication priority.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the replication rate divisor.
    pub fn with_rate_divisor(mut self, rate_divisor: u8) -> Self {
        self.rate_divisor = rate_divisor;
        self
    }
}

/// Replicated transform data.
///
/// This is a serializable version of Transform for network replication.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReplicatedTransform {
    /// Position in world space
    pub translation: Vec3,

    /// Rotation as a quaternion
    pub rotation: Quat,

    /// Scale
    pub scale: Vec3,
}

impl ReplicatedTransform {
    /// Creates a new replicated transform.
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

    /// Creates from a Transform component.
    pub fn from_transform(transform: &praxis_ecs::Transform) -> Self {
        Self {
            translation: transform.translation,
            rotation: transform.rotation,
            scale: transform.scale,
        }
    }

    /// Converts to a Transform component.
    pub fn to_transform(&self) -> praxis_ecs::Transform {
        praxis_ecs::Transform {
            translation: self.translation,
            rotation: self.rotation,
            scale: self.scale,
        }
    }
}

/// Replicated velocity data.
///
/// Used for extrapolation and prediction of entity movement.
#[derive(Component, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ReplicatedVelocity {
    /// Linear velocity
    pub linear: Vec3,

    /// Angular velocity (axis-angle representation)
    pub angular: Vec3,
}

impl ReplicatedVelocity {
    /// Creates a new replicated velocity.
    pub fn new(linear: Vec3, angular: Vec3) -> Self {
        Self { linear, angular }
    }

    /// Creates a zero velocity.
    pub fn zero() -> Self {
        Self {
            linear: Vec3::ZERO,
            angular: Vec3::ZERO,
        }
    }
}

impl Default for ReplicatedVelocity {
    fn default() -> Self {
        Self::zero()
    }
}

/// Interpolation state for remote entities.
///
/// Stores snapshots for smooth interpolation between network updates.
#[derive(Component, Debug)]
pub struct NetworkInterpolation {
    /// Whether interpolation is enabled for this entity
    pub enabled: bool,

    /// Target interpolation delay in milliseconds
    pub delay_ms: f32,

    /// Current interpolation time
    pub current_time: f32,
}

impl NetworkInterpolation {
    /// Creates a new interpolation state.
    pub fn new(delay_ms: f32) -> Self {
        Self {
            enabled: true,
            delay_ms,
            current_time: 0.0,
        }
    }
}

impl Default for NetworkInterpolation {
    fn default() -> Self {
        Self::new(100.0)
    }
}

/// Extrapolation state for remote entities.
///
/// Used when no recent updates are available to predict future positions.
#[derive(Component, Debug)]
pub struct NetworkExtrapolation {
    /// Whether extrapolation is enabled for this entity
    pub enabled: bool,

    /// Maximum extrapolation time before freezing in milliseconds
    pub max_time_ms: f32,

    /// Time since last update
    pub time_since_update: f32,
}

impl NetworkExtrapolation {
    /// Creates a new extrapolation state.
    pub fn new(max_time_ms: f32) -> Self {
        Self {
            enabled: true,
            max_time_ms,
            time_since_update: 0.0,
        }
    }
}

impl Default for NetworkExtrapolation {
    fn default() -> Self {
        Self::new(200.0)
    }
}

/// Marks an entity as client-predicted.
///
/// Client-predicted entities are updated locally on the owning client
/// and corrections are applied when server updates arrive.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ClientPredicted {
    /// Last acknowledged server tick
    pub last_ack_tick: u64,

    /// Number of pending commands
    pub pending_commands: u32,
}

impl ClientPredicted {
    /// Creates a new client prediction state.
    pub fn new() -> Self {
        Self {
            last_ack_tick: 0,
            pending_commands: 0,
        }
    }
}

/// Marks an entity as server-authoritative.
///
/// Server-authoritative entities are never predicted on clients and
/// always use interpolation/extrapolation.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ServerAuthoritative;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_id() {
        let id = NetworkId::new(42);
        assert_eq!(id.get(), 42);
    }

    #[test]
    fn test_network_owner() {
        let server_owner = NetworkOwner::server();
        assert!(server_owner.is_server());
        assert!(!server_owner.is_client());
        assert_eq!(server_owner.client_id(), None);

        let client_owner = NetworkOwner::client(123);
        assert!(!client_owner.is_server());
        assert!(client_owner.is_client());
        assert_eq!(client_owner.client_id(), Some(123));
    }

    #[test]
    fn test_replicated() {
        let replicated = Replicated::new().with_priority(255).with_rate_divisor(2);

        assert!(replicated.enabled);
        assert_eq!(replicated.priority, 255);
        assert_eq!(replicated.rate_divisor, 2);
    }

    #[test]
    fn test_replicated_transform() {
        let transform = praxis_ecs::Transform {
            translation: Vec3::new(1.0, 2.0, 3.0),
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        };

        let replicated = ReplicatedTransform::from_transform(&transform);
        assert_eq!(replicated.translation, Vec3::new(1.0, 2.0, 3.0));

        let back = replicated.to_transform();
        assert_eq!(back.translation, transform.translation);
    }

    #[test]
    fn test_replicated_velocity() {
        let vel = ReplicatedVelocity::new(Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO);
        assert_eq!(vel.linear, Vec3::new(1.0, 0.0, 0.0));

        let zero = ReplicatedVelocity::zero();
        assert_eq!(zero.linear, Vec3::ZERO);
    }

    #[test]
    fn test_client_predicted() {
        let mut predicted = ClientPredicted::new();
        assert_eq!(predicted.last_ack_tick, 0);
        assert_eq!(predicted.pending_commands, 0);

        predicted.last_ack_tick = 100;
        predicted.pending_commands = 5;
        assert_eq!(predicted.last_ack_tick, 100);
        assert_eq!(predicted.pending_commands, 5);
    }
}
