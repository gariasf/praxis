# ECS Design Patterns

This document presents common Entity-Component-System design patterns used in Praxis. These patterns help structure game logic efficiently, maintain clean code, and leverage the ECS architecture for maximum performance.

**Related Architecture Documentation:**
- [ECS System Execution Order](ecs-system-execution-order.md) - Visual guide to when systems run and how they interact
- [ECS Architecture](../concepts/ecs-architecture.md) - Core ECS concepts and theory

## Introduction to ECS

The Entity-Component-System architecture separates data (Components) from behavior (Systems):

- **Entities**: Unique identifiers (just IDs)
- **Components**: Pure data structures
- **Systems**: Logic that operates on components

```rust
// Component: Pure data
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

// System: Pure logic
fn damage_system(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        health.current = (health.current - 10.0).max(0.0);
    }
}
```

This separation enables:
- **Composition over Inheritance**: Build entities from components
- **Data-Oriented Design**: Excellent cache performance
- **Parallelization**: Systems can run concurrently
- **Flexibility**: Easy to add/remove behavior dynamically

## Core Patterns

### 1. Component Composition

Build complex entities from simple, reusable components.

#### Basic Composition

```rust
use praxis_ecs::{World, Transform, GlobalTransform};

// A static prop
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    GlobalTransform::default(),
    MeshHandle::new("rock"),
    Name::new("Boulder"),
));

// An interactive object
world.spawn((
    Transform::from_xyz(0.0, 1.0, 0.0),
    GlobalTransform::default(),
    MeshHandle::new("crate"),
    MaterialHandle::new("wood"),
    RigidBody::Dynamic,
    Collider::cuboid(1.0, 1.0, 1.0),
    Name::new("Crate"),
));

// An NPC character
world.spawn((
    Transform::from_xyz(5.0, 0.0, 5.0),
    GlobalTransform::default(),
    MeshHandle::new("character"),
    Health { current: 100.0, max: 100.0 },
    Velocity::default(),
    NavAgent::new(),
    Name::new("Guard"),
));
```

**Benefits**:
- Same components reused across entity types
- Easy to add new functionality (add a component)
- Clear data dependencies

**Antipattern**:
```rust
// DON'T: Kitchen-sink component
#[derive(Component)]
struct GameObject {
    position: Vec3,
    rotation: Quat,
    mesh: String,
    health: f32,
    velocity: Vec3,
    ai_state: AIState,
    inventory: Vec<Item>,
    // ... grows forever
}
```

Instead, split into focused components.

### 2. Marker Components

Use empty components as tags to categorize entities.

```rust
#[derive(Component, Default)]
struct Player;

#[derive(Component, Default)]
struct Enemy;

#[derive(Component, Default)]
struct Projectile;

#[derive(Component, Default)]
struct Static;

// Spawn entities with markers
world.spawn((
    Transform::default(),
    Player,
    Health { current: 100.0, max: 100.0 },
));

// Query only player entities
fn player_input_system(query: Query<(&Transform, &Velocity), With<Player>>) {
    for (transform, velocity) in query.iter() {
        // Handle player input
    }
}

// Query excluding specific entities
fn ai_system(query: Query<&Transform, (With<Enemy>, Without<Player>)>) {
    for transform in query.iter() {
        // AI logic for enemies only
    }
}
```

**Use Cases**:
- Entity categorization (Player, Enemy, NPC)
- Behavior flags (Frozen, Stunned, Invulnerable)
- System filters (Visible, Active, Selected)

**Benefits**:
- Zero memory overhead
- Fast filtering
- Clear intent in code

### 3. Flag Components

Similar to markers but with boolean semantics.

```rust
#[derive(Component)]
struct Visible(bool);

#[derive(Component)]
struct Active(bool);

#[derive(Component)]
struct Culled(bool);

// Toggle visibility
fn toggle_visibility(mut query: Query<&mut Visible>) {
    for mut visible in query.iter_mut() {
        visible.0 = !visible.0;
    }
}

// Conditional processing
fn render_system(query: Query<(&MeshHandle, &Transform), With<Visible>>) {
    for (mesh, transform) in query.iter() {
        // Only render visible entities
    }
}
```

**Built-in Examples**:
```rust
use praxis_ecs::{Visible, Active, Culled};

// Praxis provides standard flags
world.spawn((
    Transform::default(),
    Visible::default(),    // For rendering
    Active::default(),     // For simulation
    Culled::default(),     // For frustum culling
));
```

