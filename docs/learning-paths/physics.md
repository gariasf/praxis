# Physics Learning Path

Master rigid body physics simulation using Rapier3D for realistic game interactions.

## Path Overview

**Time Investment**: 2-3 weeks  
**Prerequisites**: Basic 3D math and transforms  
**Final Goal**: Build production-ready physics-based gameplay

## Progression Map

```
Beginner (1 week)
├── Rigid body types
├── Basic colliders
├── Physics configuration
└── ECS integration
    ↓
Intermediate (1 week)
├── Collision events
├── Physics queries
├── Advanced colliders
└── Character controllers
    ↓
Advanced (1 week)
├── Custom behaviors
├── Performance optimization
├── Ragdoll physics
└── Physics debugging
```

---

## Beginner: Rigid Body Fundamentals

**Goal**: Create basic physics simulations with rigid bodies and colliders.

### Prerequisites

- ✓ Understanding of 3D transforms
- ✓ Basic physics concepts (mass, velocity, forces)
- ✓ Completed [Getting Started](../getting-started/README.md)

### Step 1: Physics Concepts

**Theory** (2-3 hours):
1. Read [Physics Concepts](../concepts/physics.md)
   - Rigid body types (dynamic, static, kinematic)
   - Colliders and collision detection
   - Physics simulation loop
   - Integration with ECS

2. Read crate documentation: `crates/praxis_physics/README.md`

**Key Concepts**:
- **Dynamic**: Affected by forces and gravity
- **Static**: Never moves, infinite mass
- **Kinematic**: Moved by code, affects dynamic bodies

### Step 2: Basic Setup

**Practice** (3-4 hours):
1. Read [Physics Guide](../guides/physics.md) - Quick Start section
2. Initialize physics world
3. Configure physics settings

**Code Pattern**:
```rust
use praxis_physics::{PhysicsWorld, PhysicsConfig};

// Setup physics
world.insert_resource(PhysicsWorld::new());
world.insert_resource(PhysicsConfig {
    gravity: Vec3::new(0.0, -9.81, 0.0),
    timestep: 1.0 / 60.0,
    ..Default::default()
});

// Add systems to schedule
schedule.add_systems((
    sync_transforms_to_physics,
    step_physics_simulation,
    sync_transforms_from_physics,
).chain());
```

**Understanding Goal**: How physics integrates with ECS

### Step 3: Dynamic Bodies

**Practice** (4-5 hours):
1. Create falling objects
2. Adjust physics properties
3. Observe behavior

**Exercises**:
1. Drop a sphere from height
2. Create bouncing balls with different restitution
3. Stack boxes with different masses
4. Create a Newton's cradle

**Code Examples**:
```rust
// Bouncing ball
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Dynamic,
    Collider::sphere(0.5),
    Restitution::new(0.9),  // Bouncy
    Mass::new(1.0),
));

// Heavy box
world.spawn((
    Transform::from_xyz(2.0, 5.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Dynamic,
    Collider::cuboid(1.0, 1.0, 1.0),
    Mass::new(10.0),
    Friction::new(0.5),
));
```

### Step 4: Static and Kinematic Bodies

**Practice** (3-4 hours):
1. Create static ground/walls
2. Create moving platforms (kinematic)
3. Test interactions

**Exercises**:
1. Build a room with walls
2. Create static ramps
3. Add moving platform
4. Create rotating platform

**Static Bodies**:
```rust
// Ground
world.spawn((
    Transform::default(),
    GlobalTransform::default(),
    RigidBody::Static,
    Collider::cuboid(50.0, 0.5, 50.0),
));
```

**Kinematic Bodies**:
```rust
// Moving platform
world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    GlobalTransform::default(),
    RigidBody::Kinematic,
    Collider::cuboid(5.0, 0.5, 5.0),
    MovingPlatform,  // Custom component
));

// System to move platform
fn move_platform_system(
    mut query: Query<&mut Transform, With<MovingPlatform>>,
    time: Res<Time>,
) {
    for mut transform in query.iter_mut() {
        transform.translation.y = 2.0 + (time.elapsed().sin() * 3.0);
    }
}
```

### Step 5: Collider Shapes

**Practice** (3-4 hours):
1. Use different collider types
2. Understand shape properties
3. Choose appropriate shapes

**Available Shapes**:
- `Collider::sphere(radius)`
- `Collider::cuboid(hx, hy, hz)`
- `Collider::capsule(half_height, radius)`
- `Collider::cylinder(half_height, radius)`
- `Collider::cone(half_height, radius)`

**Exercises**:
1. Create ball pit (spheres)
2. Create domino chain (boxes)
3. Create bowling pins (capsules)
4. Create barrel (cylinder)

