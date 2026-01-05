# Praxis Physics

Physics integration for the Praxis game engine using Rapier3D.

## Overview

This crate provides physics simulation capabilities for Praxis, wrapping the Rapier3D physics engine in an ECS-friendly interface. It handles rigid body dynamics, collision detection, and physics-based interactions.

## Architecture

The physics system integrates with Praxis's ECS architecture by:

1. **Components**: Physics properties (RigidBody, Collider, etc.) are stored as ECS components
2. **Resources**: The physics pipeline and configuration are stored as ECS resources
3. **Systems**: Physics simulation runs as an ECS system each frame

### Key Components

- **`RigidBody`**: Marks an entity as a physics-simulated rigid body (Dynamic, Static, or Kinematic)
- **`Collider`**: Defines collision geometry (box, sphere, capsule, mesh, etc.)
- **`Velocity`**: Linear and angular velocity for dynamic bodies
- **`ExternalForces`**: Accumulator for forces and torques applied to bodies
- **`Mass`**: Mass properties (mass, center of mass, inertia tensor)
- **`Friction`**: Surface friction coefficient
- **`Restitution`**: Bounciness coefficient (0.0 = no bounce, 1.0 = perfect bounce)
- **`CollisionGroups`**: Filtering for what collides with what
- **`Sleeping`**: Controls whether bodies can sleep when at rest

### Resources

- **`PhysicsWorld`**: Wrapper around Rapier's physics pipeline, managing simulation state
- **`PhysicsConfig`**: Global physics configuration (gravity, timestep, solver iterations)

### Systems

- **`sync_transforms_to_physics`**: Copies Transform data from entities to Rapier bodies
- **`step_physics_simulation`**: Advances the physics simulation by one timestep
- **`sync_transforms_from_physics`**: Updates entity Transforms based on Rapier body states
- **`apply_external_forces`**: Applies accumulated forces to physics bodies

## Rapier Wrapper Design Decisions

### ECS-First Design

Rather than exposing Rapier's handles directly, we use ECS components to store physics properties. This provides:

- **Consistency**: Physics data lives alongside other entity data
- **Familiarity**: Users work with components they already understand
- **Flexibility**: Easy to query, inspect, and modify physics properties
- **Decoupling**: Game logic doesn't need to know about Rapier internals

### Transform Synchronization

The physics system maintains a bidirectional sync between ECS `Transform` components and Rapier rigid body positions:

1. **Before physics step**: Entity transforms are copied to Rapier bodies (for kinematic movement)
2. **After physics step**: Rapier body positions are copied back to entity transforms (for dynamic bodies)

This allows:
- Non-physics systems to move kinematic bodies via Transform
- Physics-controlled bodies to automatically update Transform for rendering

### Component-Based Collision Shapes

Collider shapes are defined through components rather than requiring manual shape construction:

```rust
// Simple box collider
world.spawn((
    Transform::from_xyz(0.0, 5.0, 0.0),
    RigidBody::Dynamic,
    Collider::cuboid(1.0, 1.0, 1.0),
));
```

This provides a cleaner API while still supporting complex scenarios through builder methods.

### Lazy Initialization

Physics bodies and colliders are created lazily when components are added. This means:

- No setup boilerplate required
- Components can be added/removed dynamically
- Physics state is automatically managed

### Performance Considerations

The wrapper is designed with performance in mind:

- **Minimal allocations**: Rapier handles are cached in internal maps
- **Batch operations**: Transform syncing happens in bulk each frame
- **Optional features**: Systems can be disabled if not needed
- **Fixed timestep**: Physics runs at a fixed rate independent of frame rate

### Query-Friendly Design

All physics data is accessible through standard ECS queries:

```rust
// Find all dynamic bodies with high velocity
fn fast_objects(query: Query<(&Velocity, &RigidBody)>) {
    for (velocity, body) in query.iter() {
        if body.is_dynamic() && velocity.linear.length() > 10.0 {
            // Handle fast-moving object
        }
    }
}
```

This integrates seamlessly with Praxis's existing system patterns.

### Contact/Event Handling

Physics events (collisions, contacts, etc.) are exposed through:

1. **ContactEvents Resource**: Queue of collision enter/exit events
2. **Query-based**: Check collision state through components
3. **Callbacks**: Optional system integration for immediate response

This provides flexibility for different use cases while maintaining the ECS paradigm.

## Usage Example

```rust
use praxis_physics::{
    PhysicsWorld, PhysicsConfig,
    RigidBody, Collider, Velocity, Restitution,
    step_physics_simulation, sync_transforms_from_physics, sync_transforms_to_physics,
};
use praxis_ecs::{World, Schedule, IntoSystemConfigs, Transform};
use praxis_math::Vec3;

// Setup
let mut world = World::new();
world.insert_resource(PhysicsWorld::new());
world.insert_resource(PhysicsConfig::default());

let mut schedule = Schedule::default();
schedule.add_systems((
    sync_transforms_to_physics,
    step_physics_simulation,
    sync_transforms_from_physics,
).chain());

// Create a ground plane
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    RigidBody::Static,
    Collider::cuboid(50.0, 0.5, 50.0),
));

// Create a dynamic bouncing ball
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    Velocity::default(),
    Restitution::new(0.8), // Bouncy!
));

// Run simulation
world.inner_mut().run_schedule(&mut schedule);
```

## Integration with Praxis Scene Graph

The physics system respects the scene graph hierarchy:

- Physics bodies use `GlobalTransform` for world-space positioning
- Local `Transform` can still be used for parenting relationships
- Kinematic bodies can be moved via Transform changes

## Future Extensions

Planned features include:

- Character controller component
- Joint/constraint components
- Raycast/shapecast utilities
- Debug visualization
- Physics material system
- Trigger volumes
- Continuous collision detection options

## Examples

See the physics system in action:

```bash
cargo run --example comprehensive_scene_demo
```

## Dependencies

- `rapier3d` 0.22: Physics engine
- `bevy_ecs` 0.14: ECS integration
- `praxis_ecs`: Transform components and systems
- `praxis_math`: Math types (Vec3, Quat)
- `praxis_utils`: Error handling

## References

- [Rapier3D Documentation](https://rapier.rs/)
- [Rapier3D User Guide](https://rapier.rs/docs/user_guides/rust/getting_started)

## See Also

- [Physics Guide](../../docs/guides/physics.md)
- [ECS System](../praxis_ecs/README.md)
