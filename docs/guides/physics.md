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

## Design Rationale and Tradeoffs

### Why Rapier3D?

**Decision**: Use Rapier3D as the underlying physics engine

**Rationale**:
- **Pure Rust**: Memory safe, no FFI overhead, integrates seamlessly with Rust ecosystem
- **Performance**: Competitive with established C++ engines (Bullet, PhysX)
- **Active development**: Modern architecture, regular updates, responsive maintainers
- **Feature complete**: Rigid bodies, joints, CCD, sensors - everything needed for games
- **Cross-platform**: Works on all platforms without platform-specific code

**Alternatives Considered**:

| Engine | Pros | Cons | Why Not Chosen |
|--------|------|------|----------------|
| **PhysX** (via physx-rs) | Industry standard, excellent performance, widespread | C++ FFI, unsafe code, complex build | FFI overhead, safety concerns, build complexity |
| **Bullet** (via bullet3-rs) | Mature, well-tested, used in many games | C++ FFI, outdated API, sparse Rust bindings | Poor Rust integration, unmaintained bindings |
| **nphysics** | Pure Rust, predecessor to Rapier | Deprecated, superseded by Rapier | Replaced by Rapier by same author |
| **Custom physics** | Full control, tailored to needs | Massive development effort, bug-prone | Years of effort for equivalent features |

**Key Tradeoff**: Rapier is slightly less mature than PhysX/Bullet but provides vastly better Rust integration and safety guarantees. For a Rust-first engine, this is the right choice.

### ECS Integration Architecture

**Decision**: Component-based physics with bidirectional transform synchronization

**Rationale**:
- Physics properties as ECS components is idiomatic for Bevy ECS architecture
- Queries and systems work naturally with physics data
- No special API needed; users work with familiar ECS patterns
- Maintains single source of truth for entity state (ECS world)

**Synchronization Strategy**:
```
Frame N:
  1. sync_transforms_to_physics     // ECS → Rapier (kinematic bodies)
  2. step_physics_simulation         // Rapier updates dynamic bodies
  3. sync_transforms_from_physics    // Rapier → ECS (dynamic bodies)
  4. apply_external_forces           // Apply forces for next frame
```

**Why Bidirectional Sync?**
- **To Rapier**: Allows game code to move kinematic bodies via Transform component
- **From Rapier**: Updates Transform so rendering system sees physics results
- **Alternative (rejected)**: Physics owns transform entirely - would break scene hierarchy

**Performance Cost**: ~2-5μs per entity for sync
- Mitigated by: Bulk operations, cache-friendly iteration, only syncing changed entities
- Acceptable because: Transform updates needed for rendering anyway

### Fixed Timestep Design

**Decision**: Physics runs at fixed 60Hz independent of frame rate

**Rationale**:
- **Determinism**: Same simulation results regardless of frame rate
- **Stability**: Prevents timestep-dependent instabilities (tunneling, jitter)
- **Network sync**: Fixed timestep essential for multiplayer
- **Replay systems**: Deterministic physics enables replay functionality

**Implementation**: Accumulator pattern
```rust
accumulator += delta_time;
while accumulator >= FIXED_TIMESTEP {
    step_physics(FIXED_TIMESTEP);
    accumulator -= FIXED_TIMESTEP;
}
```

**Why 60Hz?**
- Industry standard for physics simulation
- 16.67ms budget is reasonable for most games
- Higher rates (120Hz) rarely needed unless simulating fast projectiles
- Lower rates (30Hz) cause noticeable jitter and instability

**Alternatives Considered**:
1. **Variable timestep**: Rejected - causes non-determinism and instability
2. **Fixed 30Hz**: Rejected - too low, causes visible jitter
3. **Fixed 120Hz**: Rejected - overkill for most games, wastes CPU
4. **Adaptive timestep**: Too complex, determinism issues

**Tradeoff**: If frame rate drops below 60fps, physics slows down (or skips frames)
- Mitigation: Profile and optimize to maintain 60fps
- Alternative: Allow configurable timestep (advanced users)

### Rigid Body Type Design

**Decision**: Three rigid body types - Static, Dynamic, Kinematic

**Why Three Types?**

