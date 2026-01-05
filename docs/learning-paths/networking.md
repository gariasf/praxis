# Networking Learning Path

Build multiplayer games with client-server architecture, entity replication, and lag compensation.

## Path Overview

**Time Investment**: 2-3 weeks  
**Prerequisites**: Understanding of ECS and async programming  
**Final Goal**: Production-ready multiplayer game systems

## Progression Map

```
Beginner (1 week)
├── Client-server architecture
├── Connection management
├── Basic messaging
└── Network configuration
    ↓
Intermediate (1 week)
├── Entity replication
├── Component synchronization
├── Interpolation/extrapolation
└── Bandwidth optimization
    ↓
Advanced (1 week)
├── Lag compensation
├── Client prediction
├── Network profiling
└── Production deployment
```

---

## Beginner: Client-Server Setup

### Prerequisites
- ✓ Understanding of async/await (Tokio)
- ✓ ECS concepts
- ✓ Basic networking knowledge

### Learning Path

**Theory** (3-4 hours):
1. Read [Networking Guide](../guides/systems/networking.md) - Overview & Architecture
2. Read `crates/praxis_networking/README.md`
3. Understand client-server model

**Practice** (6-8 hours):
1. Setup server
2. Setup client
3. Establish connection
4. Send basic messages

**Example**:
```rust
// Server
let config = NetworkConfig {
    bind_addr: "0.0.0.0:7777".to_string(),
    max_clients: 32,
    tick_rate: 60,
    ..Default::default()
};
let mut server = NetworkServer::new(config).await?;
server.start().await?;

// Client
let mut client = NetworkClient::new(config).await?;
client.connect("127.0.0.1:7777", "Player1".to_string()).await?;
```

**Run Example**:
```bash
cargo run --example networking_demo
```

### Checkpoint
- [ ] Server accepts connections
- [ ] Client connects successfully
- [ ] Messages sent/received
- [ ] Understand TCP vs UDP

**Time**: 10-15 hours

---

## Intermediate: Entity Replication

### Prerequisites
- ✓ Completed Beginner section
- ✓ Comfortable with ECS

### Learning Path

**Theory** (3-4 hours):
1. Continue [Networking Guide: Entity Replication](../guides/systems/networking.md#entity-replication)
2. Understand component registration
3. Learn synchronization strategies

**Practice** (8-10 hours):
1. Register components for replication
2. Implement automatic sync
3. Handle entity spawning/despawning
4. Optimize bandwidth

**Pattern**:
```rust
// Register components for replication
let mut registry = ReplicationRegistry::new();
registry.register_transform();
registry.register_velocity();
registry.register_health();

// Entities with replicated components sync automatically
world.spawn((
    Transform::from_xyz(0.0, 5.0, 0.0),
    Velocity::default(),
    Health::new(100),
    Replicated,  // Mark for replication
));
```

**Interpolation**:
```rust
// Smooth remote entity movement
let config = InterpolationConfig {
    buffer_time: 0.1,  // 100ms buffer
    extrapolation_limit: 0.05,
    ..Default::default()
};
```

### Checkpoint
- [ ] Entities replicate across network
- [ ] Movement is smooth (interpolation)
- [ ] Bandwidth is reasonable
- [ ] Late-join clients sync state

**Time**: 15-20 hours

---

## Advanced: Lag Compensation

### Prerequisites
- ✓ Completed Intermediate section
- ✓ Understanding of networked game challenges

### Learning Path

**Theory** (4-5 hours):
1. Continue [Networking Guide: Lag Compensation](../guides/systems/networking.md#lag-compensation)
2. Study server-side rewind
3. Learn client prediction

**Practice** (10-12 hours):
1. Implement lag compensation
2. Server-side hit detection
3. Client prediction
4. Network profiling

**Lag Compensation**:
```rust
// Server rewinds to client's view of the world
let hit = lag_compensator.check_hit(
    shooter_client_id,
    ray_origin,
    ray_direction,
    &physics_world,
);
```

**Client Prediction**:
```rust
// Client predicts own movement
if is_local_player {
    apply_input_immediately();
    predicted_state.push(current_state);
}

// Server correction
if let Some(authoritative_state) = server_update {
    reconcile_predicted_states(authoritative_state);
}
```

**Network Profiler**:
```rust
let profiler = NetworkProfiler::new();
profiler.enable();

// Monitor in real-time
let stats = profiler.get_stats();
println!("Bandwidth: {:.2} KB/s", stats.bandwidth_kbps);
println!("Latency: {:.1}ms", stats.latency_ms);
println!("Packet loss: {:.1}%", stats.packet_loss_percent);
```

### Checkpoint
- [ ] Lag compensation working
- [ ] Fair hit detection
- [ ] Client prediction implemented
- [ ] Network profiler integrated
- [ ] Optimized for various network conditions

**Time**: 20-25 hours

---

## Cross-References

### Related Systems
- [Physics Path](physics.md) - Physics replication
- [Scripting Path](scripting.md) - Network game logic
- [Animation Path](animation.md) - Replicate animations

### Performance
- [Profiling](../profiling.md) - Network performance

---

## Practice Resources

```bash
cargo run --example networking_demo
```

### Testing Setup
1. Run server: `cargo run --bin game_server`
2. Run client 1: `cargo run --bin game_client`
3. Run client 2: `cargo run --bin game_client`
4. Test with artificial lag

---

## Next Steps

1. **Specialize**: Voice chat, dedicated servers
2. **Scale**: MMO-style networking
3. **Secure**: Anti-cheat systems

---

[← Back to Learning Paths](README.md) | [Next: Audio Path →](audio.md)
