# ECS Architecture

Entity-Component-System (ECS) is the core architectural pattern in Praxis, powered by `bevy_ecs`.

**Related Architecture Documentation:**
- [ECS System Execution Order](../architecture/ecs-system-execution-order.md) - Visual guide to system scheduling and data flow
- [ECS Design Patterns](../architecture/ecs-patterns.md) - Common patterns and best practices

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

## ECS Data Flow

```
┌────────────────────────────────────────────────────────────────────┐
│ World (ECS Container)                                              │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Entities (IDs):  [1, 2, 3, 4, 5, ...]                           │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │ Component Storage (Table-based)                          │    │
│  │                                                           │    │
│  │  Position Table:                                         │    │
│  │  ┌────────┬──────────────┐                              │    │
│  │  │ Entity │ Position     │                              │    │
│  │  ├────────┼──────────────┤                              │    │
│  │  │   1    │ (0, 0, 0)    │  ◄─ Contiguous memory       │    │
│  │  │   2    │ (1, 2, 3)    │  ◄─ Cache-friendly          │    │
│  │  │   4    │ (5, 0, -2)   │                              │    │
│  │  └────────┴──────────────┘                              │    │
│  │                                                           │    │
│  │  Velocity Table:                                         │    │
│  │  ┌────────┬──────────────┐                              │    │
│  │  │ Entity │ Velocity     │                              │    │
│  │  ├────────┼──────────────┤                              │    │
│  │  │   1    │ (1, 0, 0)    │                              │    │
│  │  │   2    │ (0, 1, 0)    │                              │    │
│  │  └────────┴──────────────┘                              │    │
│  │                                                           │    │
│  │  Health Table:                                           │    │
│  │  ┌────────┬──────────┐                                  │    │
│  │  │ Entity │ Health   │                                  │    │
│  │  ├────────┼──────────┤                                  │    │
│  │  │   1    │ 100.0    │                                  │    │
│  │  │   3    │ 75.0     │                                  │    │
│  │  │   4    │ 50.0     │                                  │    │
│  │  └────────┴──────────┘                                  │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐    │
│  │ Resources (Global Singletons)                            │    │
│  │                                                           │    │
│  │  - DeltaTime: 0.016                                     │    │
│  │  - PhysicsWorld: { ... }                                │    │
│  │  - RenderContext: { ... }                               │    │
│  └──────────────────────────────────────────────────────────┘    │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌────────────────────────────────────────────────────────────────────┐
│ Systems (Logic)                                                    │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌──────────────────────────────────────────────────────┐         │
│  │ System 1: Movement                                   │         │
│  │                                                       │         │
│  │  Query: (&mut Position, &Velocity)                   │         │
│  │         └─ Only entities with BOTH components        │         │
│  │                                                       │         │
│  │  Matched Entities: [1, 2]                            │         │
│  │                                                       │         │
│  │  for (pos, vel) in query.iter_mut():                 │         │
│  │    pos += vel * delta_time                           │         │
│  │                                                       │         │
│  │  ┌────────────────────────────────────┐              │         │
│  │  │ Processes:                         │              │         │
│  │  │  Entity 1: (0,0,0) + (1,0,0)*0.016│              │         │
│  │  │          = (0.016, 0, 0)           │              │         │
│  │  │  Entity 2: (1,2,3) + (0,1,0)*0.016│              │         │
│  │  │          = (1, 2.016, 3)           │              │         │
│  │  └────────────────────────────────────┘              │         │
│  └──────────────────────────────────────────────────────┘         │
│                                                                    │
│  ┌──────────────────────────────────────────────────────┐         │
│  │ System 2: Health Regeneration                        │         │
│  │                                                       │         │
│  │  Query: &mut Health                                  │         │
│  │                                                       │         │
│  │  Matched Entities: [1, 3, 4]                         │         │
│  │                                                       │         │
│  │  for health in query.iter_mut():                     │         │
│  │    if health < 100.0:                                │         │
│  │      health += 5.0 * delta_time                      │         │
│  │                                                       │         │
│  │  ┌────────────────────────────────────┐              │         │
│  │  │ Processes:                         │              │         │
│  │  │  Entity 1: 100.0 (no change)       │              │         │
│  │  │  Entity 3: 75.0 + 5.0*0.016        │              │         │
│  │  │          = 75.08                   │              │         │
│  │  │  Entity 4: 50.0 + 5.0*0.016        │              │         │
│  │  │          = 50.08                   │              │         │
│  │  └────────────────────────────────────┘              │         │
│  └──────────────────────────────────────────────────────┘         │
│                                                                    │
│  ┌──────────────────────────────────────────────────────┐         │
│  │ System 3: Render                                      │         │
│  │                                                       │         │
│  │  Query: (&Position, Option<&Color>)                  │         │
│  │                                                       │         │
│  │  Matched Entities: [1, 2, 4]  (all with Position)    │         │
│  │                                                       │         │
│  │  for (pos, color) in query.iter():                   │         │
│  │    let c = color.unwrap_or(&DEFAULT_COLOR)           │         │
│  │    render_at(pos, c)                                 │         │
│  └──────────────────────────────────────────────────────┘         │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘

Parallel Execution:
═══════════════════

System 1 (reads: Velocity, writes: Position)
     │
     ├─ Can run in parallel with ─┐
     │                             │
System 2 (reads: none, writes: Health)
                                   │
                                   ├─ CANNOT run in parallel
                                   │  (both write Position)
System 3 (reads: Position, writes: none)

Execution Order:
════════════════

Frame Start
    │
    ├─── System 1 (Movement)       ─┐
    │                               ├─ Run in parallel
    └─── System 2 (Health Regen)   ─┘
    │
    └─── System 3 (Render)  ◄─ Must wait for System 1
    │
Frame End

Component Archetype Example:
════════════════════════════

Entity 1: [Position, Velocity, Health]         ← Player archetype
Entity 2: [Position, Velocity]                 ← Moving object
Entity 3: [Health]                             ← Stationary damageable
Entity 4: [Position, Health]                   ← Stationary damageable with position
Entity 5: [Position, Velocity, Renderable]     ← Visual-only moving object

Queries match based on component combinations:
  Query<(&Position, &Velocity)>        → Entities [1, 2, 5]
  Query<&Health>                       → Entities [1, 3, 4]
  Query<(&Position, With<Velocity>)>   → Entities [1, 2, 5]
  Query<(&Position, Without<Health>)>  → Entities [2, 5]
```

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

- [Beginner's Guide: ECS Data Flow](../beginners-guide.md#ecs-data-flow) - Deep dive with diagrams
- [Getting Started](../getting-started/README.md) - First steps with Praxis
- [praxis_ecs crate](../../crates/praxis_ecs/README.md) - API documentation
- [Transform Hierarchy](transform-hierarchy.md) - Scene graph concepts
- [bevy_ecs documentation](https://docs.rs/bevy_ecs) - Underlying ECS library