### 4. Hierarchical Entities

Model parent-child relationships for transforms and logic.

```rust
use praxis_ecs::{Parent, Children, Transform, GlobalTransform};

// Create parent
let parent = world.spawn((
    Transform::from_xyz(10.0, 0.0, 0.0),
    GlobalTransform::default(),
    Name::new("Parent"),
));

// Create child with Parent component
let child = world.spawn((
    Transform::from_xyz(5.0, 0.0, 0.0),  // Relative to parent
    GlobalTransform::default(),
    Parent(parent),  // Links to parent
    Name::new("Child"),
));

// Parent automatically gets Children component
// Child's global position will be (15, 0, 0)
```

**Transform Propagation**:
```rust
use praxis_ecs::systems::*;

schedule.add_systems((
    sync_parent_child_relationships,
    cleanup_removed_parents,
    propagate_transforms,
    propagate_transforms_for_reparented,
    propagate_transforms_for_changed_children,
).chain());
```

**Use Cases**:
- Skeletal animation (bones hierarchy)
- Vehicle with turret (turret follows vehicle)
- UI layout (panels, buttons, text)
- Scene graph (spatial organization)

**Example: Turret on Tank**:
```rust
// Tank (parent)
let tank = world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    MeshHandle::new("tank_body"),
    Name::new("Tank"),
));

// Turret (child) - rotates independently
let turret = world.spawn((
    Transform::from_xyz(0.0, 1.5, 0.0)
        .with_rotation(Quat::from_rotation_y(PI / 4.0)),
    GlobalTransform::default(),
    Parent(tank),
    MeshHandle::new("tank_turret"),
    Name::new("Turret"),
));

// Barrel (child of turret) - pitches independently  
let barrel = world.spawn((
    Transform::from_xyz(0.0, 0.5, 2.0)
        .with_rotation(Quat::from_rotation_x(-PI / 8.0)),
    GlobalTransform::default(),
    Parent(turret),
    MeshHandle::new("tank_barrel"),
    Name::new("Barrel"),
));

// System to rotate turret toward target
fn turret_aim_system(
    mut turrets: Query<&mut Transform, With<Turret>>,
    targets: Query<&GlobalTransform, With<TargetMarker>>,
) {
    if let Ok(target) = targets.get_single() {
        for mut turret_transform in turrets.iter_mut() {
            let direction = target.translation() - turret_transform.translation;
            let rotation = Quat::from_rotation_arc(Vec3::Z, direction.normalize());
            turret_transform.rotation = rotation;
        }
    }
}
```

### 5. Resource Components

Share global state across systems.

```rust
use praxis_ecs::{Resource, Res, ResMut};

#[derive(Resource, Default)]
struct GameSettings {
    volume: f32,
    difficulty: Difficulty,
}

#[derive(Resource, Default)]
struct Score(u32);

#[derive(Resource)]
struct DeltaTime(Duration);

// Insert resources
world.insert_resource(GameSettings::default());
world.insert_resource(Score(0));
world.insert_resource(DeltaTime(Duration::from_secs_f32(1.0/60.0)));

// Access in systems
fn scoring_system(mut score: ResMut<Score>) {
    score.0 += 10;
}

fn physics_system(
    time: Res<DeltaTime>,
    mut query: Query<(&mut Transform, &Velocity)>
) {
    let dt = time.0.as_secs_f32();
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * dt;
    }
}
```

**Built-in Resources**:
```rust
use praxis_ecs::{LightingData, DeltaTime};

fn gather_lights(mut lighting: ResMut<LightingData>) {
    // Lighting data automatically populated by gather_lighting_system
}
```

**When to Use**:
- Global configuration
- Frame timing information
- Input state
- Shared caches/pools

**Antipattern**:
```rust
// DON'T: Use resources for entity-specific data
#[derive(Resource)]
struct AllEnemies {
    enemies: Vec<(Entity, Transform, Health)>,
}
```

Use queries instead - ECS handles this efficiently.

### 6. State Components

Implement state machines as components.

