# Praxis Physics

Rapier3D-based physics integration for the Praxis game engine.

## Overview

ECS-integrated physics simulation with rigid bodies, colliders, and bidirectional transform synchronization.

**Key Features:**
- Dynamic, static, and kinematic rigid bodies
- Collision detection (box, sphere, capsule, mesh)
- Fixed timestep simulation (60 Hz default)
- Bidirectional ECS transform sync
- Forces, velocity, friction, restitution
- Collision events and queries

## Quick Start

```rust
use praxis_physics::{PhysicsWorld, PhysicsConfig, RigidBody, Collider};

// Setup
world.insert_resource(PhysicsWorld::new());
world.insert_resource(PhysicsConfig::default());

schedule.add_systems((
    sync_transforms_to_physics,
    step_physics_simulation,
    sync_transforms_from_physics,
).chain());

// Dynamic entity
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Dynamic,
    Collider::sphere(1.0),
));

// Static ground
world.spawn((
    Transform::default(),
    RigidBody::Static,
    Collider::cuboid(50.0, 0.5, 50.0),
));
```

## Components

- `RigidBody`: Dynamic, Static, or Kinematic
- `Collider`: Shape (box, sphere, capsule, etc.)
- `Velocity`: Linear and angular
- `ExternalForces`: Force/torque accumulator
- `Mass`, `Friction`, `Restitution`, `CollisionGroups`, `Sleeping`

## Documentation

**Comprehensive Guide:**
- [Physics Guide](../../docs/guides/physics.md) - Complete usage, patterns, best practices

**Concepts:**
- [Physics Concepts](../../docs/concepts/physics.md) - Theory and design

## Examples

```bash
cargo run --example comprehensive_scene_demo
```

## Dependencies

- `rapier3d` 0.22: Physics engine
- `bevy_ecs` 0.14: ECS integration
