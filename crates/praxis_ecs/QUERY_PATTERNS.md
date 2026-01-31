# ECS Query Patterns Guide

Comprehensive guide to query patterns in the Praxis ECS, covering common use cases and advanced techniques.

## Table of Contents

1. [Basic Queries](#basic-queries)
2. [Query Filters](#query-filters)
3. [Mutable vs Immutable Access](#mutable-vs-immutable-access)
4. [Advanced Patterns](#advanced-patterns)
5. [Performance Considerations](#performance-considerations)
6. [Common Pitfalls](#common-pitfalls)

## Basic Queries

### Simple Component Query

Query entities with specific components:

```rust
use praxis_ecs::{Query, Transform};

fn render_system(query: Query<&Transform>) {
    for transform in query.iter() {
        // Process each transform
        println!("Position: {:?}", transform.translation);
    }
}
```

### Multiple Components

Query entities that have all specified components:

```rust
use praxis_ecs::{Query, Transform, Velocity};

fn movement_system(query: Query<(&Transform, &Velocity)>) {
    for (transform, velocity) in query.iter() {
        // Entity must have both Transform AND Velocity
        println!("Moving from {:?} with {:?}", transform.translation, velocity.0);
    }
}
```

### Mutable Components

Use `&mut` for components you need to modify:

```rust
use praxis_ecs::{Query, Transform, Velocity};

fn apply_velocity(mut query: Query<(&mut Transform, &Velocity)>) {
    for (mut transform, velocity) in query.iter_mut() {
        transform.translation += velocity.0;
    }
}
```

### Entity IDs

Include `Entity` to get the entity identifier:

```rust
use praxis_ecs::{Query, Entity, Transform};

fn list_entities(query: Query<(Entity, &Transform)>) {
    for (entity, transform) in query.iter() {
        println!("Entity {:?} at {:?}", entity, transform.translation);
    }
}
```

## Query Filters

Filters refine which entities match without accessing component data.

### With Filter

Include only entities that have a specific component:

```rust
use praxis_ecs::{Query, Transform, With, Player};

// Only entities with BOTH Transform and Player
fn player_system(query: Query<&Transform, With<Player>>) {
    for transform in query.iter() {
        // This is definitely a player
    }
}
```

### Without Filter

Exclude entities that have a specific component:

```rust
use praxis_ecs::{Query, Transform, Without, Dead};

// Only entities with Transform but WITHOUT Dead
fn update_living(query: Query<&Transform, Without<Dead>>) {
    for transform in query.iter() {
        // Only living entities
    }
}
```

### Changed Filter

Only entities where the component was modified:

```rust
use praxis_ecs::{Query, Transform, Changed};

// Only entities where Transform changed this frame
fn update_dirty(query: Query<&Transform, Changed<Transform>>) {
    for transform in query.iter() {
        // Transform was modified since last frame
        println!("Transform changed: {:?}", transform.translation);
    }
}
```

### Added Filter

Only entities where the component was just added:

```rust
use praxis_ecs::{Query, Transform, Added};

// Only entities where Transform was just added
fn initialize_new(query: Query<(Entity, &Transform), Added<Transform>>) {
    for (entity, transform) in query.iter() {
        println!("New transform on {:?}", entity);
    }
}
```

### Or Filter

Match entities with any of the specified components:

```rust
use praxis_ecs::{Query, Transform, Or, With, Player, Enemy};

// Entities with Transform AND (Player OR Enemy)
fn combatants(query: Query<&Transform, Or<(With<Player>, With<Enemy>)>>) {
    for transform in query.iter() {
        // Either a player or an enemy
    }
}
```

### Combining Filters

Chain multiple filters together:

```rust
use praxis_ecs::{Query, Transform, With, Without, Changed, Player, Dead, Frozen};

// Complex filter: Players that are alive, not frozen, and just moved
fn active_players(
    query: Query<
        &Transform,
        (
            With<Player>,
            Without<Dead>,
            Without<Frozen>,
            Changed<Transform>
        )
    >
) {
    for transform in query.iter() {
        // Very specific subset of entities
    }
}
```

## Mutable vs Immutable Access

### Read-Only Queries

Multiple systems can read the same components in parallel:

```rust
// System 1 - reads Transform
fn render_system(query: Query<&Transform>) { }

// System 2 - also reads Transform (can run in parallel)
fn physics_debug(query: Query<&Transform>) { }

// Both systems can run simultaneously
```

### Mutable Queries

Only ONE system can have mutable access at a time:

```rust
// System 1 - mutates Transform
fn movement_system(mut query: Query<&mut Transform>) { }

// System 2 - also mutates Transform (CANNOT run in parallel)
fn physics_system(mut query: Query<&mut Transform>) { }

// Scheduler ensures these run sequentially
```

### Mixed Access

Systems can mix read and write access:

```rust
use praxis_ecs::{Query, Transform, Velocity, Health};

// Reads Velocity, writes Transform
fn apply_velocity(mut query: Query<(&Velocity, &mut Transform)>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0;
    }
}

// Reads Transform, writes Health
fn fall_damage(mut query: Query<(&Transform, &mut Health)>) {
    for (transform, mut health) in query.iter_mut() {
        if transform.translation.y < -100.0 {
            health.current = 0.0;
        }
    }
}

// These systems don't conflict - can run in parallel
```

## Advanced Patterns

### Optional Components

Use `Option<&T>` for components that might not exist:

```rust
use praxis_ecs::{Query, Transform, Health, Shield};

fn render_health_bars(query: Query<(&Transform, &Health, Option<&Shield>)>) {
    for (transform, health, shield) in query.iter() {
        // All entities have Transform and Health
        // Some might have Shield
        if let Some(shield) = shield {
            println!("Health: {}, Shield: {}", health.current, shield.0);
        } else {
            println!("Health: {} (no shield)", health.current);
        }
    }
}
```

**Note:** Using `Option` is less efficient than separate queries with filters.

### ParamSet for Conflicting Queries

Access the same component type multiple ways:

```rust
use praxis_ecs::{ParamSet, Query, Entity, Transform, Parent};

fn hierarchy_system(
    mut queries: ParamSet<(
        Query<&Transform>,                    // Read all transforms
        Query<(&Parent, &mut Transform)>,     // Write child transforms
    )>,
) {
    // Phase 1: Read parent transforms
    let parent_data: Vec<_> = queries.p0()
        .iter()
        .map(|t| t.clone())
        .collect();
    
    // Phase 2: Update child transforms
    for (parent, mut transform) in queries.p1().iter_mut() {
        // Use parent_data to update child
    }
}
```

### Nested Queries

Query within a query (use sparingly):

```rust
use praxis_ecs::{Query, Entity, Transform, Target};

fn targeting_system(
    attackers: Query<(Entity, &Transform, &Target)>,
    mut targets: Query<&mut Health>,
) {
    for (entity, transform, target) in attackers.iter() {
        if let Ok(mut health) = targets.get_mut(target.0) {
            // Apply damage to target
            health.current -= 10.0;
        }
    }
}
```

### QueryState for Dynamic Queries

Create queries dynamically:

```rust
use praxis_ecs::{World, QueryState, Transform};

fn dynamic_query(world: &mut World) {
    let mut query: QueryState<&Transform> = world.query::<&Transform>();
    
    for transform in query.iter(world.inner()) {
        println!("Position: {:?}", transform.translation);
    }
}
```

### Get Single Entity

Query for a single specific entity:

```rust
use praxis_ecs::{Query, Entity, Transform};

fn update_player(
    mut query: Query<&mut Transform>,
    player: Entity,
) {
    if let Ok(mut transform) = query.get_mut(player) {
        transform.translation.y += 1.0;
    }
}
```

### Single Component Query

When you know there's only one:

```rust
use praxis_ecs::{Query, Camera, CameraMatrices, With};

fn get_main_camera(query: Query<&CameraMatrices, With<Camera>>) {
    if let Ok(matrices) = query.get_single() {
        // Use the one camera
    } else {
        // No camera or multiple cameras
    }
}
```

### Iter vs Iter_mut

```rust
use praxis_ecs::{Query, Transform, Velocity};

fn example_system(mut query: Query<(&Transform, &mut Velocity)>) {
    // Use iter_mut when you have mutable components
    for (transform, mut velocity) in query.iter_mut() {
        velocity.0 += transform.translation;
    }
    
    // Use iter when all components are immutable
    // query.iter() - but this won't compile since Velocity is &mut
}

fn read_only_system(query: Query<(&Transform, &Velocity)>) {
    // All immutable - use iter()
    for (transform, velocity) in query.iter() {
        println!("{:?} moving at {:?}", transform.translation, velocity.0);
    }
}
```

## Performance Considerations

### Filter Before Optional

Filters are more efficient than Optional:

**Good:**
```rust
// Two separate queries
fn render_shields(query: Query<(&Transform, &Shield), With<HasShield>>) { }
fn render_no_shields(query: Query<&Transform, Without<HasShield>>) { }
```

**Less Efficient:**
```rust
fn render_all(query: Query<(&Transform, Option<&Shield>)>) {
    for (transform, shield) in query.iter() {
        if let Some(shield) = shield {
            // Render with shield
        } else {
            // Render without shield
        }
    }
}
```

### Narrow Query Scope

Only query what you need:

**Good:**
```rust
fn render_system(query: Query<(&Transform, &MeshHandle), With<Visible>>) {
    // Only processes visible meshes
}
```

**Bad:**
```rust
fn render_system(query: Query<(&Transform, &MeshHandle, &Health, &Velocity)>) {
    // Unnecessarily queries Health and Velocity
}
```

### Use Change Detection

Only process modified entities:

```rust
// Good: Only processes changed entities
fn update_bounds(
    mut query: Query<(&Transform, &mut BoundingBox), Changed<Transform>>
) { }

// Bad: Processes all entities every frame
fn update_bounds(mut query: Query<(&Transform, &mut BoundingBox)>) { }
```

### Batch Collection

Collect entities first for batch operations:

```rust
fn despawn_dead(
    mut commands: Commands,
    query: Query<Entity, With<Dead>>,
) {
    // Collect all dead entities
    let dead: Vec<Entity> = query.iter().collect();
    
    // Batch despawn
    for entity in dead {
        commands.entity(entity).despawn();
    }
}
```

## Common Pitfalls

### 1. Mutable Aliasing

**Wrong:**
```rust
// DON'T: Can't have two mutable queries to same component
fn bad_system(
    mut query1: Query<&mut Transform>,
    mut query2: Query<&mut Transform>,  // ERROR!
) { }
```

**Right:**
```rust
// DO: Use ParamSet
fn good_system(
    mut queries: ParamSet<(
        Query<&mut Transform, With<Player>>,
        Query<&mut Transform, With<Enemy>>,
    )>,
) { }
```

### 2. Forgetting iter_mut

**Wrong:**
```rust
fn bad_system(mut query: Query<&mut Transform>) {
    for transform in query.iter() {  // WRONG: iter() not iter_mut()
        // Can't mutate!
    }
}
```

**Right:**
```rust
fn good_system(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {  // Correct
        transform.translation.x += 1.0;
    }
}
```

### 3. Querying During Mutation

**Wrong:**
```rust
fn bad_system(world: &mut World, query: Query<Entity>) {
    for entity in query.iter() {
        world.spawn(Enemy);  // WRONG: Mutating during iteration
    }
}
```

**Right:**
```rust
fn good_system(mut commands: Commands, query: Query<Entity>) {
    for entity in query.iter() {
        commands.spawn(Enemy);  // Deferred
    }
}
```

### 4. Over-Using Options

**Less Efficient:**
```rust
fn render(query: Query<(&Transform, Option<&MeshHandle>, Option<&Material>)>) {
    // Queries many archetypes
}
```

**More Efficient:**
```rust
fn render_meshes(query: Query<(&Transform, &MeshHandle), With<Material>>) {
    // Specific archetype
}
```

### 5. Not Using Filters

**Bad:**
```rust
fn update_enemies(query: Query<(&Transform, &Enemy, &Health)>) {
    for (transform, enemy, health) in query.iter() {
        if health.current > 0.0 {  // Manual filtering
            // Update
        }
    }
}
```

**Good:**
```rust
fn update_enemies(
    query: Query<(&Transform, &Enemy, &Health), Without<Dead>>
) {
    for (transform, enemy, health) in query.iter() {
        // Automatically filtered
    }
}
```

## Query Cheat Sheet

| Pattern | Use Case |
|---------|----------|
| `Query<&T>` | Read component |
| `Query<&mut T>` | Modify component |
| `Query<(Entity, &T)>` | Get entity ID + component |
| `Query<&T, With<U>>` | Only entities with U |
| `Query<&T, Without<U>>` | Only entities without U |
| `Query<&T, Changed<T>>` | Only modified entities |
| `Query<&T, Added<T>>` | Only newly added |
| `Query<&T, Or<(With<A>, With<B>)>>` | With A or B |
| `Query<(&T, Option<&U>)>` | Optional component |
| `ParamSet<(Query<&T>, Query<&mut T>)>` | Conflicting queries |

## Summary

- **Use filters** for better performance than Optional
- **Narrow queries** to only what you need
- **Change detection** avoids unnecessary work
- **ParamSet** resolves conflicting queries
- **iter_mut()** for mutable components
- **Commands** for structural changes

Mastering these query patterns will help you write efficient, maintainable ECS code.
