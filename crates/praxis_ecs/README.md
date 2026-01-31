# praxis_ecs

Entity Component System (ECS) integration for the Praxis engine using bevy_ecs.

## Overview

This crate provides the ECS architecture that powers the Praxis engine. ECS is a design pattern that separates data (components) from behavior (systems), enabling flexible composition and high-performance parallel execution.

**New to ECS?** Start with the **[Getting Started Guide](./GETTING_STARTED.md)** for a practical introduction.

## Why ECS?

### Composition over Inheritance

Traditional OOP hierarchies become rigid and hard to maintain:

```rust
// OOP approach - rigid hierarchy
class GameObject { }
class Character extends GameObject { }
class Player extends Character { }
class Enemy extends Character { }
// What if we want a flying enemy? FlyingEnemy extends Enemy?
// What about a flying player? Multiple inheritance? Traits everywhere?
```

ECS enables flexible composition:

```rust
// ECS approach - flexible composition
world.spawn((Transform, Renderable, Health, Player));
world.spawn((Transform, Renderable, Health, Enemy, Flying));
world.spawn((Transform, Renderable, Flying, Collectible));
// Any combination works!
```

### Performance Through Data-Oriented Design

- **Cache-friendly**: Components stored contiguously in memory
- **Parallel execution**: Systems can run concurrently when accessing different components
- **Minimal indirection**: Direct access to component data

## Core Concepts

### Entities

Unique identifiers for game objects. Think of them as indices into component arrays.

```rust
use praxis_ecs::World;

let mut world = World::new();
let entity = world.spawn(());  // Spawn empty entity
```

### Components

Pure data structures representing aspects of entities:

```rust
use praxis_ecs::Component;

#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

// Spawn entity with components
world.spawn((
    Health { current: 100.0, max: 100.0 },
    Velocity { x: 0.0, y: 0.0, z: 0.0 },
));
```

### Systems

Functions that operate on entities with specific components:

```rust
use praxis_ecs::{Query, Transform};

fn movement_system(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
        transform.translation.z += velocity.z;
    }
}
```

### Resources

Global singleton data accessible to systems:

```rust
use praxis_ecs::Resource;

#[derive(Resource)]
struct GameSettings {
    difficulty: u32,
    sound_volume: f32,
}

world.insert_resource(GameSettings {
    difficulty: 5,
    sound_volume: 0.8,
});
```

## Common Patterns

### Marker Components

Zero-sized types for tagging entities:

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

// Query only player entities
fn player_system(query: Query<&Transform, With<Player>>) {
    for transform in query.iter() {
        // Only processes players
    }
}
```

### Component Bundles

Groups of commonly used components:

```rust
use praxis_ecs::Bundle;

#[derive(Bundle)]
struct CharacterBundle {
    transform: Transform,
    health: Health,
    velocity: Velocity,
}

world.spawn(CharacterBundle {
    transform: Transform::default(),
    health: Health { current: 100.0, max: 100.0 },
    velocity: Velocity { x: 0.0, y: 0.0, z: 0.0 },
});
```

### Hierarchical Relationships

Parent-child relationships using built-in components:

```rust
use praxis_ecs::{Parent, Children, Transform, GlobalTransform};

let parent = world.spawn((
    Transform::from_xyz(10.0, 0.0, 0.0),
    GlobalTransform::default(),
));

let child = world.spawn((
    Transform::from_xyz(5.0, 0.0, 0.0),  // Relative to parent
    GlobalTransform::default(),
    Parent(parent),
));

// Transform propagation system automatically updates child's GlobalTransform
```

### Query Filters

Refine queries with filters:

```rust
use praxis_ecs::{Query, With, Without, Changed, Added};

// Only entities WITH Player component
fn player_input(query: Query<&Transform, With<Player>>) { }

// Only entities WITHOUT Dead component
fn active_entities(query: Query<&Transform, Without<Dead>>) { }

// Only entities where Transform changed this frame
fn dirty_transforms(query: Query<&Transform, Changed<Transform>>) { }

// Only entities where Transform was just added
fn new_transforms(query: Query<&Transform, Added<Transform>>) { }
```

## Built-in Components

### Transform Components

- **`Transform`**: Local position, rotation, scale
- **`GlobalTransform`**: World-space transformation (computed automatically)
- **`Parent`**: Parent entity in hierarchy
- **`Children`**: Child entities in hierarchy

```rust
use praxis_ecs::{Transform, GlobalTransform, Parent};
use praxis_math::{Vec3, Quat};

world.spawn((
    Transform {
        translation: Vec3::new(1.0, 2.0, 3.0),
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    },
    GlobalTransform::default(),
));
```

### Rendering Components

- **`Camera`**: Camera marker with priority
- **`PerspectiveProjection`**: Perspective camera projection
- **`OrthographicProjection`**: Orthographic camera projection
- **`CameraMatrices`**: Computed view/projection matrices
- **`MeshHandle`**: Reference to shared mesh
- **`MaterialHandle`**: Reference to shared material
- **`Visibility`**: Show/hide entities

### Lighting Components

- **`DirectionalLight`**: Sun-like parallel lights
- **`PointLight`**: Omnidirectional light sources
- **`LightingData`**: Resource collecting all lights

### Utility Components

- **`Name`**: Debug name for entities
- **`Active`**: Marker for active entities
- **`NoSave`**: Exclude from serialization
- **`BoundingBox`**: Spatial bounds

## System Scheduling

Systems are organized into schedules that run in stages:

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};

let mut schedule = Schedule::default();

// Systems run in order when chained
schedule.add_systems((
    input_system,
    physics_system,
    animation_system,
).chain());

// Systems without conflicts run in parallel
schedule.add_systems((
    render_meshes,  // Reads Transform
    update_audio,   // Reads Transform
    // Both can run in parallel since they only read
));
```

