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
use praxis_physics::{
    sync_transforms_to_physics,
    step_physics_simulation,
    sync_transforms_from_physics,
};
use praxis_ecs::{World, Schedule, Transform, GlobalTransform};
use praxis_math::Vec3;
use color_eyre::Result;

fn setup_physics(world: &mut World, schedule: &mut Schedule) -> Result<()> {
    // Initialize physics resources
    world.insert_resource(PhysicsWorld::new());
    world.insert_resource(PhysicsConfig::default());
    
    // Add physics systems to schedule in correct order
    schedule.add_systems((
        sync_transforms_to_physics,  // Step 1: ECS → Physics
        step_physics_simulation,     // Step 2: Physics update
        sync_transforms_from_physics, // Step 3: Physics → ECS
    ).chain());
    
    // Spawn a dynamic entity (affected by physics)
    world.spawn((
        Transform::from_xyz(0.0, 10.0, 0.0),
        GlobalTransform::default(),
        RigidBody::Dynamic,
        Collider::sphere(1.0),
    ));
    
    // Spawn static ground (never moves)
    world.spawn((
        Transform::default(),
        GlobalTransform::default(),
        RigidBody::Static,
        Collider::cuboid(50.0, 0.5, 50.0),  // 50x0.5x50 box
    ));
    
    Ok(())
}
```

## Components

- `RigidBody`: Dynamic, Static, or Kinematic
- `Collider`: Shape (box, sphere, capsule, etc.)
- `Velocity`: Linear and angular velocity
- `ExternalForces`: Force/torque accumulator
- `Mass`, `Friction`, `Restitution`, `CollisionGroups`, `Sleeping`

## Documentation

**Comprehensive Guide:**
- [Physics Guide](../../docs/guides/physics.md) - Complete usage, patterns, best practices

**Concepts:**
- [Physics Concepts](../../docs/concepts/physics.md) - Theory and design

## Examples

```bash
# Comprehensive scene with physics
cargo run --example comprehensive_scene_demo
```

## Dependencies

- `rapier3d` 0.22: Physics engine
- `bevy_ecs` 0.14: ECS integration
