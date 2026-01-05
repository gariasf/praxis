# Physics Guide

Practical guide to using the Rapier3D-powered physics system in Praxis for rigid body dynamics, collisions, and physics-based gameplay.

## Quick Start

### Initialize Physics

```rust
use praxis_physics::{PhysicsWorld, PhysicsConfig};
use praxis_ecs::{World, Schedule};

let mut world = World::new();

// Insert physics resources
world.insert_resource(PhysicsWorld::new());
world.insert_resource(PhysicsConfig::default());

// Add physics systems to schedule
use praxis_physics::{
    sync_transforms_to_physics,
    step_physics_simulation,
    sync_transforms_from_physics,
};

schedule.add_systems((
    sync_transforms_to_physics,
    step_physics_simulation,
    sync_transforms_from_physics,
).chain());
```

### Create Physics Entities

```rust
use praxis_physics::{RigidBody, Collider};
use praxis_ecs::{Transform, GlobalTransform};
use praxis_math::Vec3;

// Dynamic ball (affected by physics)
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Dynamic,
    Collider::sphere(1.0),
));

// Static ground (never moves)
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Static,
    Collider::cuboid(50.0, 0.5, 50.0),
));
```

## Rigid Body Types

### Dynamic Bodies

Affected by forces, gravity, and collisions:

```rust
use praxis_ecs::{Transform, GlobalTransform};

world.spawn((
    Transform::from_xyz(0.0, 5.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    Velocity::default(),
    Mass::new(1.0),
));
```

### Static Bodies

Never move, have infinite mass:

```rust
use praxis_ecs::{Transform, GlobalTransform};

// Walls, floors, terrain
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Static,
    Collider::cuboid(10.0, 10.0, 0.5),
));
```

### Kinematic Bodies

Moved by code, affect dynamic bodies but aren't affected themselves:

```rust
use praxis_ecs::{Transform, GlobalTransform, Query, With, Component};

#[derive(Component)]
struct MovingPlatform;

// Moving platforms, doors
world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Kinematic,
    Collider::cuboid(5.0, 0.5, 5.0),
    MovingPlatform,
));

fn move_platform(mut query: Query<&mut Transform, With<MovingPlatform>>) {
    for mut transform in query.iter_mut() {
        transform.translation.y += 0.1;  // Physics will handle collisions
    }
}
```

## Collider Shapes

### Primitive Shapes

```rust
// Sphere
Collider::sphere(1.0);

// Box
Collider::cuboid(1.0, 2.0, 1.0);  // Half extents

// Capsule (pill shape)
Collider::capsule(1.0, 0.5);  // Half-height, radius

// Cylinder
Collider::cylinder(2.0, 1.0);  // Half-height, radius
```

### Complex Shapes

```rust
// Convex hull from points
let points = vec![
    Vec3::new(-1.0, 0.0, 0.0),
    Vec3::new(1.0, 0.0, 0.0),
    Vec3::new(0.0, 2.0, 0.0),
];
Collider::convex_hull(points);

// Triangle mesh (for static geometry)
let vertices = vec![/* ... */];
let indices = vec![/* ... */];
Collider::trimesh(vertices, indices);
```

## Material Properties

### Friction

Controls how "grippy" surfaces are:

```rust
world.spawn((
    RigidBody::Dynamic,
    Collider::cuboid(1.0, 1.0, 1.0),
    Friction::new(0.8),  // 0.0 = ice, 1.0 = high friction
));
```

### Restitution (Bounciness)

Controls how bouncy collisions are:

```rust
world.spawn((
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    Restitution::new(0.9),  // 0.0 = no bounce, 1.0 = perfect bounce
));
```

### Mass

```rust
// Explicit mass
world.spawn((
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    Mass::new(10.0),
));

// Default: mass computed from collider volume and density
```

## Forces and Velocity

### Direct Velocity Control

