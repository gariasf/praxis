# Praxis Networking Integration Guide

This guide explains how to integrate the networking system into your Praxis game.

## Overview

The networking system provides:
- **Server**: Manages connections, replicates entities, handles lag compensation
- **Client**: Connects to server, receives updates, sends input
- **Replication**: Synchronizes ECS components across network
- **Interpolation**: Smooths remote entity movement
- **Lag Compensation**: Fair hit detection despite network latency
- **Profiling**: Real-time network performance monitoring

## Setup

### 1. Add Dependencies

The networking crate is already included in the workspace. For standalone usage:

```toml
[dependencies]
praxis_networking = { path = "../praxis_networking" }
praxis_ecs = { path = "../praxis_ecs" }
praxis_math = { path = "../praxis_math" }
tokio = { version = "1.40", features = ["full"] }
```

### 2. Initialize Networking

```rust
use praxis_networking::{init, NetworkConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize networking system
    praxis_networking::init()?;
    
    // Your game code here
}
```

## Server Implementation

### Basic Server

```rust
use praxis_networking::{NetworkServer, NetworkConfig};

let config = NetworkConfig {
    bind_addr: "0.0.0.0:7777".to_string(),
    max_clients: 32,
    tick_rate: 60,
    ..Default::default()
};

let mut server = NetworkServer::new(config).await?;
server.start().await?;

// Game loop
loop {
    let delta = 1.0 / 60.0;
    server.update(delta)?;
    
    // Update game logic
    // ...
}
```

### Register Components for Replication

```rust
use praxis_networking::ReplicationRegistry;

let mut registry = ReplicationRegistry::new();

// Built-in components
registry.register_transform();
registry.register_velocity();

// Custom components
registry.register::<Health>("Health");
registry.register::<PlayerState>("PlayerState");
```

### Spawn Replicated Entities

```rust
use praxis_networking::{NetworkId, Replicated, ReplicatedTransform};
use praxis_ecs::World;

let mut world = World::new();

// Spawn a replicated entity
world.spawn((
    NetworkId::new(entity_id),
    Replicated::new()
        .with_priority(200)      // Higher priority = more important
        .with_rate_divisor(1),   // 1 = every tick, 2 = every other tick
    Transform::from_xyz(0.0, 0.0, 0.0),
    ReplicatedTransform::new(Vec3::ZERO, Quat::IDENTITY, Vec3::ONE),
    ReplicatedVelocity::zero(),
));
```

### Lag Compensation Setup

```rust
use praxis_networking::{LagCompensation, LagCompensationSystem};

let mut lag_comp = LagCompensation::new(1000); // 1 second history

// In game loop
LagCompensationSystem::update(&mut lag_comp, client_id, &world);
```

### Validate Client Actions with Lag Compensation

```rust
// When client performs action (e.g., shoots)
let client_timestamp = action.timestamp;
let client_rtt = get_client_rtt(client_id);

// Rewind to client's time
let hit = lag_comp.raycast_at_client_time(
    client_id,
    client_timestamp - client_rtt / 2,
    &mut world,
    ray_origin,
    ray_direction,
    max_distance,
)?;

if let Some(hit) = hit {
    // Valid hit from client's perspective
    apply_damage(hit.entity, damage);
}
```

## Client Implementation

### Basic Client

```rust
use praxis_networking::{NetworkClient, NetworkConfig};

let config = NetworkConfig::default();
let mut client = NetworkClient::new(config).await?;

// Connect to server
client.connect("127.0.0.1:7777", "Player1".to_string()).await?;

// Game loop
loop {
    let delta = 1.0 / 60.0;
    client.update(delta)?;
    
    // Process input
    // Render game
    // ...
}
```

### Setup Interpolation

```rust
use praxis_networking::{
    NetworkInterpolation, InterpolationBuffer, InterpolationSystem,
};
use praxis_ecs::Schedule;

// Add interpolation to remote entities
world.spawn((
    NetworkId::new(remote_entity_id),
    ReplicatedTransform::default(),
    NetworkInterpolation::new(100.0),  // 100ms interpolation delay
    InterpolationBuffer::default(),
));

// Create schedule with interpolation system
let mut schedule = Schedule::default();

// In game loop
schedule.run(world.inner_mut());
```

### Setup Extrapolation

```rust
use praxis_networking::{NetworkExtrapolation, ExtrapolationSystem};

// Add extrapolation to entities with velocity
world.spawn((
    NetworkId::new(entity_id),
    ReplicatedTransform::default(),
    ReplicatedVelocity::default(),
    NetworkExtrapolation::new(200.0),  // Max 200ms extrapolation
    InterpolationBuffer::default(),
));

// Update extrapolation in game loop
// (Automatically handled by ExtrapolationSystem::update)
```

### Client-Side Prediction

For player-controlled entities:

```rust
use praxis_networking::ClientPredicted;

// Mark entity as client-predicted
world.spawn((
    NetworkId::new(player_entity_id),
    ClientPredicted::new(),
    ReplicatedTransform::default(),
    // ... other components
));

// Apply input immediately
apply_player_input(&mut transform, input);

// Send input to server
send_client_command(input, tick);

// When server update arrives, reconcile
if server_state.tick > predicted_state.last_ack_tick {
    // Server is more recent, apply correction
    reconcile_prediction(local_state, server_state);
}
```

