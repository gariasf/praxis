# Physics System

Rigid body physics simulation in Praxis, powered by Rapier3D.

## Core Concepts

### Rigid Body Types

| Type | Description | Use Case |
|------|-------------|----------|
| **Dynamic** | Affected by forces and collisions | Balls, boxes, physics objects |
| **Static** | Never moves, infinite mass | Terrain, walls, level geometry |
| **Kinematic** | Moved by code, affects dynamics | Moving platforms, doors |

```rust
#[derive(Component)]
pub struct RigidBody {
    pub body_type: RigidBodyType,
}

pub enum RigidBodyType {
    Dynamic,
    Static,
    Kinematic,
}
```

### Colliders

Define collision geometry:

```rust
#[derive(Component)]
pub struct Collider {
    pub shape: ColliderShape,
    pub friction: f32,
    pub restitution: f32,  // Bounciness
}

pub enum ColliderShape {
    Ball { radius: f32 },
    Cuboid { half_extents: Vec3 },
    Capsule { half_height: f32, radius: f32 },
    Cylinder { half_height: f32, radius: f32 },
    ConvexHull { points: Vec<Vec3> },
    TriMesh { vertices: Vec<Vec3>, indices: Vec<[u32; 3]> },
}
```

### PhysicsWorld Resource

Central physics state managed by Rapier:

```rust
#[derive(Resource)]
pub struct PhysicsWorld {
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    integration_parameters: IntegrationParameters,
    physics_pipeline: PhysicsPipeline,
    // ...
}
```

## Fixed Timestep Integration

Physics runs at constant rate (60 Hz) independent of frame rate:

```
Frame Rate:  30 FPS  |  60 FPS  |  120 FPS
Physics:     2 steps |  1 step  |  0.5 steps (accumulated)
```

This ensures:
- Deterministic simulation
- Stable behavior at any frame rate
- Consistent collision detection

## Transform Synchronization

Bidirectional sync between ECS and Rapier:

```
Before Physics Step:
  ECS Transform → Rapier (for kinematic bodies)

After Physics Step:
  Rapier → ECS Transform (for dynamic bodies)
```

## Collision Events

Three event types for gameplay logic:

```rust
pub enum CollisionEvent {
    Started(Entity, Entity),   // First contact
    Stopped(Entity, Entity),   // Contact lost
    Persisted(Entity, Entity), // Ongoing contact
}
```

Handle with `CollisionEventReceiver` component:

```rust
world.spawn((
    RigidBody::dynamic(),
    Collider::ball(1.0),
    CollisionEventReceiver::default(),
));

// In system
fn collision_system(query: Query<&CollisionEventReceiver>) {
    for receiver in query.iter() {
        for event in receiver.events() {
            match event {
                CollisionEvent::Started(a, b) => { /* ... */ }
                _ => {}
            }
        }
    }
}
```

## Spatial Queries

Efficient queries for gameplay:

```rust
let physics = world.resource::<PhysicsWorld>();

// Raycast - find first hit
if let Some(hit) = physics.raycast(origin, direction, max_distance) {
    println!("Hit entity {:?} at distance {}", hit.entity, hit.distance);
}

// Raycast all - find all hits
for hit in physics.raycast_all(origin, direction, max_distance) {
    // Process each hit
}

// Shape cast - sweep a shape
let hit = physics.shape_cast(shape, origin, direction, max_distance);

// Point query - what's at this point?
let entities = physics.point_intersections(point);
```

## Usage Example

```rust
use praxis_physics::{PhysicsWorld, RigidBody, Collider, physics_step_system};

// Initialize
world.insert_resource(PhysicsWorld::new());

// Spawn physics entity
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    RigidBody::dynamic(),
    Collider::ball(0.5),
));

// Spawn static ground
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    RigidBody::fixed(),
    Collider::cuboid(Vec3::new(50.0, 0.5, 50.0)),
));

// Schedule systems
schedule.add_systems((
    sync_physics_transforms_system,
    physics_step_system,
    sync_physics_transforms_system,
).chain());
```

## Forces and Velocity

```rust
#[derive(Component)]
pub struct PhysicsVelocity {
    pub linear: Vec3,
    pub angular: Vec3,
}

#[derive(Component)]
pub struct ExternalForces {
    pub force: Vec3,
    pub torque: Vec3,
}

// Apply impulse
forces.force += Vec3::new(0.0, 1000.0, 0.0);
```

## See Also

- [BEGINNERS_GUIDE - Physics System](../BEGINNERS_GUIDE.md#physics-system) - Deep dive explanation
- [praxis_physics crate](../../crates/praxis_physics/README.md) - API documentation
- [physics_demo](../../examples/physics_demo.rs) - Working example
