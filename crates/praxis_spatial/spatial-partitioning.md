# Spatial Partitioning System

## Overview

The Praxis spatial partitioning system provides efficient scene organization and querying for 3D game worlds. It implements both Octree and BVH (Bounding Volume Hierarchy) data structures with automatic management, dynamic updates, and ECS integration.

## Key Features

### 1. Multiple Data Structures

**Octree**
- Hierarchical subdivision of 3D space into eight octants
- Best for uniformly distributed objects
- Efficient for range queries and neighbor finding
- Configurable depth and entity limits per node
- Automatic rebalancing based on entity distribution

**BVH (Bounding Volume Hierarchy)**
- Bottom-up tree construction using spatial proximity
- Better for ray tracing and non-uniform distributions
- Efficient rebuilding for dynamic scenes
- Surface area heuristic for optimal splits

### 2. Dynamic Management

**Movement Tracking**
- Configurable movement threshold to minimize updates
- Only updates when entities move significantly
- Tracks previous positions automatically
- Dirty entity tracking for batch processing

**Automatic Rebalancing**
- Monitors structure quality
- Rebuilds when imbalance detected
- Configurable rebalance intervals
- Minimal performance impact

### 3. Query Types

**Bounding Box Queries**
```rust
let query_bounds = Aabb::from_center_half_extents(Vec3::ZERO, Vec3::splat(50.0));
let results = manager.query(&query_bounds);
```

**Radius Queries**
```rust
let results = manager.query_radius(Vec3::ZERO, 100.0);
```

**Ray Queries**
```rust
let origin = Vec3::new(0.0, 0.0, 0.0);
let direction = Vec3::X;
let max_distance = 1000.0;
let results = manager.query_ray(origin, direction, max_distance);
```

**Sorted Ray Queries** (returns entities with distances)
```rust
let sorted_hits = manager.query_ray_sorted(origin, direction, max_distance);
for (entity, distance) in sorted_hits {
    println!("Hit {:?} at {:.2}", entity, distance);
}
```

**Frustum Queries**
```rust
let frustum = Frustum::from_view_projection(view_projection_matrix);
let visible = manager.query_frustum(&frustum);
```

### 4. ECS Integration

**Components**
- `SpatialEntity`: Marks an entity for spatial tracking
- `SpatialBounds`: Cached AABB for quick queries
- `SpatialBundle`: Convenience bundle for spawning

**Systems**
- `insert_spatial_entities`: Adds new entities to structure
- `update_spatial_entities`: Updates moved entities
- `remove_spatial_entities`: Removes despawned entities
- `flush_spatial_updates`: Processes pending updates
- `auto_rebalance_spatial`: Performs automatic rebalancing

**Resource**
- `SpatialResource`: Holds the spatial manager instance

## Usage Patterns

### Basic Setup

```rust
use praxis_spatial::{SpatialManager, SpatialConfig, SpatialStructureType};

let config = SpatialConfig {
    center: Vec3::ZERO,
    size: 1000.0,
    max_entities_per_node: 8,
    movement_threshold: 0.5,
    rebalance_interval: 100,
};

let mut manager = SpatialManager::new(config, SpatialStructureType::Octree);
```

### Entity Management

```rust
// Insert entity
let entity = Entity::from_raw(1);
let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
manager.insert(entity, bounds);

// Update position (only if moved beyond threshold)
let new_bounds = Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0));
manager.update(entity, new_bounds);

// Force update (ignores threshold)
manager.force_update(entity, new_bounds);

// Remove entity
manager.remove(entity);
```

### Query and Maintenance

```rust
// Perform queries
let nearby = manager.query_radius(player_pos, 100.0);

// Flush pending updates
manager.flush_updates();

// Manual rebalancing
if manager.needs_rebalancing() {
    manager.rebuild();
}

// Get statistics
let stats = manager.stats();
println!("Entities: {}, Dirty: {}", stats.entity_count, stats.dirty_count);
```

### ECS Integration Pattern

```rust
use praxis_ecs::{World, Schedule, IntoSystemConfigs};
use praxis_spatial::*;

fn setup_spatial_world() -> (World, Schedule) {
    let mut world = World::new();
    
    // Initialize spatial resource
    let config = SpatialConfig::default();
    world.insert_resource(SpatialResource::new_octree(config));
    
    // Setup systems
    let mut schedule = Schedule::default();
    schedule.add_systems((
        insert_spatial_entities,
        update_spatial_entities,
        remove_spatial_entities,
        flush_spatial_updates,
        auto_rebalance_spatial,
    ).chain());
    
    (world, schedule)
}

fn spawn_spatial_entity(world: &mut World) {
    world.spawn(SpatialBundle::from_center_half_extents(
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::splat(2.0),
    ));
}

fn query_spatial(world: &World) {
    let spatial = world.inner().resource::<SpatialResource>();
    let results = spatial.manager.query_radius(Vec3::ZERO, 50.0);
    println!("Found {} entities nearby", results.len());
}
```

## Performance Considerations

### Choosing a Structure Type

**Use Octree when:**
- Objects are uniformly distributed
- You need simple spatial queries (radius, box)
- Objects are mostly static or move slowly
- You want predictable memory usage

