# Getting Started with Praxis ECS

A practical guide to getting up and running with the Entity Component System in the Praxis engine.

## Quick Start (5 Minutes)

### Step 1: Create a World

```rust
use praxis_ecs::World;

let mut world = World::new();
```

### Step 2: Define Components

```rust
use praxis_ecs::Component;

#[derive(Component)]
struct Health(f32);

#[derive(Component)]
struct Speed(f32);
```

### Step 3: Spawn Entities

```rust
let entity = world.spawn((
    Health(100.0),
    Speed(5.0),
));
```

### Step 4: Create Systems

```rust
use praxis_ecs::Query;

fn movement_system(query: Query<(&Health, &Speed)>) {
    for (health, speed) in query.iter() {
        println!("Entity has {} health and {} speed", health.0, speed.0);
    }
}
```

### Step 5: Run Systems

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};

let mut schedule = Schedule::default();
schedule.add_systems(movement_system);
schedule.run(world.inner_mut());
```

That's it! You've created your first ECS application.

## Your First Game (15 Minutes)

Let's build a simple "player vs enemies" scenario with movement and combat.

### Define Game Components

```rust
use praxis_ecs::{Component, Bundle, Transform, GlobalTransform};
use praxis_math::Vec3;

// Marker components for entity types
#[derive(Component)]
struct Player;

#[derive(Component)]
struct Enemy;

// Data components
#[derive(Component)]
struct Health {
    current: f32,
    max: f32,
}

#[derive(Component)]
struct Velocity(Vec3);

#[derive(Component)]
struct Speed(f32);

// Bundle for common entity structure
#[derive(Bundle)]
struct CharacterBundle {
    transform: Transform,
    global_transform: GlobalTransform,
    health: Health,
    velocity: Velocity,
    speed: Speed,
}
```

### Create Game World

```rust
use praxis_ecs::World;

fn setup_game() -> World {
    let mut world = World::new();
    
    // Spawn player
    world.spawn((
        CharacterBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            global_transform: GlobalTransform::default(),
            health: Health { current: 100.0, max: 100.0 },
            velocity: Velocity(Vec3::ZERO),
            speed: Speed(5.0),
        },
        Player,
    ));
    
    // Spawn enemies in a circle
    for i in 0..5 {
        let angle = (i as f32) * std::f32::consts::TAU / 5.0;
        let x = angle.cos() * 10.0;
        let z = angle.sin() * 10.0;
        
        world.spawn((
            CharacterBundle {
                transform: Transform::from_xyz(x, 0.0, z),
                global_transform: GlobalTransform::default(),
                health: Health { current: 50.0, max: 50.0 },
                velocity: Velocity(Vec3::ZERO),
                speed: Speed(3.0),
            },
            Enemy,
        ));
    }
    
    world
}
```

### Write Game Systems

```rust
use praxis_ecs::{Query, Res, With, Commands, Entity};

// Resource for tracking time
#[derive(Resource)]
struct DeltaTime(f32);

// Player movement (simplified - normally you'd read input)
fn player_movement(
    mut query: Query<(&mut Velocity, &Speed), With<Player>>
) {
    for (mut velocity, speed) in query.iter_mut() {
        // Move forward
        velocity.0 = Vec3::new(0.0, 0.0, -1.0) * speed.0;
    }
}

// Enemy AI: chase player
fn enemy_ai(
    mut enemies: Query<(&Transform, &mut Velocity, &Speed), With<Enemy>>,
    player: Query<&Transform, With<Player>>,
) {
    let Ok(player_transform) = player.get_single() else {
        return;
    };
    
    for (transform, mut velocity, speed) in enemies.iter_mut() {
        // Calculate direction to player
        let direction = (player_transform.translation - transform.translation)
            .normalize_or_zero();
        velocity.0 = direction * speed.0;
    }
}

// Apply velocity to position
fn apply_velocity(
    mut query: Query<(&mut Transform, &Velocity)>,
    time: Res<DeltaTime>,
) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0 * time.0;
    }
}