```rust
#[derive(Component)]
enum AIState {
    Idle,
    Patrolling { waypoint_index: usize },
    Chasing { target: Entity },
    Attacking { target: Entity, cooldown: f32 },
    Fleeing { threat: Entity },
}

fn ai_state_machine(
    mut query: Query<(&mut AIState, &mut Transform, &Health)>,
    time: Res<DeltaTime>,
) {
    for (mut state, mut transform, health) in query.iter_mut() {
        match *state {
            AIState::Idle => {
                // Check for nearby enemies
                // Transition to Chasing or Patrolling
                if detect_enemy() {
                    *state = AIState::Chasing { target: enemy_entity };
                }
            }
            AIState::Patrolling { waypoint_index } => {
                // Move to waypoint
                // Check if reached, advance to next
                if at_waypoint() {
                    *state = AIState::Patrolling { 
                        waypoint_index: (waypoint_index + 1) % total_waypoints 
                    };
                }
            }
            AIState::Chasing { target } => {
                // Move toward target
                // Check if in attack range
                if in_range(target) {
                    *state = AIState::Attacking { target, cooldown: 0.0 };
                }
                // Check if lost target
                if !can_see(target) {
                    *state = AIState::Patrolling { waypoint_index: 0 };
                }
            }
            AIState::Attacking { target, ref mut cooldown } => {
                // Attack when cooldown ready
                *cooldown -= time.0.as_secs_f32();
                if *cooldown <= 0.0 {
                    attack(target);
                    *cooldown = 2.0;
                }
                // Check health for fleeing
                if health.current < health.max * 0.25 {
                    *state = AIState::Fleeing { threat: target };
                }
            }
            AIState::Fleeing { threat } => {
                // Run away from threat
                // Check if escaped
                if far_enough(threat) {
                    *state = AIState::Idle;
                }
            }
        }
    }
}
```

**Benefits**:
- Clear state transitions
- Self-documenting behavior
- Easy to debug (inspect component)

**Alternative: Multiple Components**:
```rust
// Instead of enum, use separate components
#[derive(Component)] struct Idle;
#[derive(Component)] struct Patrolling { waypoint_index: usize }
#[derive(Component)] struct Chasing { target: Entity }

// Add/remove components to change state
commands.entity(entity)
    .remove::<Idle>()
    .insert(Chasing { target: player });
```

Choose based on:
- **Enum**: Few states, frequently switching
- **Components**: Many states, rarely switching

### 7. Event Components

Use components as temporary event markers.

```rust
#[derive(Component)]
struct DamageEvent {
    amount: f32,
    source: Entity,
}

#[derive(Component)]
struct HealEvent {
    amount: f32,
}

// Producer: Create events
fn collision_system(
    mut commands: Commands,
    collisions: Query<(&Projectile, &Transform)>,
    entities: Query<Entity, With<Health>>,
) {
    for (projectile, transform) in collisions.iter() {
        // Check for hits
        if let Some(hit_entity) = check_hit(transform, &entities) {
            commands.entity(hit_entity).insert(DamageEvent {
                amount: projectile.damage,
                source: projectile.owner,
            });
        }
    }
}

// Consumer: Process and remove events
fn health_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Health, Option<&DamageEvent>, Option<&HealEvent>)>,
) {
    for (entity, mut health, damage, heal) in query.iter_mut() {
        if let Some(damage) = damage {
            health.current -= damage.amount;
            commands.entity(entity).remove::<DamageEvent>();
        }
        
        if let Some(heal) = heal {
            health.current = (health.current + heal.amount).min(health.max);
            commands.entity(entity).remove::<HealEvent>();
        }
    }
}
```

**Scheduling**:
```rust
schedule.add_systems((
    collision_system,     // Produces events
    health_system,        // Consumes events
).chain());
```

**Benefits**:
- Decoupled systems
- Testable in isolation
- Clear data flow

**Alternative: Bevy Events**:
```rust
// For global events not tied to entities
#[derive(Event)]
struct LevelCompleteEvent;

fn trigger_level_complete(mut events: EventWriter<LevelCompleteEvent>) {
    events.send(LevelCompleteEvent);
}

fn handle_level_complete(mut events: EventReader<LevelCompleteEvent>) {
    for _event in events.read() {
        // Handle level completion
    }
}
```

### 8. Query Composition

Build complex queries with filters and combinators.