```rust
fn jump_on_input(
    input: Res<InputState>,
    mut query: Query<&mut Velocity>,
) {
    if input.keyboard.just_pressed(KeyCode::Space) {
        for mut vel in query.iter_mut() {
            vel.linear.y = 10.0;  // Jump velocity
        }
    }
}
```

### Applying Forces

```rust
use praxis_physics::ExternalForces;

fn apply_thrust(
    input: Res<InputState>,
    mut query: Query<&mut ExternalForces>,
) {
    for mut forces in query.iter_mut() {
        if input.keyboard.pressed(KeyCode::KeyW) {
            forces.force += Vec3::new(0.0, 0.0, -100.0);
        }
    }
}
```

### Impulses

One-time velocity changes:

```rust
fn explosion(
    mut query: Query<(&Transform, &mut Velocity)>,
    explosion_pos: Vec3,
) {
    for (transform, mut vel) in query.iter_mut() {
        let dir = (transform.translation - explosion_pos).normalize();
        let distance = transform.translation.distance(explosion_pos);
        let strength = (10.0 - distance).max(0.0) * 5.0;
        
        vel.linear += dir * strength;
    }
}
```

## Collision Detection

### Collision Events

```rust
use praxis_physics::{CollisionEvent, CollisionEventReceiver};

// Add receiver to entity
world.spawn((
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    CollisionEventReceiver::default(),
));

// Handle collision events
fn handle_collisions(
    query: Query<(Entity, &CollisionEventReceiver, Option<&Name>)>,
) {
    for (entity, receiver, name) in query.iter() {
        for event in receiver.events() {
            match event {
                CollisionEvent::Started(entity_a, entity_b) => {
                    tracing::info!("Collision started: {:?} <-> {:?}", entity_a, entity_b);
                }
                CollisionEvent::Stopped(entity_a, entity_b) => {
                    tracing::info!("Collision ended: {:?} <-> {:?}", entity_a, entity_b);
                }
                CollisionEvent::Persisted(_, _) => {
                    // Ongoing collision
                }
            }
        }
    }
}
```

### Collision Groups

Filter what collides with what:

```rust
use praxis_physics::CollisionGroups;

// Player bullets don't collide with player
world.spawn((
    RigidBody::Dynamic,
    Collider::sphere(0.1),
    CollisionGroups::new()
        .with_memberships(0b0010)  // Bullet group
        .with_filter(0b1101),      // Collides with everything except player (0b0010)
));
```

## Spatial Queries

### Raycasting

```rust
fn shoot_raycast(
    physics_world: Res<PhysicsWorld>,
    origin: Vec3,
    direction: Vec3,
) {
    let max_distance = 100.0;
    
    if let Some(hit) = physics_world.raycast(origin, direction, max_distance) {
        tracing::info!("Hit entity {:?} at distance {}", hit.entity, hit.distance);
        tracing::info!("Hit point: {:?}", hit.point);
        tracing::info!("Hit normal: {:?}", hit.normal);
    }
}
```

### Raycast All

Find all hits along a ray:

```rust
fn penetrating_ray(
    physics_world: Res<PhysicsWorld>,
    origin: Vec3,
    direction: Vec3,
) {
    let hits = physics_world.raycast_all(origin, direction, 100.0);
    
    for hit in hits {
        tracing::info!("Pierced entity {:?} at {}", hit.entity, hit.distance);
    }
}
```

### Shape Casting

Sweep a shape through space:

```rust
// Check if player can fit in new position
fn can_move_to(
    physics_world: Res<PhysicsWorld>,
    player_collider: &Collider,
    from: Vec3,
    to: Vec3,
) -> bool {
    let direction = (to - from).normalize();
    let distance = from.distance(to);
    
    match physics_world.shape_cast(player_collider, from, direction, distance) {
        Some(_hit) => false,  // Something in the way
        None => true,         // Path is clear
    }
}
```

### Point Queries

Find what's at a point:

```rust
fn check_point(physics_world: Res<PhysicsWorld>, point: Vec3) {
    let entities = physics_world.point_intersections(point);
    
    for entity in entities {
        tracing::info!("Point intersects entity {:?}", entity);
    }
}
```

