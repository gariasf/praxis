# Praxis Networking

Client-server networking with entity replication for the Praxis game engine.

## Overview

Comprehensive multiplayer networking with automatic component synchronization, interpolation, and lag compensation.

**Key Features:**
- Client-server architecture (TCP/UDP)
- Automatic entity replication
- Component serialization (any Serde type)
- Interpolation/extrapolation for smooth movement
- Server-side lag compensation for hit detection
- Network profiler (bandwidth, latency, jitter)

## Quick Start

### Server

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

// Register components
let mut registry = ReplicationRegistry::new();
registry.register_transform();
registry.register_velocity();

// Game loop
loop {
    server.update(delta_time)?;
}
```

### Client

```rust
use praxis_networking::NetworkClient;

let mut client = NetworkClient::new(NetworkConfig::default()).await?;
client.connect("127.0.0.1:7777", "Player1".to_string()).await?;

loop {
    client.update(delta_time)?;
}
```

## Entity Replication

```rust
use praxis_networking::{NetworkId, Replicated, ReplicatedTransform};

world.spawn((
    NetworkId::new(1),
    Replicated::new().with_priority(255),
    Transform::default(),
    ReplicatedTransform::default(),
));
```

## Documentation

**Comprehensive Guide:**
- [Networking Guide](../../docs/guides/systems/networking.md) - Complete multiplayer guide

**Learning Path:**
- [Networking Learning Path](../../docs/learning-paths/networking.md)

## Examples

```bash
cargo run --example networking_demo
```

## Dependencies

- `tokio` 1.40: Async runtime
- `bincode`: Binary serialization
- `bevy_ecs` 0.14: ECS integration