### System Ordering

```rust
// Explicit ordering
schedule.add_systems(
    physics_system
        .before(animation_system)
        .after(input_system)
);

// System sets for logical grouping
use praxis_ecs::systems::CoreSystemSet;

schedule.add_systems(
    my_system.in_set(CoreSystemSet::Update)
);
```

## Serialization

Save and load world state:

```rust
use praxis_ecs::{World, ComponentRegistry, Transform, Name};

// Create world
let mut world = World::new();
world.spawn((
    Name::new("Player"),
    Transform::from_xyz(1.0, 2.0, 3.0),
));

// Register serializable components
let mut registry = ComponentRegistry::new();
registry.register::<Name>();
registry.register::<Transform>();

// Serialize to RON
let ron_string = world.serialize(&registry).unwrap();
std::fs::write("save.ron", ron_string).unwrap();

// Deserialize
let mut new_world = World::new();
let data = std::fs::read_to_string("save.ron").unwrap();
new_world.deserialize(&data, &registry).unwrap();
```

## Performance Best Practices

### 1. Use Queries Efficiently

```rust
// Good: Narrow queries with filters
fn render_visible(query: Query<(&Transform, &Renderable), With<Visible>>) {
    // Only processes visible entities
}

// Bad: Wide query with manual filtering
fn render_all(query: Query<(&Transform, &Renderable, Option<&Visible>)>) {
    for (transform, renderable, visible) in query.iter() {
        if visible.is_some() {
            // Manual filtering is slower
        }
    }
}
```

### 2. Batch Spawning

```rust
// Good: Batch spawning
let entities = world.spawn_batch(vec![
    (Transform::default(), Enemy),
    (Transform::default(), Enemy),
    (Transform::default(), Enemy),
]);

// Bad: Individual spawns
for _ in 0..3 {
    world.spawn((Transform::default(), Enemy));
}
```

### 3. Use Change Detection

```rust
// Good: Only process changed entities
fn update_dirty(query: Query<&Transform, Changed<Transform>>) {
    // Only processes entities where Transform changed
}

// Bad: Process everything
fn update_all(query: Query<&Transform>) {
    // Processes all entities every frame
}
```

### 4. Avoid Exclusive World Access

```rust
// Good: Specific queries enable parallelism
fn my_system(
    query: Query<&Transform>,
    resource: Res<GameState>,
) { }

// Bad: Exclusive access blocks parallelism
fn my_system(world: &mut World) {
    // No other system can run in parallel
}
```

## Common Pitfalls

### 1. Forgetting to Register Components for Serialization

```rust
// Component won't be saved!
world.spawn(MyComponent);

// Must register first:
registry.register::<MyComponent>();
```

### 2. Circular Parent Relationships

```rust
// Don't create cycles!
let e1 = world.spawn(Parent(e2));
let e2 = world.spawn(Parent(e1));  // Cycle!
```

### 3. Mutating During Iteration

```rust
// Bad: Can't spawn during iteration
fn spawn_enemy(mut commands: Commands, query: Query<&Transform>) {
    for transform in query.iter() {
        // Use Commands for deferred operations
        commands.spawn(Enemy);
    }
}
```

## Examples

See the `examples/` directory for complete examples:

- `ecs_integration`: Basic ECS usage
- `transform_propagation_demo`: Hierarchical transforms
- `scene_demo`: Scene management

## API Documentation

Full API documentation is available in the source code. Key modules:

- `components`: Built-in component types
- `systems`: Pre-built systems
- `serialization`: World save/load
- `world`: World container and API

## Dependencies

- **bevy_ecs**: High-performance ECS framework
- **serde**: Serialization support
- **ron**: Rusty Object Notation format

## Related Documentation

### Crate Documentation

- **[Getting Started Guide](./GETTING_STARTED.md)** - Practical introduction for newcomers
- **[ECS Best Practices](./ECS_BEST_PRACTICES.md)** - Comprehensive guide to writing efficient ECS code
- **[Query Patterns](./QUERY_PATTERNS.md)** - Complete guide to querying entities
- **[System Ordering](./SYSTEM_ORDERING.md)** - Understanding and controlling system execution order
- **[Transform Propagation](./transform-propagation.md)** - Hierarchical transform system details
- **[Serialization Guide](./serialization.md)** - Saving and loading world state

### Engine Documentation

- [ECS Architecture](../../docs/concepts/ecs-architecture.md) - High-level ECS concepts
- [ECS Patterns](../../docs/architecture/ecs-patterns.md) - Common architectural patterns
- [System Execution Order](../../docs/architecture/ecs-system-execution-order.md) - Engine-level system scheduling