## Configuration

### Global Physics Settings

```rust
use praxis_physics::PhysicsConfig;

let config = PhysicsConfig {
    gravity: Vec3::new(0.0, -9.81, 0.0),  // Earth gravity
    timestep: 1.0 / 60.0,                  // 60 Hz fixed timestep
    solver_iterations: 4,                   // Constraint solver precision
    ..Default::default()
};

world.insert_resource(config);
```

### Per-Body Settings

```rust
use praxis_physics::Sleeping;

world.spawn((
    RigidBody::Dynamic,
    Collider::sphere(1.0),
    Sleeping::enabled(),  // Allow body to sleep when at rest
));
```

## Common Patterns

### Character Controller

Simple kinematic character:

```rust
#[derive(Component)]
struct CharacterController {
    speed: f32,
    jump_force: f32,
}

fn character_movement(
    input: Res<InputState>,
    mut query: Query<(&CharacterController, &mut Transform, &mut Velocity)>,
) {
    for (controller, mut transform, mut velocity) in query.iter_mut() {
        // Horizontal movement
        let mut move_dir = Vec3::ZERO;
        if input.keyboard.pressed(KeyCode::KeyW) { move_dir.z -= 1.0; }
        if input.keyboard.pressed(KeyCode::KeyS) { move_dir.z += 1.0; }
        if input.keyboard.pressed(KeyCode::KeyA) { move_dir.x -= 1.0; }
        if input.keyboard.pressed(KeyCode::KeyD) { move_dir.x += 1.0; }
        
        velocity.linear.x = move_dir.x * controller.speed;
        velocity.linear.z = move_dir.z * controller.speed;
        
        // Jump
        if input.keyboard.just_pressed(KeyCode::Space) {
            velocity.linear.y = controller.jump_force;
        }
    }
}
```

### Moving Platform

```rust
#[derive(Component)]
struct MovingPlatform {
    start_pos: Vec3,
    end_pos: Vec3,
    speed: f32,
    time: f32,
}

fn update_platform(
    time: Res<Time>,
    mut query: Query<(&mut MovingPlatform, &mut Transform), With<RigidBody>>,
) {
    for (mut platform, mut transform) in query.iter_mut() {
        platform.time += time.delta_seconds() * platform.speed;
        let t = (platform.time.sin() + 1.0) / 2.0;  // Oscillate 0-1
        
        transform.translation = platform.start_pos.lerp(platform.end_pos, t);
    }
}
```

### Physics-Based Projectile

```rust
fn spawn_projectile(
    world: &mut World,
    position: Vec3,
    direction: Vec3,
    speed: f32,
) {
    world.spawn((
        Transform::from_translation(position),
        RigidBody::Dynamic,
        Collider::sphere(0.2),
        Velocity {
            linear: direction.normalize() * speed,
            angular: Vec3::ZERO,
        },
        Mass::new(0.5),
        Restitution::new(0.3),
    ));
}
```

### Trigger Volumes

Static colliders that detect presence without blocking:

```rust
#[derive(Component)]
struct TriggerZone;

world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    RigidBody::Static,
    Collider::cuboid(5.0, 5.0, 5.0),
    CollisionEventReceiver::default(),
    TriggerZone,
));

fn handle_triggers(
    query: Query<(&TriggerZone, &CollisionEventReceiver)>,
) {
    for (_, receiver) in query.iter() {
        for event in receiver.events() {
            if let CollisionEvent::Started(_, other) = event {
                tracing::info!("Entity {:?} entered trigger", other);
            }
        }
    }
}
```

### Breakable Objects

```rust
#[derive(Component)]
struct Health(f32);

fn damage_on_collision(
    mut commands: Commands,
    query: Query<(Entity, &Health, &CollisionEventReceiver)>,
) {
    for (entity, health, receiver) in query.iter() {
        for event in receiver.events() {
            if let CollisionEvent::Started(_, _) = event {
                if health.0 <= 0.0 {
                    commands.entity(entity).despawn();
                    spawn_debris(&mut commands, entity);
                }
            }
        }
    }
}
```