```rust
use praxis_ecs::{With, Without, Or, Changed, Added};

// Multiple required components
fn system(query: Query<(&Transform, &Velocity, &Mass)>) {
    // Entities must have all three components
}

// With filter: Require component without reading it
fn render_visible_system(query: Query<(&Transform, &MeshHandle), With<Visible>>) {
    // Only entities with Visible component
}

// Without filter: Exclude component
fn ai_npcs_system(query: Query<&AIState, (With<NPC>, Without<Player>)>) {
    // NPCs but not players
}

// Or filter: Match any condition
fn projectile_system(query: Query<&Transform, Or<(With<Bullet>, With<Arrow>, With<Fireball>)>>) {
    // Any projectile type
}

// Changed filter: Only entities with modified components
fn update_on_move(query: Query<&Transform, Changed<Transform>>) {
    // Only entities whose transform changed this frame
}

// Added filter: Only entities that just got component
fn initialize_system(query: Query<&Transform, Added<Health>>) {
    // Entities that just got Health component
}

// Complex combinations
fn complex_system(
    query: Query<
        (&Transform, &Velocity, &Health),
        (
            Or<(With<Player>, With<Ally>)>,
            Without<Dead>,
            Changed<Health>,
        )
    >
) {
    // Players or allies, not dead, whose health changed
}
```

**Performance Tips**:
- Filters are zero-cost (compile-time)
- More specific queries = better cache performance
- Changed/Added use change detection (small overhead)

### 9. System Ordering

Control execution order for dependent systems.

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs, SystemSet};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
enum GameLoop {
    Input,
    PreUpdate,
    Update,
    PostUpdate,
    Render,
}

let mut schedule = Schedule::default();

// Configure set ordering
schedule.configure_sets((
    GameLoop::Input,
    GameLoop::PreUpdate,
    GameLoop::Update,
    GameLoop::PostUpdate,
    GameLoop::Render,
).chain());

// Add systems to sets
schedule.add_systems(
    (
        keyboard_input_system,
        mouse_input_system,
    ).in_set(GameLoop::Input)
);

schedule.add_systems(
    (
        transform_propagation_system,
        camera_update_system,
    ).chain()  // Explicit ordering within set
    .in_set(GameLoop::PreUpdate)
);

schedule.add_systems(
    (
        physics_system,
        animation_system,
        ai_system,
    ).in_set(GameLoop::Update)
);

schedule.add_systems(
    (
        frustum_culling_system,
        gather_lighting_system,
    ).chain()
    .in_set(GameLoop::PostUpdate)
);

schedule.add_systems(
    render_system.in_set(GameLoop::Render)
);
```

**Built-in System Sets**:
```rust
use praxis_ecs::systems::CoreSystemSet;

// Praxis defines standard sets
schedule.add_systems(
    my_system
        .in_set(CoreSystemSet::TransformPropagate)
        .after(sync_parent_child_relationships)
);
```

**Dependencies**:
```rust
schedule.add_systems((
    system_a.before(system_b),
    system_b.after(system_a),
    system_c.before(system_b).after(system_a),
));

// Equivalent with chain
schedule.add_systems((
    system_a,
    system_c,
    system_b,
).chain());
```

### 10. Component Bundles

Group commonly used components.

```rust
use praxis_ecs::Bundle;

#[derive(Bundle)]
struct TransformBundle {
    transform: Transform,
    global_transform: GlobalTransform,
}

impl TransformBundle {
    fn from_xyz(x: f32, y: f32, z: f32) -> Self {
        Self {
            transform: Transform::from_xyz(x, y, z),
            global_transform: GlobalTransform::default(),
        }
    }
}

#[derive(Bundle)]
struct CharacterBundle {
    #[bundle]
    transform: TransformBundle,
    mesh: MeshHandle,
    health: Health,
    velocity: Velocity,
    name: Name,
}

// Spawn with bundle
world.spawn(CharacterBundle {
    transform: TransformBundle::from_xyz(0.0, 0.0, 0.0),
    mesh: MeshHandle::new("character"),
    health: Health { current: 100.0, max: 100.0 },
    velocity: Velocity::default(),
    name: Name::new("Hero"),
});
```

**Built-in Bundles**:
```rust
use praxis_ecs::systems::{TransformBundle, PerspectiveCameraBundle, OrthographicCameraBundle};

// Transform bundle
world.spawn(TransformBundle::from_xyz(10.0, 5.0, 0.0));

// Camera bundle
world.spawn(PerspectiveCameraBundle::new(
    Vec3::new(0.0, 5.0, 10.0),
    70.0_f32.to_radians(),
    16.0 / 9.0,
));
```

**Benefits**:
- Ensures complete entity setup
- Reduces boilerplate
- Self-documenting entity types
- Easy to extend

## Advanced Patterns

### 11. Archetype Optimization

Organize entities to maximize cache performance.

```rust
// GOOD: Entities with same components stored together
for _ in 0..1000 {
    world.spawn((Transform::default(), Velocity::default()));
}
for _ in 0..1000 {
    world.spawn((Transform::default(), Velocity::default(), Health::default()));
}

