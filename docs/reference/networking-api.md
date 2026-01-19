# Networking API Reference

API reference for the Praxis multiplayer networking system.

## Core Types

### NetworkServer

Server-side networking controller.

```rust
pub struct NetworkServer { /* ... */ }
```

**Methods:**
- `new(config: NetworkConfig) -> Result<Self>`
- `start() -> Result<()>` - Begin accepting connections
- `stop() -> Result<()>` - Stop server
- `update(delta_time: f32) -> Result<()>` - Process network events
- `send_to(client_id: u64, message: &impl Serialize) -> Result<()>`
- `broadcast(message: &impl Serialize) -> Result<()>`
- `disconnect_client(client_id: u64) -> Result<()>`
- `client_count() -> usize`

### NetworkClient

Client-side networking controller.

```rust
pub struct NetworkClient { /* ... */ }
```

**Methods:**
- `new(config: NetworkConfig) -> Result<Self>`
- `connect(addr: &str, player_name: String) -> Result<()>`
- `disconnect() -> Result<()>`
- `update(delta_time: f32) -> Result<()>` - Process network events
- `send(message: &impl Serialize) -> Result<()>`
- `is_connected() -> bool`

### NetworkConfig

Configuration for client or server.

```rust
pub struct NetworkConfig {
    pub bind_addr: String,           // Server listen address
    pub max_clients: usize,          // Maximum concurrent clients
    pub tick_rate: u32,              // Network updates per second (default: 60)
    pub timeout_seconds: f32,        // Client timeout (default: 10.0)
    pub compression: bool,           // Enable message compression
    pub interpolation_delay: f32,    // Client interpolation delay (default: 0.1)
}
```

## Entity Replication

### Components

#### NetworkId

Unique network identifier for replicated entities.

```rust
#[derive(Component)]
pub struct NetworkId(pub u64);
```

**Methods:**
- `new(id: u64)` - Create from ID
- `generate()` - Generate unique ID

#### Replicated

Marks entity for network replication.

```rust
#[derive(Component)]
pub struct Replicated {
    pub priority: u8,        // Replication priority (0-255, higher = more important)
    pub owner: Option<u64>,  // Client ID of owner (None = server)
}
```

**Methods:**
- `new()` - Default priority
- `with_priority(priority: u8)` - Set priority
- `with_owner(client_id: u64)` - Set owner

#### ReplicatedTransform

Replicated transform component.

```rust
#[derive(Component)]
pub struct ReplicatedTransform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
```

#### ReplicatedVelocity

Replicated velocity for prediction.

```rust
#[derive(Component)]
pub struct ReplicatedVelocity {
    pub linear: Vec3,
    pub angular: Vec3,
}
```

### ReplicationRegistry

Registry for component serialization.

```rust
pub struct ReplicationRegistry { /* ... */ }
```

**Methods:**
- `new()` - Create empty registry
- `register_transform()` - Register transform replication
- `register_velocity()` - Register velocity replication
- `register<T: Component + Serialize + DeserializeOwned>(name: &str)`
- `unregister(name: &str)`

## Interpolation

### InterpolationBuffer

Client-side interpolation buffer.

```rust
pub struct InterpolationBuffer<T> {
    pub delay: f32,  // Interpolation delay in seconds
}
```

**Methods:**
- `new(delay: f32)` - Create buffer
- `add_sample(timestamp: f32, value: T)` - Add server state
- `interpolate(current_time: f32) -> Option<T>` - Get interpolated value

### PredictionState

Client-side prediction state.

```rust
#[derive(Component)]
pub struct PredictionState {
    pub predicted_position: Vec3,
    pub predicted_rotation: Quat,
    pub last_ack_tick: u32,
}
```

## Lag Compensation

### LagCompensationHistory

Server-side state history for lag compensation.

```rust
pub struct LagCompensationHistory {
    pub max_history_seconds: f32,  // Maximum history retention
}
```

**Methods:**
- `new(max_seconds: f32)` - Create history buffer
- `record_snapshot(tick: u32, world: &World)` - Record world state
- `rewind_to(tick: u32, world: &mut World)` - Restore past state
- `restore_current(world: &mut World)` - Restore to present

## Network Profiling

### NetworkProfiler

Performance monitoring for network operations.

```rust
pub struct NetworkProfiler { /* ... */ }
```

**Methods:**
- `new()` - Create profiler
- `record_sent(bytes: usize)` - Record bytes sent
- `record_received(bytes: usize)` - Record bytes received
- `average_latency() -> f32` - Get average ping
- `bandwidth_usage() -> (f32, f32)` - (sent, received) bytes/sec
- `packet_loss() -> f32` - Packet loss percentage

## Events

### NetworkEvent

Events from the network layer.

```rust
pub enum NetworkEvent {
    ClientConnected { client_id: u64, player_name: String },
    ClientDisconnected { client_id: u64 },
    MessageReceived { from: u64, data: Vec<u8> },
    ConnectionError { error: String },
}
```

## Common Patterns

### Server Setup

```rust
use praxis_networking::{NetworkServer, NetworkConfig, ReplicationRegistry};

let config = NetworkConfig {
    bind_addr: "0.0.0.0:7777".to_string(),
    max_clients: 32,
    tick_rate: 60,
    ..Default::default()
};

let mut server = NetworkServer::new(config).await?;
server.start().await?;

let mut registry = ReplicationRegistry::new();
registry.register_transform();
registry.register_velocity();

world.insert_resource(server);
world.insert_resource(registry);
```

### Client Setup

```rust
use praxis_networking::{NetworkClient, NetworkConfig};

let config = NetworkConfig {
    tick_rate: 60,
    interpolation_delay: 0.1,
    ..Default::default()
};

let mut client = NetworkClient::new(config).await?;
client.connect("127.0.0.1:7777", "Player1".to_string()).await?;

world.insert_resource(client);
```

### Spawning Replicated Entity

```rust
world.spawn((
    NetworkId::generate(),
    Replicated::new().with_priority(255),
    Transform::default(),
    ReplicatedTransform::default(),
    Velocity::default(),
    ReplicatedVelocity::default(),
));
```

### Server Update Loop

```rust
fn network_server_system(
    mut server: ResMut<NetworkServer>,
    time: Res<Time>,
) {
    server.update(time.delta_seconds()).unwrap();
}
```

### Client Update Loop

```rust
fn network_client_system(
    mut client: ResMut<NetworkClient>,
    time: Res<Time>,
) {
    client.update(time.delta_seconds()).unwrap();
}
```

## See Also

- [Networking Guide](../guides/systems/networking.md) - Comprehensive multiplayer guide
- [Networking Learning Path](../learning-paths/networking.md) - Step-by-step tutorials
- [praxis_networking Crate](../../crates/praxis_networking/README.md) - Crate documentation
