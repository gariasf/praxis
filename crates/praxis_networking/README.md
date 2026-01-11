# Praxis Networking

Comprehensive networking system for the Praxis game engine with client-server architecture, entity replication, interpolation/extrapolation, and lag compensation.

## Features

- **Client-Server Architecture**: Robust server and client implementations with TCP/UDP support
- **Entity Replication**: Automatic synchronization of ECS components across network
- **Component Serialization**: Generic serialization system for any Serde-compatible component
- **Interpolation**: Smooth remote entity movement with configurable delay
- **Extrapolation**: Prediction for entities when updates are delayed
- **Lag Compensation**: Server-side rewind for fair hit detection
- **Network Profiler**: Real-time monitoring of bandwidth, latency, and jitter
- **Flexible Configuration**: Customizable tick rates, buffer sizes, and timeouts

## Architecture

The networking system is built on several key components:

### Transport Layer
- **TCP Transport**: Reliable, ordered message delivery for critical data
- **UDP Transport**: Fast, unreliable delivery for real-time updates
- Automatic connection management and heartbeat

### Replication System
- **Component Registry**: Register any Serde-compatible component for replication
- **Priority System**: Control bandwidth usage with component priorities
- **Rate Control**: Adjust update frequency per entity or component
- **Snapshot-based**: Efficient delta compression for large worlds

### Interpolation & Extrapolation
- **Snapshot Buffer**: Stores historical entity states
- **Automatic Interpolation**: Smooth movement between network updates
- **Dead Reckoning**: Extrapolate movement when updates are delayed
- **Configurable Delays**: Balance smoothness vs. responsiveness

### Lag Compensation
- **History Buffer**: Stores entity states over time
- **Time Rewind**: Server rewinds to client's perspective for hit detection
- **Automatic Restore**: Returns world to current state after validation
- **Interpolated History**: Smooth state reconstruction at any timestamp

### Network Profiler
- **Bandwidth Tracking**: Monitor bytes sent/received per second
- **Latency Metrics**: RTT, jitter, min/max/avg measurements
- **Per-Type Stats**: Track bandwidth usage by message type
- **Peak Detection**: Identify network spikes

## Usage

### Server

```rust
use praxis_networking::{NetworkServer, NetworkConfig, ReplicationRegistry};
use praxis_ecs::World;

#[tokio::main]
async fn main() -> Result<()> {
    let mut world = World::new();
    
    // Configure server
    let config = NetworkConfig {
        bind_addr: "0.0.0.0:7777".to_string(),
        max_clients: 32,
        tick_rate: 60,
        enable_interpolation: true,
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
        let delta_time = 1.0 / 60.0;
        server.update(delta_time)?;
        
        // Update game logic here
    }
}
```

### Client

```rust
use praxis_networking::{NetworkClient, NetworkConfig};

#[tokio::main]
async fn main() -> Result<()> {
    let config = NetworkConfig::default();
    let mut client = NetworkClient::new(config).await?;
    
    // Connect to server
    client.connect("127.0.0.1:7777", "Player1".to_string()).await?;
    
    // Game loop
    loop {
        let delta_time = 1.0 / 60.0;
        client.update(delta_time)?;
        
        // Send input, render game, etc.
    }
}
```

### Entity Replication

```rust
use praxis_networking::{NetworkId, Replicated, ReplicatedTransform};
use praxis_ecs::{World, Transform, GlobalTransform};
use praxis_math::{Vec3, Quat};

let mut world = World::new();

// Spawn a replicated entity
let entity = world.spawn((
    NetworkId::new(1),
    Replicated::new()
        .with_priority(255)
        .with_rate_divisor(1),
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
));
```

### Interpolation

```rust
use praxis_networking::{
    NetworkInterpolation, InterpolationBuffer, InterpolationSystem,
};

// Add interpolation components to remote entities
world.spawn((
    NetworkId::new(2),
    ReplicatedTransform::default(),
    NetworkInterpolation::new(100.0), // 100ms delay
    InterpolationBuffer::default(),
));

// Update interpolation each frame
InterpolationSystem::update(query, delta_time);
```