| Type | Use Case | CPU Cost | Example |
|------|----------|----------|---------|
| **Static** | Never moves, infinite mass | ~0 (no updates) | Walls, floors, terrain |
| **Dynamic** | Physics controlled | Medium | Balls, boxes, ragdolls |
| **Kinematic** | Code controlled, affects others | Low | Moving platforms, doors |

**Why Not Just Two?**
- Could combine Static + Kinematic as "non-dynamic"
- But: Different CPU paths, different collision handling, different use cases
- Separating them makes intent clear and enables optimizations

**Kinematic vs Static**:
- Kinematic bodies can move (via Transform) and push dynamic bodies
- Static bodies never move and are spatially indexed differently
- Rapier optimizes collision checks based on type (Static-Static checks skipped)

### Collider Shape Hierarchy

**Decision**: Primitive shapes (sphere, box, capsule) plus complex shapes (convex hull, trimesh)

**Performance Hierarchy** (fastest to slowest):
1. **Sphere**: Distance check only, no rotation needed
2. **Capsule**: Two spheres + cylinder, excellent for characters
3. **Box (cuboid)**: Separating axis theorem, very fast
4. **Cylinder**: Slightly slower than box, less common
5. **Convex hull**: GJK algorithm, ~10x slower than primitives
6. **Triangle mesh**: Only for static geometry, slowest

**Why This Set?**
- Primitives cover 90% of game objects efficiently
- Convex hull for custom shapes (rocks, vehicles)
- Triangle mesh for static level geometry (terrain, buildings)

**Alternatives Considered**:
- **Sphere-only physics**: Too limiting, poor approximations
- **Mesh-only physics**: Too slow, unstable
- **Signed distance fields**: Great but complex to author and process

**Design Guidelines**:
- Use primitives wherever possible
- Compound shapes (multiple primitives) better than single convex hull
- Reserve triangle meshes for large static geometry only

**Why No Concave Colliders for Dynamic Bodies?**
- Concave shapes lack clear "inside" and "outside"
- Collision response undefined (which way to push?)
- Solution: Decompose into convex pieces or use compound shapes

### Component-Based Properties

**Decision**: Separate components for Velocity, Mass, Friction, Restitution

**Why Not Single PhysicsBody Component?**
- **Granular queries**: `Query<&Velocity>` vs `Query<&PhysicsBody>` where we only need velocity
- **Optional properties**: Not all bodies need custom mass or friction
- **Performance**: Smaller components improve cache utilization
- **Flexibility**: Mix and match properties independently

**Default Values**:
- Mass: Computed from collider volume and density (1.0)
- Friction: 0.5 (medium)
- Restitution: 0.0 (no bounce)
- Rationale: Sensible defaults reduce boilerplate for common cases

### Collision Event System

**Decision**: Component-based event receivers + resource-based global events

**Two-Tier System**:
1. **Per-entity events**: `CollisionEventReceiver` component
2. **Global events**: `ContactEvents` resource

**Why Both?**
- Per-entity: Fast, direct access for objects that care about their own collisions
- Global: Necessary for systems that need all collision events (audio, particle effects)

**Alternatives Considered**:
1. **Only global events**: Forces all entities to check if event applies to them
2. **Only per-entity**: No way to observe third-party collisions
3. **Callback functions**: Not idiomatic for ECS, hard to serialize

**Performance**: Event storage cost is ~32 bytes per collision per frame
- Mitigated by: Clearing events each frame, only storing what's needed
- Collision groups filter out unwanted collisions before events created

### Collision Groups and Filtering

**Decision**: Bitmask-based collision filtering (16 layers)

**Why Bitmasks?**
- **Fast**: Single bitwise AND operation to check collision
- **Flexible**: Each entity can collide with multiple groups
- **Compact**: 32-bit value stores membership and filter

**16 Layers Rationale**:
- Sufficient for most games (player, enemy, projectile, environment, triggers, etc.)
- Could extend to 32 layers with 64-bit masks if needed
- More layers = more complex management for minimal benefit

**Alternatives Considered**:
1. **Tag-based filtering**: More flexible but much slower (string comparisons)
2. **Raycast-only filtering**: Insufficient - need group-group filtering
3. **Unlimited groups**: Memory waste, overcomplicated API

