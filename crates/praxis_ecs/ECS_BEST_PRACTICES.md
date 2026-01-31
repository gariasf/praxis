# ECS Best Practices for Praxis Engine

This guide provides comprehensive best practices for working with the Entity Component System (ECS) in the Praxis engine. These patterns are battle-tested and designed to help you write efficient, maintainable game code.

## Table of Contents

1. [Component Design](#component-design)
2. [System Design](#system-design)
3. [Query Patterns](#query-patterns)
4. [Performance Optimization](#performance-optimization)
5. [Common Patterns](#common-patterns)
6. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
7. [Architecture Guidelines](#architecture-guidelines)

## Component Design

### Keep Components Pure Data

Components should be simple data containers without behavior. Behavior belongs in systems.

**Good:**
```rust
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

// Helper methods that don't modify state are fine
impl Health {
    fn is_alive(&self) -> bool {
        self.current > 0.0
    }
    
    fn percentage(&self) -> f32 {
        self.current / self.max
    }
}

// Behavior goes in systems
fn regeneration_system(mut query: Query<&mut Health>) {
    for mut health in query.iter_mut() {
        if health.current < health.max {
            health.current = (health.current + 1.0).min(health.max);
        }
    }
}
```

**Bad:**
```rust
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

impl Health {
    // DON'T: Behavior in component
    fn regenerate(&mut self) {
        self.current = (self.current + 1.0).min(self.max);
    }
    
    // DON'T: Side effects in component methods
    fn damage(&mut self, amount: f32, world: &mut World) {
        self.current -= amount;
        if self.current <= 0.0 {
            // Spawning particles, playing sounds, etc.
        }
    }
}
```

### Prefer Small, Focused Components

Small components enable better composition and more efficient queries.

**Good:**
```rust
#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Acceleration(Vec3);

#[derive(Component)]
struct MaxSpeed(f32);

#[derive(Component)]
struct DragCoefficient(f32);

// Systems can query exactly what they need
fn apply_acceleration(mut query: Query<(&mut Velocity, &Acceleration, &MaxSpeed)>) {
    for (mut velocity, acceleration, max_speed) in query.iter_mut() {
        velocity.0 += acceleration.0;
        velocity.0 = velocity.0.clamp_length_max(max_speed.0);
    }
}

fn apply_drag(mut query: Query<(&mut Velocity, &DragCoefficient)>) {
    for (mut velocity, drag) in query.iter_mut() {
        velocity.0 *= 1.0 - drag.0;
    }
}
```

**Bad:**
```rust
// DON'T: Monolithic component with everything
#[derive(Component)]
struct Movement {
    velocity: Vec3,
    acceleration: Vec3,
    max_speed: f32,
    drag: f32,
    jump_force: f32,
    gravity_multiplier: f32,
    ground_friction: f32,
    air_friction: f32,
    // Systems must query all of this even if they only need velocity
}
```

### Use Marker Components for Tags

Zero-sized types are perfect for categorizing entities.

```rust
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct Invulnerable;

#[derive(Component)]
struct Boss;

// Efficient queries using markers
fn player_input(query: Query<&mut Transform, With<Player>>) { }

fn damage_enemies(
    mut enemies: Query<&mut Health, (With<Enemy>, Without<Invulnerable>)>
) { }

fn boss_ai(query: Query<(&Transform, &Health), (With<Boss>, With<Enemy>)>) { }
```

### Make Components Serializable

Always implement serialization for components that should persist.

```rust
use serde::{Serialize, Deserialize};

#[derive(Component, Serialize, Deserialize)]
struct PlayerStats {
    level: u32,
    experience: u64,
    score: u32,
}

// Register for serialization
registry.register::<PlayerStats>();
```

Components that should NOT be saved (runtime-only):

```rust
#[derive(Component)]
struct DebugGizmo;  // Don't serialize

// Or mark with NoSave
world.spawn((
    Transform::default(),
    DebugGizmo,
    NoSave,  // Explicitly mark as temporary
));
```

### Avoid References to Other Entities in Components

Use Entity IDs instead of storing references.

**Good:**
```rust
#[derive(Component)]
struct Target(Entity);  // Store entity ID

#[derive(Component)]
struct Owner(Entity);

fn target_system(
    query: Query<(&Target, &Transform)>,
    targets: Query<&Transform>,
) {
    for (target, transform) in query.iter() {
        if let Ok(target_transform) = targets.get(target.0) {
            // Use target_transform
        }
    }
}
```

**Bad:**
```rust
// DON'T: Store references
#[derive(Component)]
struct Target {
    entity: Entity,
    transform: &Transform,  // Lifetime issues!
}
```

## System Design

### One Responsibility per System

Systems should do one thing well.

**Good:**
```rust
fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>, time: Res<DeltaTime>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.0;
    }
}

fn apply_gravity(mut query: Query<&mut Velocity, With<GravityAffected>>) {
    for mut velocity in query.iter_mut() {
        velocity.0.y -= 9.8 * 0.016;
    }
}

fn handle_collisions(
    query: Query<(Entity, &Transform, &Collider)>,
    mut velocities: Query<&mut Velocity>,
) {
    // Handle collisions
}
```

**Bad:**
```rust
// DON'T: Kitchen sink system
fn physics_system(
    mut query: Query<(Entity, &mut Transform, &mut Velocity, &Collider)>,
    time: Res<DeltaTime>,
) {
    // Apply gravity
    // Apply velocity
    // Handle collisions
    // Apply friction
    // Resolve constraints
    // Update spatial partitioning
    // Too many responsibilities!
}
```

### Use Commands for Structural Changes

Never spawn/despawn entities or add/remove components during iteration.

**Good:**
```rust
fn spawn_projectiles(
    mut commands: Commands,
    query: Query<&Transform, (With<Player>, With<Shooting>)>,
) {
    for transform in query.iter() {
        commands.spawn((
            Transform::from_translation(transform.translation),
            Projectile,
            Velocity(transform.forward() * 10.0),
        ));
    }
}
```

**Bad:**
```rust
// DON'T: Direct world mutation during iteration
fn spawn_projectiles(
    world: &mut World,
    query: Query<&Transform, (With<Player>, With<Shooting>)>,
) {
    for transform in query.iter() {
        world.spawn((  // WRONG: Mutating during iteration
            Transform::from_translation(transform.translation),
            Projectile,
        ));
    }
}
```

### Prefer Narrow System Parameters

Only access what you need to enable parallelism.

**Good:**
```rust
// Can run in parallel with other systems
fn render_system(
    query: Query<(&Transform, &MeshHandle)>,
    cameras: Query<&CameraMatrices>,
) { }

fn audio_system(
    query: Query<(&Transform, &AudioSource)>,
    listener: Query<&Transform, With<AudioListener>>,
) { }
```

**Bad:**
```rust
// DON'T: Blocks all parallel execution
fn render_system(world: &mut World) {
    // Do rendering
}
```

### Order Systems Explicitly When Dependencies Exist

```rust
schedule.add_systems((
    input_system,
    physics_system,
    animation_system,
    rendering_system,
).chain());  // Explicit ordering

// Or use before/after
schedule.add_systems(
    collision_system
        .before(physics_system)
        .after(movement_system)
);
```

## Query Patterns

### Use Filters Effectively

Filters improve performance by narrowing the search space.

```rust
use praxis_ecs::{With, Without, Changed, Added, Or};

// Only active enemies
fn ai_system(query: Query<&Transform, (With<Enemy>, With<Active>)>) { }

// Entities without Dead marker
fn update_living(query: Query<&mut Health, Without<Dead>>) { }

// Only entities where Transform changed
fn update_bounds(query: Query<(&Transform, &mut BoundingBox), Changed<Transform>>) { }

// Only newly added entities
fn initialize_entities(query: Query<Entity, Added<Transform>>) { }

// Either Player or Enemy
fn combatants(query: Query<&Transform, Or<(With<Player>, With<Enemy>)>>) { }
```

### Avoid Option in Queries When Possible

Optional components in queries are less efficient than filters.

**Good:**
```rust
// Separate queries for different cases
fn render_visible(query: Query<(&Transform, &MeshHandle), With<Visible>>) { }
fn render_invisible(query: Query<(&Transform, &MeshHandle), Without<Visible>>) { }
```

**Less Efficient:**
```rust
fn render_all(query: Query<(&Transform, &MeshHandle, Option<&Visible>)>) {
    for (transform, mesh, visible) in query.iter() {
        if visible.is_some() {
            // Render
        }
    }
}
```

### Use ParamSet for Conflicting Queries

When you need multiple mutable queries to the same components:

```rust
fn parent_child_system(
    mut queries: ParamSet<(
        Query<&Transform>,                    // Read all
        Query<(&Parent, &mut Transform)>,     // Write children
    )>,
) {
    // Collect parent data
    let parent_transforms: Vec<_> = queries.p0()
        .iter()
        .map(|t| t.clone())
        .collect();
    
    // Update children
    for (parent, mut transform) in queries.p1().iter_mut() {
        // Use parent data to update child
    }
}
```

### Batch Operations

Collect entity IDs first, then operate on them:

```rust
fn cleanup_system(
    mut commands: Commands,
    query: Query<Entity, With<Dead>>,
) {
    let dead_entities: Vec<Entity> = query.iter().collect();
    
    for entity in dead_entities {
        commands.entity(entity).despawn();
    }
}
```

## Performance Optimization

### Use Change Detection

Only process entities that actually changed:

```rust
fn update_transform_matrices(
    mut query: Query<(&Transform, &mut GlobalTransform), Changed<Transform>>
) {
    for (transform, mut global) in query.iter_mut() {
        global.matrix = transform.compute_matrix();
    }
}
```

### Batch Spawning

Spawn many entities at once for better performance:

```rust
// Good: Batch spawn
let enemies = (0..100)
    .map(|i| (
        Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
        Enemy,
        Health { current: 100.0, max: 100.0 },
    ))
    .collect::<Vec<_>>();

world.spawn_batch(enemies);

// Bad: Individual spawns
for i in 0..100 {
    world.spawn((
        Transform::from_xyz(i as f32 * 2.0, 0.0, 0.0),
        Enemy,
        Health { current: 100.0, max: 100.0 },
    ));
}
```

### Prefer Queries to Resource Lookups

Resources have exclusive access requirements:

```rust
// Good: Query for component data (allows parallelism)
fn render_system(
    meshes: Query<(&Transform, &MeshHandle)>,
    cameras: Query<&CameraMatrices>,
) { }

// Less efficient: Accessing large resource
fn render_system(
    mesh_registry: Res<MeshRegistry>,  // Blocks other systems
) { }
```

### Use Events for Rare Communications

Don't poll every frame for rare events:

```rust
#[derive(Event)]
struct EnemyDiedEvent {
    entity: Entity,
    position: Vec3,
}

fn damage_system(
    mut events: EventWriter<EnemyDiedEvent>,
    mut query: Query<(Entity, &Transform, &mut Health)>,
) {
    for (entity, transform, mut health) in query.iter_mut() {
        if health.current <= 0.0 {
            events.send(EnemyDiedEvent {
                entity,
                position: transform.translation,
            });
        }
    }
}

fn spawn_death_effects(
    mut commands: Commands,
    mut events: EventReader<EnemyDiedEvent>,
) {
    for event in events.read() {
        commands.spawn((
            Transform::from_translation(event.position),
            ParticleEmitter,
        ));
    }
}
```

### Clear Detection Flags Appropriately

```rust
// Manually clear change detection when needed
world.clear_trackers();
```

## Common Patterns

### State Machine Pattern

```rust
#[derive(Component)]
enum EnemyState {
    Idle,
    Patrolling { waypoint: usize },
    Chasing { target: Entity },
    Attacking { target: Entity, cooldown: f32 },
}

fn enemy_ai_system(
    mut query: Query<(Entity, &mut EnemyState, &Transform)>,
    player: Query<&Transform, With<Player>>,
) {
    for (entity, mut state, transform) in query.iter_mut() {
        *state = match *state {
            EnemyState::Idle => {
                // Transition logic
                EnemyState::Patrolling { waypoint: 0 }
            }
            EnemyState::Patrolling { waypoint } => {
                // Patrol logic
                EnemyState::Patrolling { waypoint }
            }
            EnemyState::Chasing { target } => {
                // Chase logic
                EnemyState::Chasing { target }
            }
            EnemyState::Attacking { target, cooldown } => {
                // Attack logic
                EnemyState::Attacking { target, cooldown: cooldown - 0.016 }
            }
        };
    }
}
```

### Timer Pattern

```rust
#[derive(Component)]
struct Cooldown {
    remaining: f32,
    duration: f32,
}

impl Cooldown {
    fn new(duration: f32) -> Self {
        Self { remaining: 0.0, duration }
    }
    
    fn is_ready(&self) -> bool {
        self.remaining <= 0.0
    }
    
    fn reset(&mut self) {
        self.remaining = self.duration;
    }
}

fn update_cooldowns(mut query: Query<&mut Cooldown>, time: Res<DeltaTime>) {
    for mut cooldown in query.iter_mut() {
        cooldown.remaining = (cooldown.remaining - time.0).max(0.0);
    }
}

fn shoot_system(
    mut query: Query<(&mut Cooldown, &Transform), With<CanShoot>>,
    mut commands: Commands,
) {
    for (mut cooldown, transform) in query.iter_mut() {
        if cooldown.is_ready() {
            commands.spawn(Projectile::at(transform.translation));
            cooldown.reset();
        }
    }
}
```

### Reference Counting Pattern

```rust
#[derive(Component)]
struct ReferenceCounted {
    count: usize,
}

#[derive(Component)]
struct Reference {
    target: Entity,
}

fn cleanup_unused(
    mut commands: Commands,
    query: Query<(Entity, &ReferenceCounted)>,
) {
    for (entity, ref_count) in query.iter() {
        if ref_count.count == 0 {
            commands.entity(entity).despawn();
        }
    }
}
```

## Anti-Patterns to Avoid

### 1. God Components

**Bad:**
```rust
// DON'T: One component with everything
#[derive(Component)]
struct Player {
    position: Vec3,
    rotation: Quat,
    health: f32,
    stamina: f32,
    inventory: Vec<Item>,
    stats: PlayerStats,
    input: InputState,
    animation: AnimationState,
    // Everything!
}
```

**Good:**
```rust
// DO: Separate concerns
#[derive(Component)] struct Player;
#[derive(Component)] struct Transform { ... }
#[derive(Component)] struct Health(f32);
#[derive(Component)] struct Stamina(f32);
#[derive(Component)] struct Inventory(Vec<Item>);
#[derive(Component)] struct PlayerStats { ... }
```

### 2. Storing World/Entity References in Components

**Bad:**
```rust
// DON'T: Store references
#[derive(Component)]
struct Weapon {
    world: &World,  // Can't do this!
    owner: &Entity,  // Can't do this!
}
```

**Good:**
```rust
// DO: Store entity IDs
#[derive(Component)]
struct Weapon {
    owner: Entity,
}
```

### 3. Over-Using Exclusive World Access

**Bad:**
```rust
// DON'T: Blocks all parallelism
fn update_everything(world: &mut World) {
    // Do all updates
}
```

**Good:**
```rust
// DO: Specific system parameters
fn update_physics(mut query: Query<(&mut Transform, &Velocity)>) { }
fn update_animations(mut query: Query<(&mut AnimationState, &Transform)>) { }
// Can run in parallel!
```

### 4. Forgetting Transform Propagation

**Bad:**
```rust
// DON'T: Manual transform updates
fn update_child_transforms(
    mut children: Query<(&Parent, &mut Transform)>,
    parents: Query<&Transform>,
) {
    // Manual propagation - error-prone
}
```

**Good:**
```rust
// DO: Use built-in transform systems
schedule.add_systems((
    sync_parent_child_relationships,
    propagate_transforms,
).chain());
```

### 5. Circular Parent References

**Bad:**
```rust
// DON'T: Create cycles
let a = world.spawn(Parent(b));
let b = world.spawn(Parent(a));  // Cycle!
```

### 6. Modifying During Iteration

**Bad:**
```rust
// DON'T: Structural changes during iteration
fn bad_spawn(world: &mut World) {
    for entity in world.query::<Entity>().iter(world) {
        world.spawn(Enemy);  // WRONG!
    }
}
```

**Good:**
```rust
// DO: Use Commands
fn good_spawn(mut commands: Commands, query: Query<Entity>) {
    for entity in query.iter() {
        commands.spawn(Enemy);  // Deferred
    }
}
```

## Architecture Guidelines

### Organize by Feature, Not Type

**Good structure:**
```
src/
  player/
    components.rs  (Player, PlayerInput, PlayerStats)
    systems.rs     (player_movement, player_input, player_stats_ui)
  enemy/
    components.rs  (Enemy, EnemyAI, EnemyStats)
    systems.rs     (enemy_ai, enemy_spawn, enemy_patrol)
  weapons/
    components.rs  (Weapon, Projectile, WeaponStats)
    systems.rs     (shoot_weapon, update_projectiles)
```

**Bad structure:**
```
src/
  components.rs  (all components mixed together)
  systems.rs     (all systems mixed together)
```

### Use Bundles for Common Entity Types

```rust
#[derive(Bundle)]
struct CharacterBundle {
    name: Name,
    transform: Transform,
    global_transform: GlobalTransform,
    health: Health,
    velocity: Velocity,
}

#[derive(Bundle)]
struct EnemyBundle {
    character: CharacterBundle,
    enemy: Enemy,
    ai_state: EnemyState,
}
```

### Document System Ordering Requirements

```rust
/// Updates entity positions based on velocity.
/// 
/// **System Order:**
/// - Must run AFTER physics systems (which update velocity)
/// - Must run BEFORE rendering systems (which read transform)
fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<DeltaTime>,
) { }
```

### Use Type Aliases for Complex Queries

```rust
type PlayerQuery<'a> = Query<'a, 'a, (
    &'static Transform,
    &'static mut Health,
    &'static Velocity,
), With<Player>>;

fn player_system(mut players: PlayerQuery) {
    for (transform, mut health, velocity) in players.iter_mut() {
        // ...
    }
}
```

## Testing ECS Code

### Test Systems in Isolation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_regeneration_system() {
        let mut world = World::new();
        
        // Setup
        let entity = world.spawn(Health { current: 50.0, max: 100.0 });
        
        // Run system
        let mut schedule = Schedule::default();
        schedule.add_systems(regeneration_system);
        schedule.run(world.inner_mut());
        
        // Assert
        let health = world.get::<Health>(entity).unwrap();
        assert!(health.current > 50.0);
    }
}
```

### Test Component Behavior

```rust
#[test]
fn test_health_component() {
    let health = Health { current: 100.0, max: 100.0 };
    assert!(health.is_alive());
    assert_eq!(health.percentage(), 1.0);
}
```

## Summary

**Key Takeaways:**

1. **Components are data** - Keep them simple and pure
2. **Systems are behavior** - One responsibility per system
3. **Use filters** - Narrow queries for better performance
4. **Commands for structure** - Defer spawning/despawning
5. **Change detection** - Only process what changed
6. **Composition** - Small, focused components compose better
7. **Parallelism** - Narrow system parameters enable it
8. **Test systems** - Systems are just functions

Following these practices will help you build maintainable, performant games with the Praxis ECS.