### Lag Compensation

```rust
use praxis_networking::{LagCompensation, LagCompensationSystem};
use praxis_math::Vec3;

let mut lag_comp = LagCompensation::new(1000); // 1 second history

// Record snapshots each tick
LagCompensationSystem::update(&mut lag_comp, client_id, &world);

// Perform lag-compensated raycast
let hit = lag_comp.raycast_at_client_time(
    client_id,
    timestamp,
    &mut world,
    ray_origin,
    ray_direction,
    100.0, // max distance
)?;
```

### Network Profiler

```rust
use praxis_networking::NetworkProfiler;

let profiler = NetworkProfiler::new();

// Record network activity
profiler.record_sent(packet_size);
profiler.record_received(packet_size);
profiler.record_latency(rtt_ms);

// Update profiler
profiler.update(delta_time);

// Get statistics
let stats = profiler.get_stats();
println!("Send rate: {} bytes/sec", stats.bandwidth.send_rate);
println!("RTT: {} ms", stats.latency.rtt_ms);
println!("Jitter: {} ms", stats.latency.jitter_ms);
```

## Component Registration

Register custom components for replication:

```rust
use serde::{Serialize, Deserialize};
use praxis_ecs::Component;

#[derive(Component, Serialize, Deserialize, Clone)]
struct Health {
    current: f32,
    max: f32,
}

let mut registry = ReplicationRegistry::new();
registry.register::<Health>("Health");
```

## Network Configuration

Customize network behavior:

```rust
let config = NetworkConfig {
    bind_addr: "0.0.0.0:7777".to_string(),
    max_clients: 64,
    tick_rate: 128, // High tick rate for competitive games
    enable_interpolation: true,
    enable_extrapolation: true,
    interpolation_delay_ms: 50, // Low latency
    enable_lag_compensation: true,
    lag_compensation_history_ms: 2000, // 2 seconds
    max_packet_size: 1400, // MTU safe
    enable_profiling: true,
};
```

## Best Practices

1. **Component Selection**: Only replicate components that change and need synchronization
2. **Priority Management**: Assign higher priority to critical entities (players, projectiles)
3. **Rate Control**: Use rate divisors for less important entities (background objects)
4. **Interpolation Delay**: Balance smoothness (higher delay) vs. responsiveness (lower delay)
5. **History Length**: Longer history for lag compensation = more memory but fairer gameplay
6. **Profiling**: Monitor bandwidth to avoid overwhelming connections

## Performance

- **Efficient Serialization**: Binary format with bincode for minimal overhead
- **Delta Compression**: Only send changed components (future enhancement)
- **Batch Updates**: Multiple entity updates in single packet
- **Thread Safety**: Lock-free where possible, minimal contention
- **Memory Pooling**: Reuse buffers for network I/O (future enhancement)

## Limitations

- Currently uses simplified physics for lag compensation raycasts
- No built-in delta compression (full state updates)
- TCP transport is simplified (async I/O not fully integrated)
- No automatic reconnection handling
- Limited to 64-bit entity IDs

## Future Enhancements

- Delta compression for bandwidth optimization
- Reliable UDP implementation (custom protocol)
- Entity relevance filtering (spatial culling)
- Automatic client migration for server handoff
- Voice chat integration
- NAT traversal support

## Examples

Run the networking demo:

```bash
cargo run --example networking_demo
```

## Dependencies

- `tokio` 1.40: Async runtime
- `bincode`: Binary serialization
- `serde`: Serialization framework
- `bevy_ecs` 0.14: ECS integration
- `praxis_ecs`: Transform and component systems
- `praxis_math`: Math types
- `praxis_utils`: Error handling

## See Also

- [Networking Guide](../../docs/guides/systems/networking.md)
- [Networking Learning Path](../../docs/learning-paths/networking.md)