## Network Profiling

### Setup Profiler

```rust
use praxis_networking::NetworkProfiler;

let profiler = NetworkProfiler::new();

// Record network activity
profiler.record_sent(packet_size);
profiler.record_received(packet_size);
profiler.record_latency(rtt_ms);

// Update each frame
profiler.update(delta_time);
```

### Display Network Stats

```rust
let stats = profiler.get_stats();

println!("Bandwidth:");
println!("  Send: {:.2} KB/s", stats.bandwidth.send_rate / 1024.0);
println!("  Receive: {:.2} KB/s", stats.bandwidth.receive_rate / 1024.0);

println!("Latency:");
println!("  RTT: {:.1} ms", stats.latency.rtt_ms);
println!("  Jitter: {:.1} ms", stats.latency.jitter_ms);
```

### Integrate with GUI

```rust
// In your egui rendering code
egui::Window::new("Network Stats").show(ctx, |ui| {
    let stats = profiler.get_stats();
    
    ui.label(format!("Send Rate: {:.2} KB/s", 
        stats.bandwidth.send_rate / 1024.0));
    ui.label(format!("Recv Rate: {:.2} KB/s", 
        stats.bandwidth.receive_rate / 1024.0));
    ui.label(format!("RTT: {:.1} ms", stats.latency.rtt_ms));
    ui.label(format!("Jitter: {:.1} ms", stats.latency.jitter_ms));
});
```

## Advanced Topics

### Custom Message Types

```rust
use praxis_networking::NetworkMessage;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct CustomGameMessage {
    message_type: u32,
    data: Vec<u8>,
}

// Send custom message
let custom_msg = NetworkMessage::GameMessage {
    message_type: 100,  // Your message type
    data: bincode::serialize(&my_data)?,
};

server.send_reliable(client_addr, &custom_msg)?;
```

### Priority-Based Replication

```rust
// Critical entities (players, projectiles)
Replicated::new()
    .with_priority(255)
    .with_rate_divisor(1)  // Every tick

// Important entities (NPCs, vehicles)
Replicated::new()
    .with_priority(128)
    .with_rate_divisor(1)

// Background entities (scenery, pickups)
Replicated::new()
    .with_priority(64)
    .with_rate_divisor(2)  // Every other tick
```

### Bandwidth Management

```rust
// Monitor per-message-type bandwidth
let bandwidth = profiler.bandwidth();

for (msg_type, bytes) in &bandwidth.bytes_by_type {
    println!("{:?}: {} bytes", msg_type, bytes);
}

// Adjust replication rates based on bandwidth
if bandwidth.send_rate > target_bandwidth {
    // Reduce update frequency for low-priority entities
    for (entity, mut replicated) in query.iter_mut() {
        if replicated.priority < 128 {
            replicated.rate_divisor = 2;
        }
    }
}
```

### Dead Reckoning

```rust
// Extrapolation provides basic dead reckoning
// For more complex movement:

fn predict_position(
    transform: &ReplicatedTransform,
    velocity: &ReplicatedVelocity,
    acceleration: Vec3,
    dt: f32,
) -> Vec3 {
    // Use physics equations
    transform.translation 
        + velocity.linear * dt 
        + 0.5 * acceleration * dt * dt
}
```

## Performance Tips

1. **Tick Rate**: Balance between responsiveness and bandwidth
   - Competitive: 60-128 Hz
   - Casual: 20-30 Hz
   - MMO: 10-20 Hz

2. **Interpolation Delay**: Trade smoothness for responsiveness
   - Low latency: 50-100ms
   - Standard: 100-150ms
   - High quality: 150-200ms

3. **Replication Priorities**: 
   - Use rate divisors for less important entities
   - Adjust priorities dynamically based on distance to players

4. **Compression**: 
   - Use delta compression for large states (future enhancement)
   - Quantize floating-point values where precision isn't critical

5. **Culling**:
   - Only replicate entities relevant to each client
   - Use spatial partitioning to determine relevance

## Troubleshooting

### High Latency

- Check network profiler for spikes
- Reduce packet size or send frequency
- Use unreliable transport for non-critical updates

### Jittery Movement

- Increase interpolation delay
- Check for packet loss
- Ensure consistent tick rate

### Desynchronization

- Verify all replicated components are registered
- Check that NetworkId is unique per entity
- Ensure server is authoritative for gameplay state

### Memory Usage

- Reduce lag compensation history duration
- Limit interpolation buffer size
- Clear old snapshots regularly

## Best Practices

1. **Authority**: Server is always authoritative
2. **Prediction**: Only predict client-owned entities
3. **Validation**: Always validate client actions server-side
4. **Reconciliation**: Apply server corrections smoothly
5. **Testing**: Test with artificial latency and packet loss
6. **Monitoring**: Always enable network profiling during development

## Next Steps

- See `examples/networking_demo.rs` for complete example
- Read `crates/praxis_networking/README.md` for API details
- Check test cases in each module for usage patterns