// BAD: Interleaved different archetypes
for i in 0..1000 {
    if i % 2 == 0 {
        world.spawn((Transform::default(), Velocity::default()));
    } else {
        world.spawn((Transform::default(), Velocity::default(), Health::default()));
    }
}
```

**Why?**: ECS stores entities in "archetypes" (tables) by component combination. Entities in same archetype are stored contiguously for optimal cache access.

**Measurement**:
```rust
// Check entity distribution
let world_info = world.inner().entities();
println!("Total entities: {}", world_info.len());

// Monitor archetype count (many archetypes = fragmentation)
// Ideally: Few archetypes with many entities each
```

### 12. Change Detection

React only to modified data.

```rust
// Only process entities whose transform changed
fn sync_physics_system(
    query: Query<(&Transform, &RigidBody), Changed<Transform>>,
) {
    for (transform, rigid_body) in query.iter() {
        rigid_body.set_position(transform.translation);
        rigid_body.set_rotation(transform.rotation);
    }
}

// Process all entities first time, then only changes
fn caching_system(
    query: Query<(&Transform, &MeshHandle), Or<(Changed<Transform>, Changed<MeshHandle>)>>,
    mut cache: Local<HashMap<Entity, CachedData>>,
) {
    for (entity, (transform, mesh)) in query.iter() {
        // Recompute only when data changes
        let cached = compute_expensive_data(transform, mesh);
        cache.insert(entity, cached);
    }
}
```

**Change Detection Caveats**:
- Change detection persists across system runs
- Must call `world.clear_trackers()` in tests
- Mutable access always marks as changed

### 13. Local System State

Store per-system persistent data.

```rust
use praxis_ecs::Local;

#[derive(Default)]
struct SpawnTimer {
    cooldown: f32,
}

fn enemy_spawner_system(
    mut commands: Commands,
    time: Res<DeltaTime>,
    mut timer: Local<SpawnTimer>,
) {
    timer.cooldown -= time.0.as_secs_f32();
    
    if timer.cooldown <= 0.0 {
        // Spawn enemy
        commands.spawn((
            Transform::from_xyz(rand(), 0.0, rand()),
            Enemy,
            Health::default(),
        ));
        
        timer.cooldown = 5.0; // Reset cooldown
    }
}
```

**Benefits**:
- System-specific state without global resources
- No synchronization needed
- Automatically initialized

### 14. ParamSet for Mutable Aliasing

Access same component mutably in different contexts.

```rust
use praxis_ecs::ParamSet;

fn complex_interaction_system(
    mut queries: ParamSet<(
        Query<&mut Health, With<Player>>,
        Query<&mut Health, With<Enemy>>,
    )>,
) {
    // First, damage all players
    for mut health in queries.p0().iter_mut() {
        health.current -= 10.0;
    }
    
    // Then, heal all enemies (without double-borrowing)
    for mut health in queries.p1().iter_mut() {
        health.current += 5.0;
    }
}

// Without ParamSet, this would fail:
// fn broken_system(
//     mut players: Query<&mut Health, With<Player>>,
//     mut enemies: Query<&mut Health, With<Enemy>>,
// ) {
//     // ERROR: Can't have two mutable queries on same component!
// }
```

### 15. Entity Relationships

Model complex entity relationships.

```rust
#[derive(Component)]
struct FollowTarget(Entity);

#[derive(Component)]
struct Owner(Entity);

#[derive(Component)]
struct Inventory {
    items: Vec<Entity>,
}

// AI follows player
let player = world.spawn((..., Player));
let enemy = world.spawn((..., Enemy, FollowTarget(player)));

fn follow_system(
    followers: Query<(&mut Transform, &FollowTarget)>,
    targets: Query<&GlobalTransform>,
) {
    for (mut transform, follow) in followers.iter_mut() {
        if let Ok(target_transform) = targets.get(follow.0) {
            let direction = target_transform.translation() - transform.translation;
            transform.translation += direction.normalize() * speed * dt;
        }
    }
}