// Remove dead entities
fn death_system(
    mut commands: Commands,
    query: Query<(Entity, &Health)>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

### Run the Game

```rust
use praxis_ecs::{Schedule, IntoSystemConfigs};

fn main() {
    let mut world = setup_game();
    
    // Insert time resource
    world.insert_resource(DeltaTime(0.016)); // 60 FPS
    
    // Create schedule
    let mut schedule = Schedule::default();
    schedule.add_systems((
        player_movement,
        enemy_ai,
        apply_velocity,
        death_system,
    ).chain());
    
    // Game loop (simplified - normally event-driven)
    for frame in 0..100 {
        schedule.run(world.inner_mut());
        
        // Check for game over
        let player_count = world.query_filtered::<Entity, With<Player>>()
            .iter(world.inner())
            .count();
        
        if player_count == 0 {
            println!("Game Over!");
            break;
        }
    }
}
```

## Common Patterns

### Pattern 1: Query with Filters

```rust
use praxis_ecs::{Query, With, Without};

// Only alive players
fn player_system(
    query: Query<&Transform, (With<Player>, Without<Dead>)>
) {
    for transform in query.iter() {
        // Process player
    }
}
```

### Pattern 2: Mutable vs Immutable

```rust
// Read-only (multiple systems can run in parallel)
fn render_system(query: Query<&Transform>) { }

// Mutable (exclusive access)
fn physics_system(mut query: Query<&mut Transform>) { }
```

### Pattern 3: Change Detection

```rust
use praxis_ecs::{Query, Changed, Added};

// Only process entities where Transform changed
fn update_bounds(
    query: Query<(&Transform, &mut BoundingBox), Changed<Transform>>
) { }

// Only process newly added entities
fn initialize(query: Query<Entity, Added<Transform>>) { }
```

### Pattern 4: Commands for Spawning

```rust
use praxis_ecs::Commands;

fn spawn_enemies(mut commands: Commands) {
    for i in 0..10 {
        commands.spawn((
            Transform::default(),
            Enemy,
        ));
    }
}
```

### Pattern 5: Parent-Child Hierarchy

```rust
use praxis_ecs::{Commands, Parent};

fn create_tank(commands: &mut Commands) {
    // Tank body
    let body = commands.spawn((
        Transform::from_xyz(0.0, 0.0, 0.0),
        GlobalTransform::default(),
    )).id();
    
    // Tank turret (child of body)
    commands.spawn((
        Transform::from_xyz(0.0, 1.0, 0.0),
        GlobalTransform::default(),
        Parent(body),
    ));
}
```

## Understanding System Execution

### Automatic Parallelism

Systems without conflicts run in parallel:

```rust
// These can run simultaneously
fn render_system(query: Query<&Transform>) { }
fn audio_system(query: Query<&Transform>) { }
```

### Forced Sequential Execution

Use `.chain()` for ordered execution:

```rust
schedule.add_systems((
    input_system,
    physics_system,
    render_system,
).chain());
```

## Working with Resources

Resources are global singleton data:

```rust
use praxis_ecs::{Resource, Res, ResMut};

#[derive(Resource)]
struct GameTime {
    elapsed: f32,
}

fn update_time(mut time: ResMut<GameTime>) {
    time.elapsed += 0.016;
}

fn check_time(time: Res<GameTime>) {
    println!("Elapsed: {}", time.elapsed);
}
```

## Debugging Tips

### Count Entities

```rust
let count = world.query::<Entity>().iter(world.inner()).count();
println!("Total entities: {}", count);
```

### Print Entity Components

```rust
use praxis_ecs::{Query, Name};

fn debug_entities(query: Query<(Entity, &Name, &Transform)>) {
    for (entity, name, transform) in query.iter() {
        println!("{:?}: {} at {:?}", 
            entity, 
            name.as_str(), 
            transform.translation
        );
    }
}
```

### Check for Specific Entities

```rust
let player_exists = world
    .query_filtered::<Entity, With<Player>>()
    .iter(world.inner())
    .count() > 0;
```

## Performance Tips

### 1. Use Change Detection

```rust
// Only update when needed
fn update_system(query: Query<&Transform, Changed<Transform>>) { }
```

### 2. Narrow Queries

```rust
// Good: Specific query
fn system(query: Query<(&Transform, &Health), With<Enemy>>) { }

// Bad: Overly broad query
fn system(query: Query<(&Transform, &Health, &Velocity, &Speed)>) { }
```

### 3. Batch Spawning

```rust
// Good: Batch spawn
world.spawn_batch(vec![
    (Enemy, Health(50.0)),
    (Enemy, Health(50.0)),
    (Enemy, Health(50.0)),
]);

// Bad: Individual spawns in loop
for _ in 0..3 {
    world.spawn((Enemy, Health(50.0)));
}
```

### 4. Use Filters Over Options

```rust
// Good: Filter
fn system(query: Query<&Transform, With<Visible>>) { }

// Less efficient: Option
fn system(query: Query<(&Transform, Option<&Visible>)>) { }
```

## Next Steps

Now that you understand the basics, explore:

1. **[Best Practices](./ECS_BEST_PRACTICES.md)** - Write efficient, maintainable ECS code
2. **[Query Patterns](./QUERY_PATTERNS.md)** - Master advanced queries
3. **[System Ordering](./SYSTEM_ORDERING.md)** - Control execution order
4. **[Examples](../../examples/)** - See complete working examples

## Common Questions

### Q: When should I use markers vs data components?

**A:** Use markers (zero-sized types) for categorization:
```rust
#[derive(Component)]
struct Player;  // Marker

#[derive(Component)]
struct Health(f32);  // Data
```

### Q: How do I share data between systems?

**A:** Use Resources for global state:
```rust
#[derive(Resource)]
struct GameState { /* ... */ }
```

### Q: Can I query one component type with different filters?

**A:** Yes! Use separate queries or ParamSet:
```rust
fn system(
    players: Query<&Transform, With<Player>>,
    enemies: Query<&Transform, With<Enemy>>,
) { }
```

### Q: How do I handle optional components?

**A:** Use `Option<&Component>` or separate queries:
```rust
// Option approach
fn system(query: Query<(&Transform, Option<&Health>)>) { }

// Separate queries (more efficient)
fn system(
    with_health: Query<(&Transform, &Health)>,
    without_health: Query<&Transform, Without<Health>>,
) { }
```

### Q: What's the difference between Transform and GlobalTransform?

**A:** 
- `Transform`: Local transform (relative to parent)
- `GlobalTransform`: World-space transform (computed by transform systems)

Always use both for entities in hierarchies!

## Summary

You now know:

- ✅ How to create a world and spawn entities
- ✅ How to define components and bundles
- ✅ How to write systems and queries
- ✅ How to organize system execution
- ✅ Common patterns and best practices
- ✅ Performance optimization tips

Ready to build your game! 🚀