## Performance Tips

### Use Appropriate Shapes

```rust
// Fast: Sphere, Capsule
Collider::sphere(1.0);        // Fastest
Collider::capsule(2.0, 0.5);  // Very fast

// Medium: Box, Cylinder
Collider::cuboid(1.0, 1.0, 1.0);  // Fast

// Slow: Convex Hull, Triangle Mesh
Collider::convex_hull(points);     // Slower
Collider::trimesh(verts, indices); // Slowest (static only)
```

### Sleeping Bodies

Let inactive bodies sleep:

```rust
world.spawn((
    RigidBody::Dynamic,
    Collider::cuboid(1.0, 1.0, 1.0),
    Sleeping::enabled(),  // Body sleeps when at rest
));
```

### Collision Groups

Reduce collision checks:

```rust
// Bullets don't collide with bullets
let bullet_group = CollisionGroups::new()
    .with_memberships(0b0010)
    .with_filter(0b1101);  // Everything except 0b0010
```

### Fixed Timestep

Physics always runs at consistent rate (default 60 Hz):

```rust
// Configure in PhysicsConfig
PhysicsConfig {
    timestep: 1.0 / 60.0,  // 60 Hz
    ..Default::default()
}
```

## Debugging

### Visualize Colliders

```rust
fn debug_draw_colliders(
    query: Query<(&Transform, &Collider)>,
    mut debug_lines: ResMut<DebugLines>,
) {
    for (transform, collider) in query.iter() {
        match collider.shape() {
            ColliderShape::Sphere { radius } => {
                debug_lines.sphere(transform.translation, *radius, Color::GREEN);
            }
            ColliderShape::Cuboid { half_extents } => {
                debug_lines.box_extents(
                    transform.translation,
                    *half_extents,
                    Color::GREEN
                );
            }
            _ => {}
        }
    }
}
```

### Log Physics State

```rust
fn log_physics_state(query: Query<(&Name, &Velocity, &Transform)>) {
    for (name, velocity, transform) in query.iter() {
        tracing::debug!(
            "{}: pos {:?}, vel {:?}",
            name.as_str(),
            transform.translation,
            velocity.linear
        );
    }
}
```

## Troubleshooting

### Bodies Falling Through Floor

**Problem**: Dynamic bodies pass through static colliders

**Solutions**:
- Ensure physics systems run in correct order (sync→step→sync)
- Check collider size isn't too small
- Verify timestep isn't too large
- Use CCD for fast-moving objects
- Check collision groups aren't filtering

### Bodies Won't Move

**Problem**: Applied forces have no effect

**Solutions**:
- Verify body type is Dynamic, not Static/Kinematic
- Check if body is sleeping (wake it up)
- Ensure `ExternalForces` component exists
- Verify mass isn't too large

### Jittery Physics

**Problem**: Bodies vibrate or shake

**Solutions**:
- Reduce friction/restitution values
- Increase solver iterations in PhysicsConfig
- Check for overlapping colliders at spawn
- Use appropriate mass values (not too small/large)

### Poor Performance

**Problem**: Physics simulation is slow

**Solutions**:
- Use simpler collider shapes (sphere > box > convex)
- Enable sleeping for static objects
- Use collision groups to reduce checks
- Reduce solver iterations if acceptable
- Consider spatial partitioning for many objects

## Examples

See working examples:
- `examples/physics_demo.rs` - Basic physics
- `examples/character_controller.rs` - Character movement
- `examples/projectiles.rs` - Physics projectiles

Run with:
```bash
cargo run --example physics_demo
```

## See Also

- [Physics Concepts](../concepts/physics.md) - Theory and architecture
- [praxis_physics README](../../crates/praxis_physics/README.md) - API documentation
- [Rapier Documentation](https://rapier.rs/docs/) - Underlying physics engine
