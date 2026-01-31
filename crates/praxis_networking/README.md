# praxis_networking

Networking and multiplayer for Praxis engine.

## Overview

Client-server networking with entity replication, lag compensation, and network profiling.

## Features

### Client-Server Architecture

- TCP for reliable commands
- UDP for unreliable state updates
- Connection management
- Heartbeat and timeout

### Entity Replication

- Automatic component synchronization
- Configurable replication frequency
- Delta compression
- Priority-based updates

### Interpolation & Extrapolation

- Client-side prediction
- Server reconciliation
- Smooth remote entity movement
- Configurable buffer size

### Lag Compensation

- Server-side rewind for hit detection
- Snapshot history
- Fair gameplay despite latency

### Network Profiler

- Bandwidth monitoring
- Latency tracking
- Packet loss detection
- Per-entity statistics

## Example

### Server

```rust
use praxis_networking::{NetworkServer, NetworkConfig};

let config = NetworkConfig {
    max_clients: 32,
    tick_rate: 60,
    ..Default::default()
};

let mut server = NetworkServer::new(config).await?;
server.start().await?;

// Game loop
loop {
    server.receive_messages()?;
    // Update game state
    server.replicate_entities(&world)?;
    server.send_updates()?;
}
```

### Client

```rust
use praxis_networking::{NetworkClient, NetworkConfig};

let config = NetworkConfig::default();
let mut client = NetworkClient::new(config).await?;
client.connect("127.0.0.1:7878").await?;

// Game loop
loop {
    client.receive_updates()?;
    client.apply_interpolation(&mut world)?;
    // Render
    client.send_commands(commands)?;
}
```

## Entity Replication

```rust
use praxis_networking::{ReplicationRegistry, Replicated};

// Register components for replication
let mut registry = ReplicationRegistry::new();
registry.register::<Transform>();
registry.register::<Velocity>();
registry.register::<Health>();

// Mark entity for replication
commands.spawn((
    Transform::default(),
    Velocity::default(),
    Replicated::new(update_frequency_hz: 20),
));
```

## Lag Compensation

```rust
// Server-side hit detection with rewind
let hit = server.check_hit_with_compensation(
    shooter_id,
    target_position,
    shooter_latency_ms,
)?;
```

## Network Profiler

```rust
let stats = server.network_stats();
println!("Bandwidth: {} KB/s", stats.bandwidth_kbps);
println!("Latency: {}ms", stats.avg_latency_ms);
println!("Packet loss: {:.2}%", stats.packet_loss_percent);
```

## Message Types

```rust
#[derive(Serialize, Deserialize)]
enum ServerMessage {
    Welcome { client_id: u64 },
    EntitySpawned { entity_id: u64, components: ComponentData },
    EntityUpdated { entity_id: u64, components: ComponentData },
    EntityDespawned { entity_id: u64 },
}

#[derive(Serialize, Deserialize)]
enum ClientMessage {
    Connect { player_name: String },
    Command { command_type: CommandType, data: Vec<u8> },
    Disconnect,
}
```

## Dependencies

- `tokio`: Async networking
- `serde`: Serialization
- `bincode`: Binary encoding
- `rustc-hash`: Fast hash maps
- `parking_lot`: Fast mutexes

## Usage

```toml
# In root Cargo.toml
[features]
networking = ["praxis_networking"]

# In your crate
praxis_networking = { path = "../praxis_networking", version = "0.1.0" }
```
