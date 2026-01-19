# Physics API Reference

API reference for the Praxis physics system powered by Rapier3D.

## Resources

### PhysicsWorld

Central resource managing the physics simulation.

```rust
#[derive(Resource)]
pub struct PhysicsWorld { /* ... */ }
```

**Methods:**
- `new()` - Create new physics world
- `step(delta: f32)` - Advance simulation by timestep

### PhysicsConfig

Configuration for physics simulation.

```rust
#[derive(Resource)]
pub struct PhysicsConfig {
    pub timestep: f32,          // Fixed timestep (default: 1/60)
    pub gravity: Vec3,          // Gravity vector
    pub max_velocity: f32,      // Maximum linear velocity
    pub max_angular_velocity: f32,
}
```

**Methods:**
- `default()` - Standard configuration (60Hz, -9.81 gravity)
- `with_gravity(gravity: Vec3)` - Set gravity
- `with_timestep(timestep: f32)` - Set fixed timestep

## Components

### RigidBody

Defines physics body type.

```rust
#[derive(Component)]
pub enum RigidBody {
    Dynamic,    // Affected by forces and gravity
    Static,     // Never moves, infinite mass
    Kinematic,  // Controlled by user, not by forces
}
```

### Collider

Collision shape attached to a body.

```rust
#[derive(Component)]
pub enum Collider {
    Cuboid { half_extents: Vec3 },
    Sphere { radius: f32 },
    Capsule { half_height: f32, radius: f32 },
    // Additional variants...
}
```

**Constructors:**
- `cuboid(hx, hy, hz)` - Box collider
- `sphere(radius)` - Sphere collider
- `capsule_y(half_height, radius)` - Capsule along Y-axis

### Velocity

Linear and angular velocity components.

```rust
#[derive(Component)]
pub struct Velocity {
    pub linear: Vec3,   // Units per second
    pub angular: Vec3,  // Radians per second
}
```

### Mass

Mass properties.

```rust
#[derive(Component)]
pub struct Mass {
    pub mass: f32,
    pub angular_inertia: f32,
}
```

### Friction

Surface friction coefficient.

```rust
#[derive(Component)]
pub struct Friction(pub f32);  // 0.0 = frictionless, 1.0 = high friction
```

### Restitution

Bounciness/elasticity.

```rust
#[derive(Component)]
pub struct Restitution(pub f32);  // 0.0 = no bounce, 1.0 = perfect bounce
```

### ExternalForces

Accumulator for forces and torques.

```rust
#[derive(Component)]
pub struct ExternalForces {
    pub force: Vec3,
    pub torque: Vec3,
}
```

**Methods:**
- `apply_force(force: Vec3)` - Add force
- `apply_torque(torque: Vec3)` - Add torque
- `apply_impulse(impulse: Vec3)` - Instantaneous velocity change
- `clear()` - Reset forces/torques

### CollisionGroups

Collision filtering.

```rust
#[derive(Component)]
pub struct CollisionGroups {
    pub memberships: u32,  // Bitmask of groups this collider belongs to
    pub filters: u32,      // Bitmask of groups this collider interacts with
}
```

### Sleeping

Sleep state for performance optimization.

```rust
#[derive(Component)]
pub struct Sleeping {
    pub sleeping: bool,
}
```

## Systems

### sync_transforms_to_physics

Synchronizes ECS Transform components to physics bodies.

**Schedule:** Run before physics step.

```rust
fn sync_transforms_to_physics(
    query: Query<(&Transform, &RigidBody)>,
    physics_world: ResMut<PhysicsWorld>,
)
```

### step_physics_simulation

Advances the physics simulation by one timestep.

**Schedule:** Run after transform sync, before sync back.

```rust
fn step_physics_simulation(
    config: Res<PhysicsConfig>,
    physics_world: ResMut<PhysicsWorld>,
)
```

### sync_transforms_from_physics

Synchronizes physics body positions back to ECS Transform components.

**Schedule:** Run after physics step.

```rust
fn sync_transforms_from_physics(
    query: Query<&mut Transform, With<RigidBody>>,
    physics_world: Res<PhysicsWorld>,
)
```

### collect_collision_events

Collects and processes collision events.

**Schedule:** Run after physics step.

```rust
fn collect_collision_events(
    physics_world: Res<PhysicsWorld>,
    events: EventWriter<CollisionEvent>,
)
```

## Events

### CollisionEvent

Event fired when two bodies collide or separate.

```rust
pub enum CollisionEvent {
    Started(Entity, Entity),   // Collision began
    Stopped(Entity, Entity),   // Collision ended
}
```

## Queries

### Raycasting

```rust
// Via PhysicsWorld
let hit = physics_world.cast_ray(
    origin,
    direction,
    max_distance,
    solid,
    filter,
);

if let Some((entity, toi)) = hit {
    // Process hit
}
```

### Shape Casting

```rust
let hits = physics_world.cast_shape(
    shape,
    position,
    direction,
    max_distance,
    filter,
);
```

### Intersection Tests

```rust
let entities = physics_world.intersections_with_aabb(aabb);
let entities = physics_world.intersections_with_sphere(center, radius);
```

## Common Patterns

### Dynamic Object

```rust
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    Velocity::default(),
    Mass::new(1.0),
    Friction(0.5),
    Restitution(0.3),
));
```

### Static Ground

```rust
world.spawn((
    Transform::default(),
    GlobalTransform::default(),
    RigidBody::Static,
    Collider::cuboid(50.0, 0.5, 50.0),
    Friction(0.8),
));
```

### Kinematic Platform

```rust
world.spawn((
    Transform::default(),
    GlobalTransform::default(),
    RigidBody::Kinematic,
    Collider::cuboid(2.0, 0.2, 2.0),
    Velocity::default(),
));
```

### Applying Forces

```rust
fn movement_system(
    input: Res<InputState>,
    mut query: Query<&mut ExternalForces, With<Player>>,
) {
    for mut forces in &mut query {
        if input.is_key_pressed(KeyCode::KeyW) {
            forces.apply_force(Vec3::new(0.0, 0.0, -10.0));
        }
    }
}
```

## See Also

- [Physics Guide](../guides/physics.md) - Comprehensive usage guide
- [Physics Concepts](../concepts/physics.md) - Theory and design
- [Physics Learning Path](../learning-paths/physics.md) - Structured learning progression
- [praxis_physics Crate](../../crates/praxis_physics/README.md) - Crate documentation
