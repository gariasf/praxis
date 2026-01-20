# Spatial Partitioning Quick Reference

## Quick Start

```rust
use praxis_spatial::{SpatialManager, SpatialConfig, Aabb};
use praxis_math::Vec3;
use bevy_ecs::entity::Entity;

// Create manager
let mut manager = SpatialManager::new_octree(SpatialConfig::default());

// Add entity
let entity = Entity::from_raw(1);
let bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(1.0));
manager.insert(entity, bounds);

// Query
let nearby = manager.query_radius(Vec3::ZERO, 50.0);
```

## Common Operations

### Setup

```rust
// Octree (uniform distribution)
let manager = SpatialManager::new_octree(SpatialConfig::default());

// BVH (ray tracing, dynamic)
let manager = SpatialManager::new_bvh(SpatialConfig::default());

// Custom config
let config = SpatialConfig {
    center: Vec3::ZERO,
    size: 1000.0,
    max_entities_per_node: 8,
    movement_threshold: 0.5,
    rebalance_interval: 100,
};
```

### Entity Management

```rust
// Insert
manager.insert(entity, bounds);

// Update (with threshold)
manager.update(entity, new_bounds);

// Force update (ignore threshold)
manager.force_update(entity, new_bounds);

// Remove
manager.remove(entity);

// Check if contains
if manager.contains(entity) { }

// Get bounds
if let Some(bounds) = manager.get_bounds(entity) { }
```

### Queries

```rust
// Bounding box
let bounds = Aabb::from_center_half_extents(pos, Vec3::splat(size));
let results = manager.query(&bounds);

// Radius
let results = manager.query_radius(position, radius);

// Ray (unsorted)
let results = manager.query_ray(origin, direction, max_distance);

// Ray (sorted by distance)
let results = manager.query_ray_sorted(origin, direction, max_distance);
for (entity, distance) in results {
    println!("{:?} at {:.2}", entity, distance);
}

// Nearest
if let Some((entity, distance)) = manager.query_nearest(pos, max_dist) {
    println!("Nearest: {:?}", entity);
}

// K nearest
let nearest = manager.query_k_nearest(pos, 5, max_dist);

// Frustum
let visible = manager.query_frustum(&frustum);
```

### Maintenance

```rust
// Process pending updates
manager.flush_updates();

// Check if needs rebalancing
if manager.needs_rebalancing() {
    manager.rebalance_if_needed();
}

// Force rebuild
manager.rebuild();

// Get statistics
let stats = manager.stats();
println!("Entities: {}, Dirty: {}", 
    stats.entity_count, stats.dirty_count);

// Clear all
manager.clear();
```

## ECS Integration

### Setup

```rust
use praxis_ecs::{World, Schedule};
use praxis_spatial::*;

let mut world = World::new();

// Add resource
world.insert_resource(SpatialResource::new_octree(SpatialConfig::default()));

// Add systems
let mut schedule = Schedule::default();
schedule.add_systems((
    insert_spatial_entities,
    update_spatial_entities,
    remove_spatial_entities,
    flush_spatial_updates,
    auto_rebalance_spatial,
).chain());
```

### Spawning Entities

```rust
// With bundle
world.spawn(SpatialBundle::from_center_half_extents(
    Vec3::new(10.0, 0.0, 0.0),
    Vec3::splat(2.0),
));

// With individual components
world.spawn((
    SpatialEntity::enabled(),
    SpatialBounds::from_min_max(Vec3::ZERO, Vec3::ONE),
));

// Disabled initially
world.spawn((
    SpatialEntity::disabled(),
    SpatialBounds::from_center_half_extents(pos, size),
));
```

### Querying in Systems

```rust
fn my_system(spatial: Res<SpatialResource>) {
    let results = spatial.manager.query_radius(Vec3::ZERO, 100.0);
    for entity in results {
        // Process entity
    }
}
```

### Updating Bounds

```rust
fn update_system(
    mut query: Query<(&Transform, &mut SpatialBounds)>
) {
    for (transform, mut bounds) in query.iter_mut() {
        let pos = transform.translation;
        bounds.aabb = Aabb::from_center_half_extents(pos, Vec3::splat(2.0));
        // System automatically detects change and updates spatial structure
    }
}
```

## Ray Queries

### Basic Ray Cast

```rust
let origin = Vec3::new(0.0, 1.0, 0.0);
let direction = Vec3::new(1.0, 0.0, 0.0).normalize();
let max_distance = 100.0;

let hits = manager.query_ray(origin, direction, max_distance);
```

### Sorted Ray Cast

```rust
let hits = manager.query_ray_sorted(origin, direction, max_distance);

// First hit
if let Some((entity, distance)) = hits.first() {
    println!("Hit {:?} at {:.2} units", entity, distance);
}

// All hits
for (entity, distance) in hits {
    println!("{:?}: {:.2}", entity, distance);
}
```