### External Forces Pattern

**Decision**: Accumulator component for forces, cleared after physics step

**Why Accumulator?**
- Multiple systems can apply forces without coordination
- Forces accumulate additively (realistic physics)
- Cleared automatically after physics step prevents double-application

**Alternative Pattern (rejected)**: Direct velocity manipulation
```rust
// Problematic:
velocity.linear.x += force.x * dt;  // Which dt? Frame dt or physics dt?
velocity.linear.y += force.y * dt;  // Multiple systems = race conditions
```

**Forces vs Impulses**:
- **Force**: Applied over time (continuous), integrated during physics step
- **Impulse**: Instant velocity change (discrete), applied immediately
- Both supported; use forces for thrusters, impulses for explosions

### Sleeping Mechanism

**Decision**: Bodies sleep when at rest, wake on interaction

**Why Sleeping?**
- **Performance**: Sleeping bodies skip integration, constraint solving
- **Typical savings**: 60-80% CPU time in scenes with many static objects
- **Automatic**: Rapier handles sleep/wake transitions

**Sleep Criteria** (configurable):
- Linear velocity < 0.01 m/s
- Angular velocity < 0.01 rad/s  
- No forces applied
- No collisions in last N frames

**Wake Conditions**:
- External force applied
- Collision with awake body
- Manual wake call from code

**Why Not Always Awake?**
- Waste of CPU for objects sitting still (boxes on shelves, etc.)
- Physics stability actually improves (no micro-jitter from floating point errors)

### Continuous Collision Detection (CCD)

**Decision**: Optional per-body CCD for fast-moving objects

**When CCD is Needed**:
- Object speed > collider size / timestep
- Example: Bullet traveling 100 m/s needs CCD if collider < 1.6m (at 60Hz)

**Cost**: 2-3x slower than discrete collision detection

**Why Optional?**
- Most objects don't need CCD (characters, vehicles, debris)
- Always-on CCD wastes CPU for slow objects
- Manual opt-in ensures users understand the cost

### Spatial Query Design

**Decision**: Physics world owns spatial structures, exposed via query API

**Query Types Provided**:
1. **Raycast**: Find first intersection along ray
2. **Raycast all**: Find all intersections along ray
3. **Shape cast**: Sweep shape through space
4. **Point query**: Find all objects at point
5. **AABB query**: Find all objects in box

**Why Physics World Owns Spatial Structures?**
- Rapier maintains spatial indices for collision detection anyway
- Reusing them for queries is free
- Separate spatial structure would duplicate memory and logic

**See Also**: [Spatial Optimization](spatial-optimization.md) for scene-level spatial partitioning

### Integration with Transform Hierarchy

**Decision**: Physics uses GlobalTransform for world position, respects hierarchy

**Why GlobalTransform?**
- Physics operates in world space
- Child physics bodies inherit parent's world transform
- Enables parenting physics objects to moving platforms, vehicles, etc.

**Alternatives Considered**:
1. **Only local Transform**: Doesn't work for nested hierarchies
2. **Flatten all physics bodies**: Breaks parenting, limits scene organization
3. **Physics-specific transform**: Duplicates Transform logic

**Tradeoff**: Transform propagation system must run before physics sync
- Schedule: `transform_propagation → sync_to_physics → step → sync_from_physics`

### Performance Optimization Strategies

**Built-in Optimizations**:
1. **Spatial indexing**: Broadphase collision uses BVH
2. **Sleeping**: Inactive bodies skipped
3. **Island-based solving**: Independent groups solved separately  
4. **Parallel solving**: Multi-threaded constraint resolution
5. **Continuous collision cache**: Reuses calculations from previous frames

**User-Facing Optimizations**:
1. **Simple shapes**: Sphere/capsule are 10x faster than convex hull
2. **Collision groups**: Filter out unnecessary checks
3. **Static geometry**: Mark unchanging objects as Static
4. **Solver iterations**: Reduce for less accurate but faster physics

**When to Profile**:
- >500 dynamic bodies
- >50 active collisions per frame
- Complex compound shapes
- High solver iteration counts

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
