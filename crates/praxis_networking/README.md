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
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Configure server
    let config = NetworkConfig {
        bind_addr: "0.0.0.0:7777".to_string(),
        max_clients: 32,
        tick_rate: 60,
        ..Default::default()
    };
    
    // Create and start server
    let mut server = NetworkServer::new(config).await?;
    server.start().await?;
    
    // Register components for replication
    let mut registry = ReplicationRegistry::new();
    registry.register_transform();
    registry.register_velocity();
    
    // Game loop
    loop {
        let delta_time = 0.016; // 60 FPS
        server.update(delta_time)?;
    }
}
```

### Client

```rust
use praxis_networking::{NetworkClient, NetworkConfig};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create client
    let mut client = NetworkClient::new(NetworkConfig::default()).await?;
    
    // Connect to server
    client.connect("127.0.0.1:7777", "Player1".to_string()).await?;
    
    // Game loop
    loop {
        let delta_time = 0.016; // 60 FPS
        client.update(delta_time)?;
    }
}
```

## Entity Replication

```rust
use praxis_networking::{NetworkId, Replicated, ReplicatedTransform};
use praxis_ecs::{World, Transform};

fn spawn_replicated_entity(world: &mut World) {
    world.spawn((
        // Unique network identifier
        NetworkId::new(1),
        
        // Replication component with priority (255 = highest)
        Replicated::new().with_priority(255),
        
        // Standard Transform
        Transform::default(),
        
        // Replicated Transform for network sync
        ReplicatedTransform::default(),
    ));
}
```

## Documentation

**Comprehensive Guide:**
- [Networking Guide](../../docs/guides/systems/networking.md) - Complete multiplayer guide

**Learning Path:**
- [Networking Learning Path](../../docs/learning-paths/networking.md)

## Examples

```bash
# Run networking demo
cargo run --example networking_demo
```

## Dependencies

- `tokio` 1.40: Async runtime
- `bincode`: Binary serialization
- `bevy_ecs` 0.14: ECS integration