**Use BVH when:**
- Objects are clustered or non-uniformly distributed
- You need ray tracing or ray casting
- Objects move frequently
- You need better cache performance

### Optimization Tips

1. **Tune the movement threshold**: Higher values reduce updates but may cause query inaccuracy
2. **Adjust rebalance interval**: Balance between structure quality and performance
3. **Set appropriate max entities per node**: Lower values = deeper tree, higher = shallower
4. **Use batch operations**: Flush updates in batches rather than per-frame
5. **Profile your queries**: Different query types have different performance characteristics

### Typical Performance

- **Insertion**: O(log n) for both structures
- **Query (radius/box)**: O(log n + k) where k is result count
- **Ray query**: O(log n) for BVH, O(n) worst case for Octree
- **Rebalancing**: O(n log n) for both structures

## Advanced Features

### Custom Configurations

```rust
let aggressive_config = SpatialConfig {
    center: Vec3::ZERO,
    size: 2000.0,
    max_entities_per_node: 4,  // Smaller nodes, deeper tree
    movement_threshold: 0.1,   // Very sensitive to movement
    rebalance_interval: 50,    // Frequent rebalancing
};

let relaxed_config = SpatialConfig {
    center: Vec3::ZERO,
    size: 2000.0,
    max_entities_per_node: 16,  // Larger nodes, shallower tree
    movement_threshold: 5.0,    // Less sensitive to movement
    rebalance_interval: 500,    // Infrequent rebalancing
};
```

### Hybrid Approaches

You can use both structures simultaneously:

```rust
let mut octree_manager = SpatialManager::new_octree(config.clone());
let mut bvh_manager = SpatialManager::new_bvh(config);

// Use octree for radius queries
let nearby = octree_manager.query_radius(pos, radius);

// Use BVH for ray queries
let ray_hits = bvh_manager.query_ray_sorted(origin, direction, max_distance);
```

### Integration with Rendering

```rust
fn cull_and_render(
    spatial: &SpatialResource,
    frustum: &Frustum,
    camera_pos: Vec3,
) {
    // Frustum culling
    let visible = spatial.manager.query_frustum(frustum);
    
    // Distance culling
    let culled: Vec<_> = visible.iter()
        .filter(|&&entity| {
            if let Some(bounds) = spatial.manager.get_bounds(entity) {
                bounds.center().distance(camera_pos) < MAX_RENDER_DISTANCE
            } else {
                false
            }
        })
        .copied()
        .collect();
    
    // Render culled entities
    for entity in culled {
        // render(entity);
    }
}
```

## Testing

The system includes comprehensive tests:

```bash
# Run all spatial tests
cargo test -p praxis_spatial

# Run specific test module
cargo test -p praxis_spatial spatial_manager

# Run with output
cargo test -p praxis_spatial -- --nocapture
```

## Examples

See the examples directory:

```bash
# Comprehensive demo
cargo run --example spatial_partitioning_demo

# Original optimization demo
cargo run --example spatial_optimization_demo
```

## Best Practices

1. **Initialize early**: Set up spatial resources before spawning entities
2. **Use bundles**: Prefer `SpatialBundle` for spawning entities
3. **Let systems manage**: Don't manually update spatial structures when using ECS
4. **Profile first**: Measure before optimizing thresholds
5. **Consider scene type**: Choose structure based on your content
6. **Batch operations**: Group inserts/updates when possible
7. **Monitor statistics**: Use `stats()` to track performance

## Common Patterns

### Player Proximity Detection

```rust
fn find_nearby_enemies(
    spatial: &SpatialResource,
    player_pos: Vec3,
) -> Vec<Entity> {
    spatial.manager.query_radius(player_pos, 50.0)
        .into_iter()
        .filter(|&entity| is_enemy(entity))
        .collect()
}
```

### Projectile Collision

```rust
fn check_projectile_hit(
    spatial: &SpatialResource,
    start: Vec3,
    direction: Vec3,
    max_distance: f32,
) -> Option<(Entity, f32)> {
    let hits = spatial.manager.query_ray_sorted(start, direction, max_distance);
    hits.first().copied()
}
```

### Area of Effect

```rust
fn apply_explosion_damage(
    spatial: &SpatialResource,
    center: Vec3,
    radius: f32,
    damage: f32,
) {
    let affected = spatial.manager.query_radius(center, radius);
    for entity in affected {
        apply_damage(entity, damage);
    }
}
```

## Troubleshooting

**Entities not appearing in queries:**
- Check if entity has `SpatialEntity` component
- Verify bounds are correct and not empty
- Ensure systems are running in correct order

**Poor performance:**
- Reduce movement threshold if too many updates
- Increase rebalance interval
- Check if rebalancing too frequently
- Consider switching structure type

**Memory usage high:**
- Reduce max_entities_per_node for octree
- Clean up removed entities
- Check for entity leaks

## Future Enhancements

Potential additions:
- Quadtree for 2D games
- K-d tree for specific use cases
- Parallel query execution
- GPU-accelerated queries
- Temporal coherence optimization
- Incremental updates without full rebuild
