# praxis_ecs

ECS integration for Praxis engine using bevy_ecs.

## Overview

Wraps `bevy_ecs` to provide the Entity Component System architecture for Praxis.

## Concepts

### Entities

Unique identifiers for game objects:

```rust
let entity = world.spawn().id();
```

### Components

Data attached to entities:

```rust
#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Velocity(Vec3);
```

### Systems

Functions that process components:

```rust
fn movement_system(mut query: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.position += velocity.0 * time.delta_seconds();
    }
}
```

### Resources

Global state:

```rust
#[derive(Resource)]
struct GameConfig {
    difficulty: u32,
}
```

## Features

- Data-oriented design
- Parallel system execution
- Type-safe queries
- Composition over inheritance

## Example

```rust
use praxis_ecs::prelude::*;

#[derive(Component)]
struct Player;

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Player,
        Transform::default(),
        Health(100.0),
    ));
}
```

## Dependencies

- `bevy_ecs`: Entity Component System
- `serde`: Serialization support

## Usage

```toml
praxis_ecs = { path = "../praxis_ecs", version = "0.1.0" }
```