// Projectiles remember who fired them
fn projectile_system(
    projectiles: Query<(&Transform, &Owner), With<Projectile>>,
    characters: Query<Entity, With<Character>>,
    mut commands: Commands,
) {
    for (projectile_transform, owner) in projectiles.iter() {
        for character in characters.iter() {
            if character == owner.0 {
                continue; // Don't hit self
            }
            
            if hit_check(projectile_transform, character) {
                commands.entity(character).insert(DamageEvent { 
                    amount: 25.0,
                    source: owner.0 
                });
            }
        }
    }
}
```

## Performance Best Practices

### Query Efficiency

```rust
// GOOD: Specific queries
fn render_system(query: Query<(&Transform, &MeshHandle), With<Visible>>) {
    // Iterates only visible entities
}

// BAD: Filtering in loop
fn render_system(query: Query<(&Transform, &MeshHandle, &Visible)>) {
    for (transform, mesh, visible) in query.iter() {
        if visible.0 {
            // Manual filtering is slower
        }
    }
}
```

### Component Size

```rust
// GOOD: Small components
#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Velocity(Vec3);

// BAD: Large components
#[derive(Component)]
struct CharacterData {
    stats: [f32; 100],
    inventory: Vec<Item>,
    quest_log: HashMap<u32, Quest>,
    // ... etc
}
```

**Rule of Thumb**: Keep components small (<128 bytes). Use indirection for large data:

```rust
#[derive(Component)]
struct InventoryHandle(u32);  // Index into separate inventory storage

// Store large data separately
#[derive(Resource)]
struct InventoryStorage {
    inventories: HashMap<u32, Inventory>,
}
```

### System Parallelization

```rust
// GOOD: Independent systems run in parallel automatically
schedule.add_systems((
    physics_system,      // Uses Transform, Velocity
    animation_system,    // Uses AnimationPlayer, Skeleton
    particle_system,     // Uses ParticleEmitter
    // These can run in parallel!
));

// BAD: Unnecessary .chain() prevents parallelization
schedule.add_systems((
    physics_system,
    animation_system,
    particle_system,
).chain());  // Forces sequential execution
```

Use `.chain()` only when systems have dependencies.

### Batch Operations

```rust
use praxis_ecs::Commands;

// GOOD: Batch entity modifications
fn spawn_wave_system(mut commands: Commands) {
    for i in 0..100 {
        commands.spawn((
            Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            Enemy,
            Health::default(),
        ));
    }
    // All spawns happen together after system completes
}

// BAD: Immediate modifications in loop
fn spawn_wave_system_slow(world: &mut World) {
    for i in 0..100 {
        world.spawn((
            Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
            Enemy,
            Health::default(),
        ));
        // Each spawn happens immediately (slow)
    }
}
```

## Testing Patterns

### Unit Testing Systems

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use praxis_ecs::{World, Schedule};

    #[test]
    fn test_health_system() {
        let mut world = World::new();
        let mut schedule = Schedule::default();
        schedule.add_systems(health_system);

        // Setup
        let entity = world.spawn((
            Health { current: 100.0, max: 100.0 },
            DamageEvent { amount: 30.0, source: Entity::PLACEHOLDER },
        ));

        // Run system
        schedule.run(world.inner_mut());

        // Assert
        let health = world.inner().get::<Health>(entity).unwrap();
        assert_eq!(health.current, 70.0);
        
        // Verify event was consumed
        assert!(world.inner().get::<DamageEvent>(entity).is_none());
    }
}
```

### Integration Testing

```rust
#[test]
fn test_complete_game_loop() {
    let mut world = World::new();
    let mut schedule = Schedule::default();
    
    schedule.add_systems((
        input_system,
        physics_system,
        collision_system,
        health_system,
    ).chain());

    // Setup game state
    world.insert_resource(DeltaTime(Duration::from_secs_f32(1.0/60.0)));
    
    let player = world.spawn((...));
    let enemy = world.spawn((...));

    // Simulate frames
    for _ in 0..60 {
        schedule.run(world.inner_mut());
    }

    // Verify final state
    let player_health = world.inner().get::<Health>(player).unwrap();
    assert!(player_health.current > 0.0);
}
```

## Summary

ECS patterns in Praxis enable:

1. **Composition**: Build entities from reusable components
2. **Performance**: Cache-friendly data layout
3. **Maintainability**: Clear separation of data and logic
4. **Flexibility**: Easy to extend and modify behavior
5. **Testability**: Systems are pure functions over data

**Key Takeaways**:
- Keep components small and focused
- Use queries to express intent
- Let the ECS parallelize when possible
- Profile to find bottlenecks
- Test systems in isolation

Master these patterns to build efficient, maintainable games with Praxis!