### Direct AABB Ray Test

```rust
let aabb = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);

// Boolean test
if aabb.intersects_ray(origin, direction, max_distance) {
    println!("Ray hits box");
}

// Get distance
if let Some(distance) = aabb.ray_intersection_distance(origin, direction, max_distance) {
    println!("Hit at distance: {:.2}", distance);
}
```

## Common Patterns

### Player Proximity

```rust
fn find_nearby_enemies(
    spatial: &SpatialResource,
    player_pos: Vec3,
) -> Vec<Entity> {
    spatial.manager.query_radius(player_pos, 50.0)
        .into_iter()
        .filter(|&e| is_enemy(e))
        .collect()
}
```

### Projectile Hit Detection

```rust
fn check_projectile_hit(
    spatial: &SpatialResource,
    start: Vec3,
    direction: Vec3,
) -> Option<(Entity, f32)> {
    let hits = spatial.manager.query_ray_sorted(start, direction, 1000.0);
    hits.first().copied()
}
```

### Area Damage

```rust
fn explosion_damage(
    spatial: &SpatialResource,
    center: Vec3,
    radius: f32,
) {
    let affected = spatial.manager.query_radius(center, radius);
    for entity in affected {
        apply_damage(entity, calculate_damage(entity, center, radius));
    }
}
```

### View Frustum Culling

```rust
fn cull_objects(
    spatial: &SpatialResource,
    camera_matrices: &CameraMatrices,
) -> Vec<Entity> {
    let frustum = Frustum::from_view_projection(camera_matrices.view_projection);
    spatial.manager.query_frustum(&frustum)
}
```

### Nearest Enemy

```rust
fn find_closest_enemy(
    spatial: &SpatialResource,
    position: Vec3,
) -> Option<Entity> {
    spatial.manager
        .query_nearest(position, 200.0)
        .map(|(entity, _)| entity)
        .filter(|&e| is_enemy(e))
}
```

## Performance Tips

1. **Tune threshold**: Higher = fewer updates, lower = more accuracy
   ```rust
   manager.set_movement_threshold(1.0); // Adjust based on game scale
   ```

2. **Batch updates**: Use `flush_updates()` once per frame
   ```rust
   manager.flush_updates(); // After all entity updates
   ```

3. **Rebalance wisely**: Don't rebalance every frame
   ```rust
   manager.set_rebalance_interval(100); // Every 100 updates
   ```

4. **Choose right structure**:
   - Octree: Uniform distribution, simple queries
   - BVH: Ray tracing, clustered objects

5. **Profile first**: Measure before optimizing
   ```rust
   let stats = manager.stats();
   println!("Update ratio: {:.2}%", 
       stats.dirty_count as f32 / stats.entity_count as f32 * 100.0);
   ```

## Troubleshooting

**Entities not found in queries:**
```rust
// Check if entity is in structure
assert!(manager.contains(entity));

// Verify bounds are correct
if let Some(bounds) = manager.get_bounds(entity) {
    println!("Bounds: {:?}", bounds);
}
```

**Too many updates:**
```rust
// Increase threshold
manager.set_movement_threshold(2.0);

// Check stats
let stats = manager.stats();
if stats.dirty_count > stats.entity_count / 2 {
    println!("Warning: High update rate");
}
```

**Poor performance:**
```rust
// Reduce rebalancing
manager.set_rebalance_interval(500);

// Consider switching structure type
let manager = SpatialManager::new_bvh(config); // Try BVH instead
```

## API Cheat Sheet

| Operation | Method | Returns |
|-----------|--------|---------|
| Insert | `insert(entity, bounds)` | `bool` |
| Remove | `remove(entity)` | `bool` |
| Update | `update(entity, bounds)` | `bool` |
| Force update | `force_update(entity, bounds)` | `bool` |
| Contains | `contains(entity)` | `bool` |
| Get bounds | `get_bounds(entity)` | `Option<&Aabb>` |
| Box query | `query(&bounds)` | `Vec<Entity>` |
| Radius query | `query_radius(pos, radius)` | `Vec<Entity>` |
| Ray query | `query_ray(origin, dir, max)` | `Vec<Entity>` |
| Sorted ray | `query_ray_sorted(...)` | `Vec<(Entity, f32)>` |
| Nearest | `query_nearest(pos, max)` | `Option<(Entity, f32)>` |
| K nearest | `query_k_nearest(pos, k, max)` | `Vec<(Entity, f32)>` |
| Frustum | `query_frustum(&frustum)` | `Vec<Entity>` |
| Flush | `flush_updates()` | - |
| Rebalance | `rebalance_if_needed()` | - |
| Rebuild | `rebuild()` | - |
| Clear | `clear()` | - |
| Stats | `stats()` | `SpatialStats` |

## Examples

Run the demos:
```bash
cargo run --example spatial_partitioning_demo
cargo run --example spatial_optimization_demo
```
