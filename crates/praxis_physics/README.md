# praxis_physics

Physics simulation for Praxis engine using Rapier3D.

## Overview

Integrates Rapier3D physics engine with ECS, providing rigid body dynamics and collision detection.

## Features

- **Rigid Bodies**: Dynamic, Static, Kinematic
- **Colliders**: Box, Sphere, Capsule, Convex Hull, Triangle Mesh
- **Constraints**: Joints, springs, motors
- **Collision Detection**: Broad phase, narrow phase, events
- **Physics Materials**: Friction, restitution
- **Fixed Timestep**: Deterministic simulation (60 Hz default)
- **ECS Integration**: Bidirectional transform sync

## Example

```rust
use praxis_physics::{RigidBody, Collider};

// Dynamic rigid body with box collider
commands.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    RigidBody::Dynamic,
    Collider::box(1.0, 1.0, 1.0),
    PhysicsVelocity::default(),
));

// Static ground plane
commands.spawn((
    Transform::default(),
    RigidBody::Static,
    Collider::box(100.0, 1.0, 100.0),
));
```

## Architecture

```
PhysicsWorld (Resource)
    ├── Rapier PhysicsWorld
    ├── Entity -> PhysicsHandle map
    └── PhysicsHandle -> Entity map

Systems:
1. Sync ECS transforms → Physics
2. Step physics simulation
3. Sync Physics → ECS transforms
4. Emit collision events
```

## Dependencies

- `rapier3d`: Physics engine
- `serde`: Serialization support

## Usage

```toml
praxis_physics = { path = "../praxis_physics", version = "0.1.0" }
```