**Collider Selection Guide**:
- **Sphere**: Fastest, use for projectiles
- **Cuboid**: Boxes, walls, platforms
- **Capsule**: Characters, pills
- **Cylinder**: Barrels, pillars
- **Cone**: Rarely needed

### Beginner Checkpoint

**Self-Assessment**:
- [ ] Can create dynamic, static, and kinematic bodies
- [ ] Understand collider shapes and properties
- [ ] Know how to configure physics properties
- [ ] Understand ECS-physics synchronization
- [ ] Can build basic physics scenes

**Capstone Project**: Build a physics playground with:
- Ground and walls (static)
- 20+ falling objects (spheres, boxes, capsules)
- Moving platform (kinematic)
- Adjustable physics properties

**Time to Complete**: 15-20 hours

---

## Intermediate: Collisions and Interactions

**Goal**: Handle collision events and implement complex physics interactions.

### Prerequisites

- ✓ Completed Beginner section
- ✓ Comfortable with rigid bodies
- ✓ Understanding of ECS queries

### Step 1: Collision Events

**Theory** (2-3 hours):
1. Continue [Physics Guide: Collision Events](../guides/physics.md)
2. Understand event types:
   - Collision started
   - Collision ongoing
   - Collision ended

**Practice** (4-5 hours):
1. Implement collision handlers
2. React to collisions
3. Track collision pairs

**Code Pattern**:
```rust
fn collision_handler(
    mut collision_events: EventReader<CollisionEvent>,
    query: Query<&Name>,
) {
    for event in collision_events.iter() {
        match event {
            CollisionEvent::Started(e1, e2) => {
                let name1 = query.get(*e1).unwrap();
                let name2 = query.get(*e2).unwrap();
                println!("{} hit {}", name1, name2);
            }
            CollisionEvent::Stopped(e1, e2) => {
                // Collision ended
            }
        }
    }
}
```

**Exercises**:
1. Play sound on collision
2. Apply damage on impact
3. Spawn particles on collision
4. Track collision count

### Step 2: Raycasting

**Theory** (2 hours):
1. Understand raycasting
2. Use cases (hit detection, line of sight)

**Practice** (4-5 hours):
1. Implement basic raycast
2. Process raycast results
3. Visualize rays

**Code Pattern**:
```rust
fn raycast_system(
    physics_world: Res<PhysicsWorld>,
) {
    let ray_origin = Vec3::new(0.0, 5.0, 0.0);
    let ray_direction = Vec3::new(0.0, -1.0, 0.0);
    let max_distance = 100.0;

    if let Some(hit) = physics_world.raycast(
        ray_origin,
        ray_direction,
        max_distance,
        true,  // query trigger
    ) {
        println!("Hit at distance: {}", hit.time_of_impact);
        println!("Hit entity: {:?}", hit.entity);
        println!("Hit point: {:?}", hit.point);
        println!("Hit normal: {:?}", hit.normal);
    }
}
```

**Exercises**:
1. Shoot raycast from camera
2. Implement click-to-select
3. Check line of sight between entities
4. Ground detection for character

### Step 3: Physics Queries

**Practice** (3-4 hours):
1. Shape casting
2. Overlap queries
3. Spatial queries

**Query Types**:
```rust
// Check if point is inside collider
physics_world.point_inside(point, entity);

// Get all entities in sphere
physics_world.sphere_overlap(center, radius);

// Shape cast (moving shape)
physics_world.cast_shape(shape, start, direction, max_distance);
```

**Exercises**:
1. Explosion radius (sphere overlap)
2. Trigger zones (point inside)
3. Swept collision (shape cast)
4. Area of effect detection

### Step 4: Advanced Colliders

**Theory** (2-3 hours):
1. Compound colliders
2. Trimesh colliders
3. Convex hull colliders

**Practice** (5-6 hours):
1. Create compound shapes
2. Use mesh colliders for terrain
3. Generate convex hulls

**Compound Collider**:
```rust
// Car made of multiple shapes
let car_body = Collider::cuboid(2.0, 1.0, 4.0);
let wheels = vec![
    (Vec3::new(-1.5, -1.0, 2.0), Collider::sphere(0.5)),
    (Vec3::new(1.5, -1.0, 2.0), Collider::sphere(0.5)),
    (Vec3::new(-1.5, -1.0, -2.0), Collider::sphere(0.5)),
    (Vec3::new(1.5, -1.0, -2.0), Collider::sphere(0.5)),
];

let compound = Collider::compound(
    vec![car_body],
    wheels,
);
```

**Exercises**:
1. Build complex object from primitives
2. Use terrain mesh as trimesh collider
3. Create convex hull for custom mesh

### Step 5: Character Controller

**Theory** (3 hours):
1. Character controller concepts
2. Ground detection
3. Slope handling
4. Step climbing

**Practice** (6-8 hours):
1. Implement kinematic character controller
2. Add movement and rotation
3. Handle ground detection
4. Implement jumping

