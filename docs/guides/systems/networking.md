# Networking System Guide

Practical guide to building multiplayer games with Praxis using client-server architecture, entity replication, and lag compensation.

**Related Architecture Documentation:**
- [Multiplayer Data Flow](../../architecture/multiplayer-data-flow.md) - Complete visual guide to client-server data flow, entity replication, and lag compensation

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Architecture](#architecture)
4. [Server Setup](#server-setup)
5. [Client Setup](#client-setup)
6. [Entity Replication](#entity-replication)
7. [Interpolation & Extrapolation](#interpolation--extrapolation)
8. [Lag Compensation](#lag-compensation)
9. [Network Profiling](#network-profiling)
10. [Configuration](#configuration)
11. [Common Patterns](#common-patterns)
12. [Best Practices](#best-practices)
13. [Troubleshooting](#troubleshooting)

## Overview

The Praxis networking system provides a complete client-server multiplayer solution built on:

- **Client-Server Architecture**: Authoritative server with UDP/TCP support
- **ECS Integration**: Automatic synchronization of components across network
- **Interpolation**: Smooth remote entity movement between network updates
- **Extrapolation**: Dead reckoning for prediction during packet loss
- **Lag Compensation**: Server-side rewind for fair hit detection
- **Network Profiler**: Real-time bandwidth and latency monitoring

### Why Client-Server?

Client-server architecture was chosen over peer-to-peer for several reasons:

1. **Authority**: Server has final say on game state, preventing cheating
2. **Consistency**: Single source of truth simplifies state management
3. **Scalability**: Easier to scale with dedicated server infrastructure
4. **Late Join**: New players can connect mid-game seamlessly
5. **Industry Standard**: Well-understood patterns and best practices

### Design Philosophy

- **Pragmatic**: Focus on common multiplayer game patterns
- **ECS-Native**: Leverage Bevy ECS for efficient component replication
- **Flexible**: Support both fast-paced (FPS) and slower (strategy) games
- **Observable**: Built-in profiling for debugging network issues
- **Type-Safe**: Rust's type system prevents common networking bugs

## Quick Start

### Minimal Server

```rust
use praxis_networking::{NetworkServer, NetworkConfig};
use praxis_ecs::World;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let mut world = World::new();
    
    let config = NetworkConfig {
        bind_addr: "0.0.0.0:7777".to_string(),
        max_clients: 32,
        tick_rate: 60,
        ..Default::default()
    };
    
    let mut server = NetworkServer::new(config).await?;
    server.start().await?;
    
    loop {
        server.update(1.0 / 60.0)?;
        // Game logic here
    }
}
```

### Minimal Client

```rust
use praxis_networking::{NetworkClient, NetworkConfig};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let config = NetworkConfig::default();
    let mut client = NetworkClient::new(config).await?;
    
    client.connect("127.0.0.1:7777", "PlayerName".to_string()).await?;
    
    loop {
        client.update(1.0 / 60.0)?;
        // Handle input, rendering, etc.
    }
}
```

## Architecture

The networking system is built in layers:

```
┌────────────────────────────────────────────────────────────────────┐
│ Network Architecture - Full Stack                                 │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│ Layer 5: Game Logic                                               │
├────────────────────────────────────────────────────────────────────┤
│  - Player input handling                                          │
│  - Game state updates                                             │
│  - Collision detection                                            │
│  - AI logic                                                        │
└───────────────────────┬────────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────────┐
│ Layer 4: Replication System                                       │
├────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │ Priority         │  │ Rate Limiting   │  │ Serialization   │  │
│  │ Management       │  │                 │  │ (Bincode)       │  │
│  │ - Player: 255    │  │ - Players: 60Hz │  │                 │  │
│  │ - NPC: 128       │  │ - NPCs: 30Hz    │  │ Component →     │  │
│  │ - Props: 64      │  │ - Props: 15Hz   │  │ Binary Data     │  │
│  └──────────────────┘  └─────────────────┘  └─────────────────┘  │
└───────────────────────┬────────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────────┐
│ Layer 3: Client-Side Prediction & Lag Compensation                │
├────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Client Prediction                                            │ │
│  │                                                               │ │
│  │  Local Input → Immediate Response → Reconcile with Server   │ │
│  │                                                               │ │
│  │  Timeline:                                                    │ │
│  │  T=0   Client moves (predict)                               │ │
│  │  T=50  Server receives input                                │ │
│  │  T=100 Server sends confirmed position                      │ │
│  │  T=150 Client reconciles (snap if error > threshold)        │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Lag Compensation (Server)                                    │ │
│  │                                                               │ │
│  │  Client fires at T=1000 (from client's perspective)         │ │
│  │  Server receives at T=1050 (50ms latency)                   │ │
│  │  Server rewinds world state to T=1000                       │ │
│  │  Validates hit detection at T=1000                          │ │
│  │  Restores to T=1050                                         │ │
│  │  Applies damage if hit confirmed                            │ │
│  └──────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────────┐
│ Layer 2: Interpolation & Extrapolation                            │
├────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Interpolation Buffer (Client)                                │ │
│  │                                                               │ │
│  │  Server Time:   S1──S2──S3──S4──S5  (snapshots)             │ │
│  │                  │   │   │   │   │                           │ │
│  │  Network ────────┼───┼───┼───┼───┼──────→                   │ │
│  │                  ▼   ▼   ▼   ▼   ▼                           │ │
│  │  Client Buffer: [S1][S2][S3][S4][S5]                        │ │
│  │                          ▲                                    │ │
│  │  Render Time: ───────────┘ (100ms behind)                   │ │
│  │                                                               │ │
│  │  Interpolate between S2 and S3 for smooth movement          │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Extrapolation (Packet Loss Fallback)                         │ │
│  │                                                               │ │
│  │  Last Known:     Position=(10, 0, 0), Velocity=(1, 0, 0)    │ │
│  │  Predict:        Position=Last + Velocity*DeltaTime          │ │
│  │  Max Duration:   500ms before snapping to last known        │ │
│  └──────────────────────────────────────────────────────────────┘ │
└───────────────────────┬────────────────────────────────────────────┘
                        │
                        ▼
┌────────────────────────────────────────────────────────────────────┐
│ Layer 1: Transport (TCP)                                          │
├────────────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐                    ┌──────────────────┐     │
│  │ Client           │                    │ Server           │     │
│  │                  │                    │                  │     │
│  │ Send:            │  ─────────────→    │ Receive:         │     │
│  │ - Input          │     (TCP)          │ - Process        │     │
│  │ - Timestamps     │                    │ - Validate       │     │
│  │                  │                    │ - Update World   │     │
│  │                  │                    │                  │     │
│  │ Receive:         │  ←─────────────    │ Send:            │     │
│  │ - World state    │     (TCP)          │ - Snapshots      │     │
│  │ - Entity updates │                    │ - Events         │     │
│  └──────────────────┘                    └──────────────────┘     │
│                                                                    │
│  TCP Characteristics:                                             │
│  ✓ Reliable delivery (retransmission)                            │
│  ✓ Ordered packets                                               │
│  ✓ Connection state tracking                                     │
│  ✗ Head-of-line blocking (one lost packet blocks all)           │
│  ✗ Higher latency than UDP                                       │
└────────────────────────────────────────────────────────────────────┘
```

### Transport Layer

**TCP Transport**
- Reliable, ordered delivery for critical data
- Connection handshake and authentication
- Automatic heartbeat to detect disconnections

**UDP Transport** (future)
- Fast, unreliable delivery for real-time position updates
- Custom reliability layer for important unreliable data
- Bandwidth optimization for high-frequency updates

**Why TCP First?**

The current implementation uses TCP for simplicity and reliability. UDP support is planned for bandwidth-sensitive games, but TCP works well for many game types:

- **Good for**: Turn-based games, MMOs, slower-paced games
- **Works with**: Fast games up to ~30 players with good netcode
- **Limitation**: Head-of-line blocking can cause latency spikes

### Replication System

The replication system synchronizes ECS components across the network:

```
┌─────────────────────────────────────────────────────────┐
│  Server                                                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  ECS World                                        │  │
│  │  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │  Entity 1   │  │  Entity 2   │               │  │
│  │  │  - NetworkId│  │  - NetworkId│               │  │
│  │  │  - Transform│  │  - Transform│               │  │
│  │  │  - Velocity │  │  - Health   │               │  │
│  │  └─────────────┘  └─────────────┘               │  │
│  └──────────────────────────────────────────────────┘  │
│             │                                            │
│             ▼                                            │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Replication System                               │  │
│  │  - Priority-based bandwidth allocation           │  │
│  │  - Rate limiting per entity                      │  │
│  │  - Binary serialization (bincode)               │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
                          │
                          │  Network
                          ▼
┌─────────────────────────────────────────────────────────┐
│  Client                                                  │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Interpolation Buffer                             │  │
│  │  - Stores snapshots                              │  │
│  │  - Configurable delay                            │  │
│  └──────────────────────────────────────────────────┘  │
│             │                                            │
│             ▼                                            │
│  ┌──────────────────────────────────────────────────┐  │
│  │  ECS World (Local Replica)                       │  │
│  │  ┌─────────────┐  ┌─────────────┐               │  │
│  │  │  Entity 1   │  │  Entity 2   │               │  │
│  │  │  - Transform│  │  - Transform│               │  │
│  │  │  (smooth)   │  │  (smooth)   │               │  │
│  │  └─────────────┘  └─────────────┘               │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Interpolation System

Clients interpolate between received snapshots for smooth movement:

```
Time ─────────────────────────────────────────────────>

Server:  S1───S2───S3───S4───S5  (snapshots sent)
          │    │    │    │    │
          ▼    ▼    ▼    ▼    ▼
Client:   R1───R2───R3───R4───R5  (snapshots received)
                    │
                    └── 100ms delay
                        │
Render:                 I3  (interpolate between R2-R3)
```

**Why Interpolation?**

Interpolation adds latency but provides smooth visuals. The delay allows buffering snapshots to interpolate between even if packets arrive out of order or late.

### Lag Compensation System

Server rewinds game state to client's perspective for hit detection:

```
Timeline:

Server Now (T=1000ms)
    │
    │  Client fires (T=1000ms client time)
    │  └─> 50ms latency
    │      └─> Server receives (T=1050ms server time)
    │
    └── Server rewinds to T=1000ms
        └── Validates hit from client's perspective
        └── Applies damage if hit confirmed
        └── Restores to T=1050ms

History Buffer: [T=0, T=16, T=33, T=50, ..., T=1000, T=1016, ...]
```

**Why Lag Compensation?**

Without lag compensation, players with high latency have a significant disadvantage in fast-paced games. Rewinding ensures hits register as the client saw them.

## Server Setup

### Basic Server

```rust
use praxis_networking::{NetworkServer, NetworkConfig, ReplicationRegistry};
use praxis_ecs::{World, Schedule};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    // Initialize world
    let mut world = World::new();
    
    // Configure server
    let config = NetworkConfig {
        bind_addr: "0.0.0.0:7777".to_string(),
        max_clients: 64,
        tick_rate: 60,
        enable_interpolation: true,
        enable_lag_compensation: true,
        ..Default::default()
    };
    
    // Create and start server
    let mut server = NetworkServer::new(config).await?;
    server.start().await?;
    
    tracing::info!("Server listening on port 7777");
    
    // Register components for replication
    let mut registry = ReplicationRegistry::new();
    registry.register_transform();
    registry.register_velocity();
    
    // Game loop
    let tick_duration = Duration::from_secs_f32(1.0 / 60.0);
    let mut last_tick = Instant::now();
    
    loop {
        let now = Instant::now();
        let delta = now.duration_since(last_tick).as_secs_f32();
        
        if delta >= tick_duration.as_secs_f32() {
            last_tick = now;
            
            // Update server
            server.update(delta)?;
            
            // Run game systems
            run_game_logic(&mut world, delta);
        }
        
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
```

### Spawning Replicated Entities

```rust
use praxis_networking::{NetworkId, Replicated, ReplicatedTransform};
use praxis_ecs::{Transform, GlobalTransform};
use praxis_math::{Vec3, Quat};

fn spawn_player(world: &mut World, network_id: u64, position: Vec3) {
    world.spawn((
        NetworkId::new(network_id),
        Replicated::new()
            .with_priority(255)  // Highest priority for players
            .with_rate_divisor(1),  // Update every tick
        Transform::from_translation(position),
        GlobalTransform::default(),
        ReplicatedTransform::new(position, Quat::IDENTITY, Vec3::ONE),
    ));
}

fn spawn_npc(world: &mut World, network_id: u64, position: Vec3) {
    world.spawn((
        NetworkId::new(network_id),
        Replicated::new()
            .with_priority(128)  // Medium priority
            .with_rate_divisor(2),  // Update every other tick
        Transform::from_translation(position),
        GlobalTransform::default(),
        ReplicatedTransform::new(position, Quat::IDENTITY, Vec3::ONE),
    ));
}
```

### Handling Client Connections

```rust
fn handle_connections(
    server: &mut NetworkServer,
    world: &mut World,
    player_spawner: &mut PlayerSpawner,
) {
    // Check for new connections
    for client_id in server.new_connections() {
        tracing::info!("Client {} connected", client_id);
        
        // Spawn player entity
        let spawn_pos = player_spawner.get_spawn_position();
        let entity = spawn_player(world, client_id, spawn_pos);
        
        // Associate client with entity
        player_spawner.register_player(client_id, entity);
    }
    
    // Check for disconnections
    for client_id in server.disconnections() {
        tracing::info!("Client {} disconnected", client_id);
        
        // Remove player entity
        if let Some(entity) = player_spawner.get_player_entity(client_id) {
            world.despawn(entity);
        }
        
        player_spawner.unregister_player(client_id);
    }
}
```

## Client Setup

### Basic Client

```rust
use praxis_networking::{NetworkClient, NetworkConfig};
use praxis_ecs::{World, Schedule};

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let mut world = World::new();
    
    // Configure client
    let config = NetworkConfig {
        enable_interpolation: true,
        enable_extrapolation: true,
        interpolation_delay_ms: 100,
        ..Default::default()
    };
    
    // Create and connect client
    let mut client = NetworkClient::new(config).await?;
    let player_name = "Player1".to_string();
    
    client.connect("127.0.0.1:7777", player_name).await?;
    
    tracing::info!("Connected to server");
    
    // Create schedule with interpolation systems
    let mut schedule = Schedule::default();
    schedule.add_systems(praxis_networking::interpolation_system);
    
    // Game loop
    let tick_duration = Duration::from_secs_f32(1.0 / 60.0);
    let mut last_tick = Instant::now();
    
    loop {
        let now = Instant::now();
        let delta = now.duration_since(last_tick).as_secs_f32();
        
        if delta >= tick_duration.as_secs_f32() {
            last_tick = now;
            
            // Update client
            client.update(delta)?;
            
            // Run client-side systems
            schedule.run(world.inner_mut());
            
            // Handle input, rendering, etc.
            handle_input(&mut world, &mut client, delta);
            render_frame(&world);
        }
        
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}
```

### Client Prediction

Predict local player movement while awaiting server confirmation:

```rust
#[derive(Component)]
struct LocalPlayer;

#[derive(Component)]
struct PredictedPosition {
    position: Vec3,
    velocity: Vec3,
}

fn predict_movement(
    mut query: Query<(&mut Transform, &PredictedPosition), With<LocalPlayer>>,
    delta: f32,
) {
    for (mut transform, predicted) in query.iter_mut() {
        // Apply prediction
        transform.translation = predicted.position + predicted.velocity * delta;
    }
}

fn reconcile_prediction(
    mut query: Query<(&mut Transform, &mut PredictedPosition), With<LocalPlayer>>,
    server_updates: &[ServerUpdate],
) {
    for (mut transform, mut predicted) in query.iter_mut() {
        // Check for server update
        if let Some(update) = server_updates.last() {
            // Snap to server position if too far off
            let error = (transform.translation - update.position).length();
            
            if error > 0.5 {
                transform.translation = update.position;
                predicted.position = update.position;
            } else {
                // Smoothly correct
                transform.translation = transform.translation.lerp(update.position, 0.1);
                predicted.position = update.position;
            }
        }
    }
}
```

### Handling Server Messages

```rust
fn process_server_messages(
    client: &mut NetworkClient,
    world: &mut World,
) {
    while let Some(message) = client.receive_message() {
        match message {
            ServerMessage::EntitySpawned { network_id, position } => {
                spawn_remote_entity(world, network_id, position);
            }
            ServerMessage::EntityDespawned { network_id } => {
                despawn_remote_entity(world, network_id);
            }
            ServerMessage::GameEvent { event_type, data } => {
                handle_game_event(world, event_type, data);
            }
            _ => {}
        }
    }
}
```

## Entity Replication

### Component Registration

Register any Serde-compatible component for network replication:

```rust
use serde::{Serialize, Deserialize};
use praxis_ecs::Component;

#[derive(Component, Serialize, Deserialize, Clone)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component, Serialize, Deserialize, Clone)]
struct Inventory {
    items: Vec<String>,
    capacity: usize,
}

let mut registry = ReplicationRegistry::new();
registry.register::<Health>("Health");
registry.register::<Inventory>("Inventory");
```

### Built-in Replicated Components

**ReplicatedTransform**

```rust
use praxis_networking::ReplicatedTransform;
use praxis_math::{Vec3, Quat};

let transform = ReplicatedTransform::new(
    Vec3::new(0.0, 5.0, 0.0),     // translation
    Quat::from_rotation_y(1.57),   // rotation
    Vec3::ONE,                     // scale
);
```

**ReplicatedVelocity**

```rust
use praxis_networking::ReplicatedVelocity;
use praxis_math::Vec3;

let velocity = ReplicatedVelocity::new(
    Vec3::new(1.0, 0.0, 0.0),  // linear velocity
    Vec3::new(0.0, 0.5, 0.0),  // angular velocity
);
```

### Priority System

Control bandwidth usage with component priorities:

```rust
// Critical entities (players, projectiles)
Replicated::new()
    .with_priority(255)      // Highest priority
    .with_rate_divisor(1);   // Update every tick

// Important entities (vehicles, NPCs)
Replicated::new()
    .with_priority(192)
    .with_rate_divisor(2);   // Update every 2 ticks

// Background entities (props, effects)
Replicated::new()
    .with_priority(64)
    .with_rate_divisor(4);   // Update every 4 ticks

// Static entities (buildings, terrain)
Replicated::new()
    .with_priority(32)
    .with_rate_divisor(10);  // Update every 10 ticks
```

### Relevance Filtering (Future)

```rust
// Future feature: Only replicate entities relevant to client

struct RelevanceFilter {
    max_distance: f32,
    interest_areas: Vec<BoundingBox>,
}

// Only send updates for entities near player
fn filter_relevant_entities(
    client_position: Vec3,
    filter: &RelevanceFilter,
    entities: &Query<(&Transform, &NetworkId)>,
) -> Vec<u64> {
    entities
        .iter()
        .filter(|(transform, _)| {
            transform.translation.distance(client_position) < filter.max_distance
        })
        .map(|(_, network_id)| network_id.0)
        .collect()
}
```

## Interpolation & Extrapolation

### Setting Up Interpolation

```rust
use praxis_networking::{
    NetworkInterpolation, InterpolationBuffer, InterpolationSystem,
};

// Add to remote entities
world.spawn((
    NetworkId::new(2),
    ReplicatedTransform::default(),
    NetworkInterpolation::new(100.0),  // 100ms interpolation delay
    InterpolationBuffer::default(),
));

// Run system each frame
fn update_interpolation(
    mut query: Query<(
        &mut Transform,
        &NetworkInterpolation,
        &InterpolationBuffer,
    )>,
    delta: f32,
) {
    InterpolationSystem::update(query, delta);
}
```

### Configuring Interpolation Delay

```rust
// Fast-paced games: Lower delay for responsiveness
NetworkInterpolation::new(50.0);   // 50ms

// Slow-paced games: Higher delay for smoothness
NetworkInterpolation::new(200.0);  // 200ms

// Adaptive: Adjust based on network conditions
fn adaptive_interpolation_delay(jitter: f32, packet_loss: f32) -> f32 {
    let base_delay = 100.0;
    let jitter_compensation = jitter * 2.0;
    let loss_compensation = packet_loss * 500.0;
    
    (base_delay + jitter_compensation + loss_compensation).clamp(50.0, 300.0)
}
```

### Extrapolation Setup

Extrapolation predicts entity movement when updates are delayed:

```rust
use praxis_networking::{NetworkExtrapolation, ExtrapolationSystem};

// Add to entities that might need prediction
world.spawn((
    NetworkId::new(3),
    ReplicatedTransform::default(),
    ReplicatedVelocity::default(),
    NetworkExtrapolation::new(500.0),  // Max 500ms extrapolation
    InterpolationBuffer::default(),
));

// Run system when packets are delayed
fn update_extrapolation(
    mut query: Query<(
        &mut Transform,
        &ReplicatedVelocity,
        &NetworkExtrapolation,
    )>,
    delta: f32,
) {
    ExtrapolationSystem::update(query, delta);
}
```

### Interpolation vs Extrapolation

**Interpolation** (past):
- Renders entity where it was 100ms ago
- Smooth, accurate, no prediction errors
- Adds visual latency

**Extrapolation** (future):
- Predicts where entity will be
- More responsive, no added latency
- Can have prediction errors (snapping)

**When to Use**:
- Interpolation: Other players, NPCs, vehicles
- Extrapolation: Fast projectiles, temporary packet loss fallback
- Neither: Local player (use client prediction)

## Lag Compensation

### Basic Setup

```rust
use praxis_networking::{LagCompensation, LagCompensationSystem};

// Initialize with history length
let mut lag_comp = LagCompensation::new(1000); // 1 second of history

// Record snapshots each tick
fn record_history(
    mut lag_comp: ResMut<LagCompensation>,
    world: &World,
    client_id: u64,
) {
    LagCompensationSystem::update(&mut lag_comp, client_id, world);
}
```

### Lag-Compensated Raycasting

```rust
use praxis_math::{Vec3, Ray};

fn handle_player_shoot(
    client_id: u64,
    client_timestamp: u64,
    ray_origin: Vec3,
    ray_direction: Vec3,
    lag_comp: &mut LagCompensation,
    world: &mut World,
) -> Result<(), NetworkError> {
    // Perform raycast at client's time
    let max_distance = 100.0;
    
    match lag_comp.raycast_at_client_time(
        client_id,
        client_timestamp,
        world,
        ray_origin,
        ray_direction,
        max_distance,
    )? {
        Some(hit) => {
            tracing::info!("Hit entity {:?} at distance {}", hit.entity, hit.distance);
            
            // Apply damage
            if let Some(mut health) = world.get_mut::<Health>(hit.entity) {
                health.current -= 25.0;
            }
        }
        None => {
            tracing::debug!("Shot missed");
        }
    }
    
    Ok(())
}
```

### Manual Rewind

For custom hit detection logic:

```rust
fn custom_hit_detection(
    client_id: u64,
    client_timestamp: u64,
    attack_area: BoundingBox,
    lag_comp: &mut LagCompensation,
    world: &mut World,
) -> Result<Vec<Entity>, NetworkError> {
    // Rewind world to client's perspective
    let rewind_state = lag_comp.rewind_to_client_time(
        client_id,
        client_timestamp,
        world,
    )?;
    
    // Perform custom hit detection at rewound time
    let mut hit_entities = Vec::new();
    
    for (entity, transform) in world.query::<(&Transform,)>().iter() {
        if attack_area.contains(transform.translation) {
            hit_entities.push(entity);
        }
    }
    
    // Restore world to current time
    lag_comp.restore_state(rewind_state, world);
    
    Ok(hit_entities)
}
```

### Lag Compensation Limits

```rust
let config = NetworkConfig {
    lag_compensation_history_ms: 1000,  // Max rewind distance
    max_client_latency_ms: 500,         // Reject clients over threshold
    ..Default::default()
};

// Validation
fn validate_shot(
    client_latency: f32,
    timestamp_age: f32,
    config: &NetworkConfig,
) -> bool {
    // Reject if latency too high
    if client_latency > config.max_client_latency_ms as f32 {
        return false;
    }
    
    // Reject if timestamp too old
    if timestamp_age > config.lag_compensation_history_ms as f32 {
        return false;
    }
    
    true
}
```

## Network Profiling

### Enable Profiling

```rust
let config = NetworkConfig {
    enable_profiling: true,
    ..Default::default()
};

let profiler = NetworkProfiler::new();
```

### Recording Metrics

```rust
// Record bandwidth
profiler.record_sent(packet_size);
profiler.record_received(packet_size);

// Record latency
profiler.record_latency(rtt_ms);

// Record by message type
profiler.record_message_type("EntityUpdate", packet_size);

// Update profiler each frame
profiler.update(delta_time);
```

### Reading Statistics

```rust
let stats = profiler.get_stats();

println!("=== Network Statistics ===");
println!("Bandwidth:");
println!("  Send: {:.2} KB/s", stats.bandwidth.send_rate / 1024.0);
println!("  Recv: {:.2} KB/s", stats.bandwidth.receive_rate / 1024.0);
println!("  Peak send: {:.2} KB/s", stats.bandwidth.peak_send_rate / 1024.0);
println!();
println!("Latency:");
println!("  Current RTT: {:.1} ms", stats.latency.rtt_ms);
println!("  Average RTT: {:.1} ms", stats.latency.avg_rtt_ms);
println!("  Jitter: {:.1} ms", stats.latency.jitter_ms);
println!();
println!("Packets:");
println!("  Sent: {}", stats.bandwidth.packets_sent);
println!("  Received: {}", stats.bandwidth.packets_received);
```

### Debug UI

```rust
use egui::{Context, Window};

fn draw_network_stats(ctx: &Context, profiler: &NetworkProfiler) {
    Window::new("Network Stats").show(ctx, |ui| {
        let stats = profiler.get_stats();
        
        ui.label(format!("RTT: {:.1} ms", stats.latency.rtt_ms));
        ui.label(format!("Jitter: {:.1} ms", stats.latency.jitter_ms));
        ui.separator();
        ui.label(format!("Send: {:.2} KB/s", stats.bandwidth.send_rate / 1024.0));
        ui.label(format!("Recv: {:.2} KB/s", stats.bandwidth.receive_rate / 1024.0));
        
        // Graph (simplified)
        let points: Vec<[f64; 2]> = profiler
            .get_latency_history()
            .iter()
            .enumerate()
            .map(|(i, &lat)| [i as f64, lat as f64])
            .collect();
            
        // Use egui plot or similar
    });
}
```

## Configuration

### Network Config Reference

```rust
let config = NetworkConfig {
    // Connection
    bind_addr: "0.0.0.0:7777".to_string(),
    max_clients: 32,
    connection_timeout_ms: 10000,
    
    // Simulation
    tick_rate: 60,
    max_packet_size: 1400,
    
    // Interpolation
    enable_interpolation: true,
    interpolation_delay_ms: 100,
    
    // Extrapolation
    enable_extrapolation: true,
    extrapolation_limit_ms: 500,
    
    // Lag Compensation
    enable_lag_compensation: true,
    lag_compensation_history_ms: 1000,
    max_client_latency_ms: 500,
    
    // Profiling
    enable_profiling: true,
    profiling_sample_rate: 1.0,
};
```

### Per-Game Type Configs

**Fast-Paced FPS**:
```rust
NetworkConfig {
    tick_rate: 128,                    // High tick rate
    interpolation_delay_ms: 50,        // Low delay
    enable_lag_compensation: true,
    lag_compensation_history_ms: 1000,
    ..Default::default()
}
```

**MMO/Strategy**:
```rust
NetworkConfig {
    tick_rate: 20,                     // Lower tick rate
    interpolation_delay_ms: 200,       // Higher delay OK
    enable_lag_compensation: false,    // Not needed
    max_clients: 100,                  // More players
    ..Default::default()
}
```

**Racing Game**:
```rust
NetworkConfig {
    tick_rate: 60,
    interpolation_delay_ms: 100,
    enable_extrapolation: true,        // Smooth vehicle movement
    extrapolation_limit_ms: 200,
    ..Default::default()
}
```

## Common Patterns

### Player Spawning

```rust
struct PlayerManager {
    players: HashMap<u64, Entity>,
    spawn_points: Vec<Vec3>,
}

impl PlayerManager {
    fn on_client_connected(
        &mut self,
        client_id: u64,
        world: &mut World,
    ) -> Entity {
        let spawn_pos = self.get_available_spawn_point();
        
        let entity = world.spawn((
            NetworkId::new(client_id),
            Replicated::new().with_priority(255),
            Transform::from_translation(spawn_pos),
            GlobalTransform::default(),
            ReplicatedTransform::new(spawn_pos, Quat::IDENTITY, Vec3::ONE),
            PlayerController::default(),
            Health::new(100.0),
        ));
        
        self.players.insert(client_id, entity);
        entity
    }
    
    fn on_client_disconnected(&mut self, client_id: u64, world: &mut World) {
        if let Some(entity) = self.players.remove(&client_id) {
            world.despawn(entity);
        }
    }
}
```

### Input Handling

```rust
#[derive(Serialize, Deserialize)]
struct PlayerInput {
    movement: Vec3,
    look: Vec2,
    jump: bool,
    shoot: bool,
    timestamp: u64,
}

// Client
fn send_input(client: &mut NetworkClient, input: PlayerInput) {
    client.send_message(ClientMessage::Input(input));
}

// Server
fn process_input(
    server: &mut NetworkServer,
    world: &mut World,
) {
    for (client_id, message) in server.receive_messages() {
        if let ClientMessage::Input(input) = message {
            apply_input(world, client_id, input);
        }
    }
}

fn apply_input(world: &mut World, client_id: u64, input: PlayerInput) {
    // Find player entity
    let player = find_player_by_client_id(world, client_id);
    
    // Apply movement
    if let Some(mut transform) = world.get_mut::<Transform>(player) {
        let speed = 5.0;
        transform.translation += input.movement.normalize_or_zero() * speed;
    }
    
    // Handle actions
    if input.shoot {
        spawn_projectile(world, player);
    }
}
```

### Chat System

```rust
#[derive(Serialize, Deserialize)]
enum ChatMessage {
    Global { sender: String, text: String },
    Team { team_id: u32, sender: String, text: String },
    Private { recipient: u64, sender: String, text: String },
}

// Client sends chat
client.send_message(ClientMessage::Chat {
    message: "Hello!".to_string(),
});

// Server broadcasts to all clients
fn handle_chat(
    server: &mut NetworkServer,
    sender_id: u64,
    message: String,
) {
    let sender_name = server.get_client_name(sender_id);
    
    let chat_msg = ChatMessage::Global {
        sender: sender_name,
        text: message,
    };
    
    server.broadcast(ServerMessage::Chat(chat_msg));
}
```

### Game State Synchronization

```rust
#[derive(Serialize, Deserialize)]
struct GameState {
    match_time: f32,
    score: HashMap<u64, u32>,
    phase: GamePhase,
}

#[derive(Serialize, Deserialize)]
enum GamePhase {
    Waiting,
    Starting,
    InProgress,
    Ending,
}

// Server updates
fn sync_game_state(
    server: &mut NetworkServer,
    state: &GameState,
) {
    server.broadcast(ServerMessage::GameState(state.clone()));
}

// Client receives
fn on_game_state_update(state: GameState) {
    match state.phase {
        GamePhase::Starting => start_countdown(),
        GamePhase::InProgress => enable_gameplay(),
        GamePhase::Ending => show_results(&state.score),
        _ => {}
    }
}
```

### Projectile Replication

```rust
fn spawn_projectile(
    world: &mut World,
    owner: Entity,
    position: Vec3,
    direction: Vec3,
) -> Entity {
    let network_id = generate_network_id();
    let velocity = direction * 50.0;
    
    world.spawn((
        NetworkId::new(network_id),
        Replicated::new()
            .with_priority(200)  // High priority
            .with_rate_divisor(1),
        Transform::from_translation(position),
        GlobalTransform::default(),
        ReplicatedTransform::new(position, Quat::IDENTITY, Vec3::ONE),
        ReplicatedVelocity::new(velocity, Vec3::ZERO),
        Projectile { owner, damage: 25.0 },
        Lifetime::new(5.0),  // Despawn after 5 seconds
    ))
}
```

## Best Practices

### 1. Component Selection

**Do replicate**:
- Transform (position, rotation)
- Velocity (for interpolation/extrapolation)
- Health, armor, status
- Animation state
- Player input (server-side)

**Don't replicate**:
- Render data (mesh, materials)
- Audio data
- UI state
- Local effects
- Debug visualization

### 2. Bandwidth Management

```rust
// Priority hierarchy
const PRIORITY_PLAYER: u8 = 255;
const PRIORITY_PROJECTILE: u8 = 200;
const PRIORITY_VEHICLE: u8 = 180;
const PRIORITY_NPC: u8 = 128;
const PRIORITY_PROP: u8 = 64;
const PRIORITY_EFFECT: u8 = 32;

// Rate control
fn assign_update_rate(entity_type: EntityType) -> u32 {
    match entity_type {
        EntityType::Player => 1,      // Every tick
        EntityType::Vehicle => 1,     // Every tick
        EntityType::Projectile => 1,  // Every tick
        EntityType::NPC => 2,         // Every 2 ticks
        EntityType::Prop => 4,        // Every 4 ticks
        EntityType::Static => 10,     // Every 10 ticks
    }
}
```

### 3. Interpolation Delay Selection

```rust
// Measure typical RTT and jitter
fn measure_network_conditions(profiler: &NetworkProfiler) -> (f32, f32) {
    let stats = profiler.get_stats();
    (stats.latency.avg_rtt_ms, stats.latency.jitter_ms)
}

// Set delay to accommodate network conditions
fn calculate_interpolation_delay(rtt: f32, jitter: f32) -> f32 {
    // Delay should be at least RTT/2 + 2*jitter
    let min_delay = (rtt / 2.0) + (2.0 * jitter);
    
    // Clamp to reasonable range
    min_delay.clamp(50.0, 200.0)
}
```

### 4. Authority Model

**Server is authoritative** for:
- Player positions (after validation)
- Health, damage, death
- Item spawning and collection
- Game state transitions
- Score, timers

**Client is authoritative** for:
- Input (send to server)
- Camera position
- UI state
- Visual effects
- Audio playback

### 5. Security Considerations

```rust
// Validate client input
fn validate_movement(
    old_pos: Vec3,
    new_pos: Vec3,
    max_speed: f32,
    delta: f32,
) -> bool {
    let distance = old_pos.distance(new_pos);
    let max_distance = max_speed * delta * 1.1; // 10% tolerance
    
    distance <= max_distance
}

// Rate limiting
struct RateLimiter {
    actions: HashMap<u64, Vec<Instant>>,
    max_actions_per_second: u32,
}

impl RateLimiter {
    fn check_rate(&mut self, client_id: u64) -> bool {
        let now = Instant::now();
        let history = self.actions.entry(client_id).or_default();
        
        // Remove old entries
        history.retain(|&t| now.duration_since(t).as_secs() < 1);
        
        if history.len() >= self.max_actions_per_second as usize {
            return false;
        }
        
        history.push(now);
        true
    }
}
```

### 6. Graceful Degradation

```rust
// Adjust quality based on network conditions
fn adapt_to_network(
    stats: &NetworkStats,
    config: &mut NetworkConfig,
) {
    // High latency: Increase interpolation delay
    if stats.latency.avg_rtt_ms > 150.0 {
        config.interpolation_delay_ms = 150;
    }
    
    // High jitter: Enable extrapolation
    if stats.latency.jitter_ms > 30.0 {
        config.enable_extrapolation = true;
    }
    
    // High bandwidth: Reduce update rate
    if stats.bandwidth.send_rate > 100_000.0 {
        // Reduce rate for lower-priority entities
        reduce_update_rates();
    }
}
```

## Troubleshooting

### Entities Not Syncing

**Symptoms**: Entities spawn on server but not on clients

**Solutions**:
1. Verify `NetworkId` component is present
2. Check `Replicated` component is attached
3. Ensure `ReplicationRegistry` has registered component types
4. Check network connection is active
5. Verify server is calling `server.update()`

```rust
// Debug: Log entity state
fn debug_replication(entity: Entity, world: &World) {
    if let Some(net_id) = world.get::<NetworkId>(entity) {
        tracing::debug!("Entity {:?} has NetworkId: {}", entity, net_id.0);
    } else {
        tracing::warn!("Entity {:?} missing NetworkId!", entity);
    }
    
    if let Some(replicated) = world.get::<Replicated>(entity) {
        tracing::debug!("Entity {:?} replication config: priority={}, rate={}",
            entity, replicated.priority, replicated.rate_divisor);
    } else {
        tracing::warn!("Entity {:?} missing Replicated component!", entity);
    }
}
```

### Stuttering Movement

**Symptoms**: Remote entities jerk or stutter during movement

**Solutions**:
1. **Increase interpolation delay**: More buffer time for smooth interpolation
2. **Enable extrapolation**: Predict movement during packet loss
3. **Check tick rate**: Ensure consistent server tick rate
4. **Verify update rate**: Entities might be updating too infrequently

```rust
// Increase delay
config.interpolation_delay_ms = 150; // Was 100

// Enable extrapolation
config.enable_extrapolation = true;

// Check tick rate consistency
let tick_variance = measure_tick_variance();
if tick_variance > 5.0 {
    tracing::warn!("Server tick rate unstable: {}ms variance", tick_variance);
}
```

### High Bandwidth Usage

**Symptoms**: Network saturated, clients lag or disconnect

**Solutions**:
1. **Reduce entity count**: Use relevance filtering
2. **Lower priority**: Reduce priority of background entities
3. **Increase rate divisor**: Update less important entities less often
4. **Delta compression** (future): Only send changed components

```rust
// Reduce update frequency for distant entities
fn adjust_replication_rate(
    player_pos: Vec3,
    mut query: Query<(&Transform, &mut Replicated)>,
) {
    for (transform, mut replicated) in query.iter_mut() {
        let distance = transform.translation.distance(player_pos);
        
        if distance < 20.0 {
            replicated.rate_divisor = 1;  // Close: every tick
        } else if distance < 50.0 {
            replicated.rate_divisor = 2;  // Medium: every 2 ticks
        } else {
            replicated.rate_divisor = 4;  // Far: every 4 ticks
        }
    }
}
```

### Lag Compensation Not Working

**Symptoms**: Hits don't register or feel unfair

**Solutions**:
1. **Check history buffer size**: Must be larger than max RTT
2. **Verify timestamps**: Client must send accurate timestamps
3. **Validate rewind distance**: Don't rewind too far
4. **Test manually**: Use debug visualization

```rust
// Debug lag compensation
fn debug_lag_compensation(
    lag_comp: &LagCompensation,
    client_id: u64,
    timestamp: u64,
) {
    let current_time = get_current_time();
    let age = current_time - timestamp;
    
    tracing::debug!("Lag compensation rewind:");
    tracing::debug!("  Client ID: {}", client_id);
    tracing::debug!("  Timestamp: {}", timestamp);
    tracing::debug!("  Age: {} ms", age);
    
    if age > 1000 {
        tracing::warn!("Timestamp too old for lag compensation!");
    }
}

// Visualize rewound positions
fn visualize_rewind(world: &World, debug_lines: &mut DebugLines) {
    for (entity, transform) in world.query::<(&Transform,)>().iter() {
        // Draw current position in green
        debug_lines.sphere(transform.translation, 0.5, Color::GREEN);
        
        // Draw rewound position in red (if available)
        // ...
    }
}
```

### Connection Issues

**Symptoms**: Clients can't connect or frequently disconnect

**Solutions**:
1. **Check firewall**: Ensure port is open
2. **Verify bind address**: Use `0.0.0.0` not `127.0.0.1` for public servers
3. **Test locally first**: Ensure server works on localhost
4. **Check timeout settings**: Increase if clients on slow connections

```rust
// Test connection
async fn test_connection(addr: &str) -> bool {
    match TcpStream::connect(addr).await {
        Ok(_) => {
            tracing::info!("Successfully connected to {}", addr);
            true
        }
        Err(e) => {
            tracing::error!("Failed to connect to {}: {}", addr, e);
            false
        }
    }
}

// Increase timeouts for slow connections
config.connection_timeout_ms = 20000; // 20 seconds
```

## Examples

See working examples:
- `examples/networking_demo.rs` - Complete client-server demo
- Run server: `cargo run --example networking_demo -- server`
- Run client: `cargo run --example networking_demo -- client`

## See Also

- [praxis_networking README](../../../crates/praxis_networking/README.md) - API documentation
- [ECS Guide](../../concepts/ecs.md) - Understanding ECS for networking
- [Physics Guide](../physics.md) - Combining physics with networking
- [Scripting Guide](../scripting.md) - Server-side game logic

## Further Reading

- [Source Multiplayer Networking](https://developer.valvesoftware.com/wiki/Source_Multiplayer_Networking) - Valve's authoritative guide
- [Gaffer on Games](https://gafferongames.com/) - Networking articles
- [Fast-Paced Multiplayer](https://www.gabrielgambetta.com/client-server-game-architecture.html) - Gabriel Gambetta's series
