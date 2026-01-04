# ECS Architecture

Entity-Component-System (ECS) is the core architectural pattern in Praxis, powered by `bevy_ecs`.

## Core Concepts

### Entities
Unique identifiers (IDs) representing game objects. Entities have no data or behavior themselves—they're just handles.

```rust
let entity = world.spawn_empty().id();
```

### Components
Pure data attached to entities. Components should be small, focused, and contain no logic.

```rust
#[derive(Component)]
struct Position(Vec3);

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Health(f32);
```

### Systems
Functions that process entities with specific component combinations. Systems contain all the logic.

```rust
fn movement_system(mut query: Query<(&mut Position, &Velocity)>) {
    for (mut pos, vel) in query.iter_mut() {
        pos.0 += vel.0;
    }
}
```

## Why ECS?

### Data-Oriented Design
- Components are stored contiguously in memory
- Cache-friendly iteration patterns
- Predictable performance characteristics

### Composition Over Inheritance
- Build complex behaviors by combining simple components
- No deep inheritance hierarchies
- Easy to add/remove capabilities at runtime

### Parallelism
- Systems with non-overlapping data access run in parallel
- Automatic scheduling based on data dependencies
- Scales with CPU cores

## Praxis ECS Patterns

### Component Bundles
Group related components for convenient spawning:

```rust
#[derive(Bundle)]
struct PlayerBundle {
    transform: Transform,
    velocity: Velocity,
    health: Health,
    player: Player,
}

world.spawn(PlayerBundle { ... });
```

### Resources
Global singletons accessible to all systems:

```rust
#[derive(Resource)]
struct GameTime {
    delta: f32,
    elapsed: f32,
}

fn timer_system(time: Res<GameTime>, mut query: Query<&mut Cooldown>) {
    for mut cooldown in query.iter_mut() {
        cooldown.remaining -= time.delta;
    }
}
```

### Change Detection
React only when data changes:

```rust
fn on_health_changed(query: Query<&Health, Changed<Health>>) {
    for health in query.iter() {
        println!("Health changed to {}", health.0);
    }
}
```

### Events
Decouple systems with message passing:

```rust
#[derive(Event)]
struct DamageEvent {
    target: Entity,
    amount: f32,
}

fn damage_system(mut events: EventReader<DamageEvent>, mut health: Query<&mut Health>) {
    for event in events.read() {
        if let Ok(mut hp) = health.get_mut(event.target) {
            hp.0 -= event.amount;
        }
    }
}
```

## Common Patterns in Praxis

### Transform Hierarchy
Parent-child relationships with automatic transform propagation:
- `Transform`: Local position, rotation, scale
- `GlobalTransform`: Computed world-space transform
- `Parent`/`Children`: Hierarchy relationships

### Marker Components
Tag entities for system filtering:

```rust
#[derive(Component)]
struct Player;  // No data, just a marker

fn player_system(query: Query<&Transform, With<Player>>) { ... }
```

### Optional Components
Handle entities that may or may not have certain components:

```rust
fn render_system(query: Query<(&Transform, Option<&Material>)>) {
    for (transform, material) in query.iter() {
        let mat = material.unwrap_or(&DEFAULT_MATERIAL);
        // render...
    }
}
```

## See Also

- [BEGINNERS_GUIDE - ECS Data Flow](../BEGINNERS_GUIDE.md#ecs-data-flow) - Deep dive with diagrams
- [praxis_ecs crate](../../crates/praxis_ecs/README.md) - API documentation
- [Transform Hierarchy](transform-hierarchy.md) - Scene graph concepts
- [bevy_ecs documentation](https://docs.rs/bevy_ecs) - Underlying ECS library
