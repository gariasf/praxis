# Praxis ECS

Entity Component System for the Praxis game engine, built on bevy_ecs.

## Overview

ECS with transform hierarchy, camera system, lighting, and comprehensive serialization.

**Key Features:**
- Transform hierarchy with automatic propagation
- Perspective and orthographic cameras
- Directional and point lights with shadows
- Component serialization to RON format
- Parent-child relationships with entity references
- Common components (Name, Transform, Visibility, etc.)

## Quick Start

```rust
use praxis_ecs::{World, Transform, Name, ComponentRegistry};

let mut world = World::new();

// Spawn entities
world.spawn((
    Name::new("Player"),
    Transform::from_xyz(10.0, 0.0, 5.0),
));

// Serialization
let mut registry = ComponentRegistry::new();
registry.register_common_types();

let ron_string = world.serialize(&registry)?;
let mut new_world = World::new();
new_world.deserialize(&ron_string, &registry)?;
```

## Transform Hierarchy

```rust
use praxis_ecs::{Transform, GlobalTransform, Parent, Children};

// Parent-child relationships automatically propagate transforms
let parent = world.spawn((
    Transform::default(),
    GlobalTransform::default(),
));

let child = world.spawn((
    Transform::from_xyz(1.0, 0.0, 0.0),
    GlobalTransform::default(),
    Parent(parent),
));

// Run transform propagation system
use praxis_ecs::systems::propagate_transforms;
schedule.add_systems(propagate_transforms);
```

## Camera System

```rust
use praxis_ecs::{PerspectiveCameraBundle, Camera};

world.spawn(PerspectiveCameraBundle::new(
    Vec3::new(0.0, 2.0, 5.0),
    60.0_f32.to_radians(),
    16.0 / 9.0,
));
```

## Documentation

**Reference:**
- [Components Reference](../../docs/reference/components.md)
- [Camera API](../../docs/reference/camera-api.md)

**Concepts:**
- [ECS Architecture](../../docs/concepts/ecs-architecture.md)
- [Transform Hierarchy](../../docs/concepts/transform-hierarchy.md)

**Crate Documentation:**
- [Transform Propagation](TRANSFORM_PROPAGATION.md)

## Dependencies

- `bevy_ecs` 0.14: Core ECS
- `ron` 0.8: Serialization
- `serde` 1.0: Serialization framework