**Basic Controller**:
```rust
#[derive(Component)]
struct CharacterController {
    speed: f32,
    jump_force: f32,
    grounded: bool,
}

fn character_controller_system(
    mut query: Query<(
        &CharacterController,
        &mut Transform,
        &mut Velocity,
    )>,
    input: Res<Input<KeyCode>>,
    physics_world: Res<PhysicsWorld>,
) {
    for (controller, mut transform, mut velocity) in query.iter_mut() {
        // Ground check
        let ray_origin = transform.translation;
        let ray_dir = Vec3::NEG_Y;
        controller.grounded = physics_world
            .raycast(ray_origin, ray_dir, 0.1, true)
            .is_some();

        // Movement
        let mut move_dir = Vec3::ZERO;
        if input.pressed(KeyCode::W) { move_dir.z -= 1.0; }
        if input.pressed(KeyCode::S) { move_dir.z += 1.0; }
        if input.pressed(KeyCode::A) { move_dir.x -= 1.0; }
        if input.pressed(KeyCode::D) { move_dir.x += 1.0; }

        velocity.linvel = move_dir.normalize_or_zero() * controller.speed;

        // Jump
        if input.just_pressed(KeyCode::Space) && controller.grounded {
            velocity.linvel.y = controller.jump_force;
        }
    }
}
```

**Exercises**:
1. WASD movement
2. Space to jump
3. Slope handling
4. Stair climbing

**Cross-Reference**: Combine with [Animation Path](animation.md) for animated character

### Intermediate Checkpoint

**Self-Assessment**:
- [ ] Can handle collision events
- [ ] Understand raycasting and queries
- [ ] Can use advanced colliders
- [ ] Built a character controller
- [ ] Know how to debug physics

**Capstone Project**: Third-person character with:
- Kinematic character controller
- Ground detection and jumping
- Slope handling
- Collision detection with environment
- Integration with input system

**Time to Complete**: 20-30 hours

---

## Advanced: Custom Integration

**Goal**: Optimize physics, create custom behaviors, implement ragdolls.

### Prerequisites

- ✓ Completed Intermediate section
- ✓ Built character controller
- ✓ Strong understanding of physics concepts

### Step 1: Physics Materials

**Theory** (2 hours):
1. Friction coefficients
2. Restitution (bounciness)
3. Material combination

**Practice** (3-4 hours):
1. Create material presets
2. Test interactions
3. Tune for gameplay feel

**Material Examples**:
```rust
// Ice (low friction, medium bounce)
let ice = PhysicsMaterial {
    friction: 0.02,
    restitution: 0.3,
    ..Default::default()
};

// Rubber (high friction, high bounce)
let rubber = PhysicsMaterial {
    friction: 0.9,
    restitution: 0.95,
    ..Default::default()
};

// Wood (medium friction, low bounce)
let wood = PhysicsMaterial {
    friction: 0.5,
    restitution: 0.2,
    ..Default::default()
};
```

**Exercises**:
1. Create ice rink
2. Add bouncy castle
3. Build wooden structures
4. Test material combinations

### Step 2: Joints and Constraints

**Theory** (3 hours):
1. Joint types (fixed, revolute, prismatic, spherical)
2. Constraints and limits
3. Motors and drives

**Practice** (5-6 hours):
1. Create hinges (doors, gates)
2. Create chains (rope, cable)
3. Create vehicles (wheels)

**Joint Examples**:
```rust
// Door hinge
let hinge = RevoluteJoint::new(Vec3::Y)  // Rotation axis
    .limit_angle(-PI / 2.0, PI / 2.0);   // -90° to +90°

// Chain link
let chain_link = SphericalJoint::new()
    .limit_angle(0.5);  // Max bend angle

// Spring
let spring = PrismaticJoint::new(Vec3::Y)
    .spring(100.0, 10.0);  // Stiffness, damping
```

**Exercises**:
1. Create swinging door
2. Build rope bridge
3. Create vehicle with suspension
4. Build crane with cable

### Step 3: Ragdoll Physics

**Theory** (3-4 hours):
1. Ragdoll architecture
2. Joint hierarchy
3. Collision setup
4. Activation/deactivation

**Practice** (8-10 hours):
1. Create ragdoll from skeleton
2. Configure joints and limits
3. Transition from animated to ragdoll
4. Blend back to animation

**Ragdoll Pattern**:
```rust
// Generate ragdoll from skeleton
fn create_ragdoll(
    skeleton: &Skeleton,
    commands: &mut Commands,
) -> Entity {
    let ragdoll = commands.spawn_empty().id();

    // Create body part for each bone
    for (index, bone) in skeleton.bones().iter().enumerate() {
        let body = commands.spawn((
            Transform::from_translation(bone.position),
            GlobalTransform::default(),
            RigidBody::Dynamic,
            Collider::capsule(bone.length / 2.0, 0.1),
            RagdollBone { bone_index: index },
        )).id();

        // Add joint to parent bone
        if let Some(parent_index) = bone.parent {
            let joint = create_joint_for_bone(bone);
            commands.entity(body).insert(joint);
        }
    }

    ragdoll
}
```

**Exercises**:
1. Create basic ragdoll
2. Add to character on death
3. Blend from animation to ragdoll
4. Apply impact forces
5. Recover from ragdoll (get up animation)

**Cross-Reference**: Combine with [Animation Path](animation.md) for seamless transitions

### Step 4: Performance Optimization

**Theory** (2-3 hours):
1. Review [Performance Path](performance.md) for profiling basics
2. Understand physics-specific performance characteristics
3. Learn to identify physics bottlenecks

**Optimization Techniques**:
- Simplify colliders
- Use collision groups
- Adjust solver iterations
- Sleep inactive bodies
- Use spatial queries efficiently

**Practice** (5-6 hours):
1. Profile physics system
2. Optimize collider complexity
3. Configure collision groups
4. Tune solver settings

**Collision Groups**:
```rust
// Define groups
const PLAYER: u32 = 0b0001;
const ENEMY: u32 = 0b0010;
const PROJECTILE: u32 = 0b0100;
const ENVIRONMENT: u32 = 0b1000;

// Player collides with environment and enemies
world.spawn((
    RigidBody::Dynamic,
    Collider::capsule(1.0, 0.5),
    CollisionGroups::new(
        PLAYER,                        // I am player
        ENVIRONMENT | ENEMY,           // I collide with these
    ),
));

// Projectile doesn't collide with player
world.spawn((
    RigidBody::Dynamic,
    Collider::sphere(0.1),
    CollisionGroups::new(
        PROJECTILE,
        ENEMY | ENVIRONMENT,  // Only enemy and environment
    ),
));
```

**Exercises**:
1. Profile scene with 1000 physics objects
2. Optimize to 60 FPS
3. Setup collision groups for game
4. Measure impact of solver iterations

### Step 5: Physics Debugging

**Practice** (3-4 hours):
1. Visualize colliders
2. Debug collision issues
3. Inspect physics state

**Debug Techniques**:
```rust
// Visualize colliders
fn draw_colliders(
    query: Query<(&GlobalTransform, &Collider)>,
    mut gizmos: Gizmos,
) {
    for (transform, collider) in query.iter() {
        match collider.shape() {
            Shape::Sphere { radius } => {
                gizmos.sphere(transform.translation(), *radius, Color::GREEN);
            }
            Shape::Cuboid { half_extents } => {
                gizmos.cuboid(transform, *half_extents, Color::GREEN);
            }
            // ... other shapes
        }
    }
}

// Log collision information
fn debug_collisions(
    mut collision_events: EventReader<CollisionEvent>,
) {
    for event in collision_events.iter() {
        println!("Collision: {:?}", event);
    }
}
```

**Exercises**:
1. Enable collider visualization
2. Debug stuck objects
3. Fix tunneling issues
4. Identify performance problems

### Advanced Checkpoint

**Self-Assessment**:
- [ ] Can tune physics materials
- [ ] Understand joints and constraints
- [ ] Implemented ragdoll system
- [ ] Optimized physics performance
- [ ] Can debug physics issues effectively

**Capstone Project**: Choose one:

1. **Vehicle System**: Full vehicle with suspension, wheels, and steering
2. **Destruction System**: Buildings that break apart realistically
3. **Ragdoll System**: Full character ragdoll with animation blending

**Time to Complete**: 25-35 hours

---

## Cross-References

### Related Systems
- [Animation Path](animation.md) - Animated characters, ragdolls
- [Networking Path](networking.md) - Physics replication
- [Scripting Path](scripting.md) - Script physics behaviors

### Performance
- [Performance Path](performance.md) - Profiling physics systems
- [Spatial Optimization](../guides/spatial-optimization.md) - Spatial acceleration structures

### Integration
- [Input Guide](../guides/input.md) - Character controls
- [Audio Guide](../guides/audio.md) - Collision sounds

---

## Practice Resources

### Examples
```bash
# Test physics in action
cargo run --example physics_demo
```

### External Resources
- Rapier3D documentation: https://rapier.rs/
- Physics for Game Programmers
- Game Physics Engine Development

---

## Next Steps

After completing this path:

1. **Specialize**: Vehicles, destruction, cloth simulation
2. **Integrate**: Combine with animation for characters
3. **Network**: Physics replication for multiplayer
4. **Optimize**: Large-scale physics simulations

---

[← Back to Learning Paths](README.md) | [Next: Scripting Path →](scripting.md)
