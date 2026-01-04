# Praxis Spatial - Spatial Optimization Systems

Comprehensive spatial optimization systems for the Praxis game engine, providing efficient culling and Level-of-Detail (LOD) management.

> **For high-level guidance on spatial optimization concepts and when to use each technique, see [docs/guides/spatial-optimization.md](../../docs/guides/spatial-optimization.md)**

## Features

### 🔍 Frustum Culling
- View frustum extraction from camera view-projection matrices
- AABB intersection tests for fast rejection of off-screen objects
- Sphere intersection tests for alternative bounding volumes
- Dramatically reduces rendering cost by eliminating invisible objects

### 🌳 Spatial Partitioning

#### Octree
- Recursive space subdivision into eight octants
- Configurable maximum depth and entities per node
- Fast spatial queries (radius, AABB, ray)
- Dynamic insertion, removal, and updates
- Automatic rebalancing based on entity distribution
- Movement threshold tracking to minimize updates
- Efficient for static or slowly moving objects

#### BVH (Bounding Volume Hierarchy)
- Bottom-up construction for optimal tree structure
- Surface area heuristic (SAH) for splitting
- Fast ray tracing and nearest neighbor queries
- Dynamic insertion, removal with automatic rebuild
- Efficient for both static and dynamic scenes
- Better than octrees for ray tracing

#### Spatial Manager
- Unified interface for both octree and BVH
- Automatic tracking of entity movement
- Configurable movement threshold
- Automatic rebalancing at intervals
- Dirty entity tracking for batch updates
- Easy switching between structure types

### 📐 Level of Detail (LOD) System
- Distance-based mesh switching to reduce polygon count
- LOD groups with multiple quality levels
- Smooth transitions between LOD levels
- Configurable distance thresholds per object type
- Significant performance gains for distant objects

### 🚫 Occlusion Culling
- Hardware occlusion queries using Vulkan
- Query pool management for efficient GPU queries
- Temporal coherence - reuse previous frame results
- Conditional rendering - skip fully occluded objects
- Reduces overdraw and fragment shader cost

## Architecture

```
┌─────────────────────────────────────────────────────┐
│             Visibility Determination                 │
│                                                      │
│  ┌────────────┐    ┌──────────┐    ┌────────────┐ │
│  │  Frustum   │───▶│   LOD    │───▶│ Occlusion  │ │
│  │  Culling   │    │ Selection│    │  Culling   │ │
│  └────────────┘    └──────────┘    └────────────┘ │
│         │                 │                │        │
│         ▼                 ▼                ▼        │
│    ┌─────────────────────────────────────────┐    │
│    │      Spatial Data Structures             │    │
│    │  ┌──────────┐        ┌─────────┐        │    │
│    │  │ Octree   │        │   BVH   │        │    │
│    │  └──────────┘        └─────────┘        │    │
│    └─────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

## Usage

### Spatial Manager (Recommended)

```rust
use praxis_spatial::{SpatialManager, SpatialConfig, Aabb};
use praxis_ecs::Entity;
use praxis_math::Vec3;

// Create a spatial manager with octree
let config = SpatialConfig {
    center: Vec3::ZERO,
    size: 1000.0,
    max_entities_per_node: 8,
    movement_threshold: 0.5,
    rebalance_interval: 100,
};
let mut manager = SpatialManager::new_octree(config);

// Insert entities
let entity = Entity::from_raw(1);
let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
manager.insert(entity, bounds);

// Update when entity moves
let new_bounds = Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0));
manager.update(entity, new_bounds);

// Query entities
let results = manager.query_radius(Vec3::ZERO, 50.0);

// Ray queries
let ray_hits = manager.query_ray_sorted(Vec3::ZERO, Vec3::X, 100.0);
for (entity, distance) in ray_hits {
    println!("Hit entity {:?} at distance {}", entity, distance);
}

// Automatic rebalancing
manager.flush_updates();
if manager.needs_rebalancing() {
    manager.rebalance_if_needed();
}
```

### ECS Integration

```rust
use praxis_ecs::{World, Schedule};
use praxis_spatial::{
    SpatialResource, SpatialBundle, SpatialConfig,
    insert_spatial_entities, update_spatial_entities,
    remove_spatial_entities, flush_spatial_updates,
};
use praxis_math::Vec3;

let mut world = World::new();

// Setup spatial resource
world.insert_resource(SpatialResource::new_octree(SpatialConfig::default()));

// Add systems to schedule
let mut schedule = Schedule::default();
schedule.add_systems((
    insert_spatial_entities,
    update_spatial_entities,
    remove_spatial_entities,
    flush_spatial_updates,
).chain());

// Spawn entities with spatial components
world.spawn(SpatialBundle::from_center_half_extents(
    Vec3::new(10.0, 0.0, 0.0),
    Vec3::splat(2.0),
));

// Run systems
schedule.run(world.inner_mut());

// Query the spatial structure
let spatial = world.inner().resource::<SpatialResource>();
let nearby = spatial.manager.query_radius(Vec3::ZERO, 50.0);
```

### Basic Frustum Culling

```rust
use praxis_spatial::{FrustumCuller, Aabb};
use praxis_math::{Mat4, Vec3};

// Create frustum culler
let mut culler = FrustumCuller::new();

// Update with camera matrices
let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::ZERO, Vec3::Y);
let proj = Mat4::perspective_rh(60.0_f32.to_radians(), 16.0 / 9.0, 0.1, 1000.0);
culler.update(proj * view);

// Test object visibility
let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
if culler.is_visible(&bounds) {
    // Render the object
}
```

### Octree Spatial Partitioning

```rust
use praxis_spatial::{Octree, Aabb};
use praxis_ecs::Entity;
use praxis_math::Vec3;

// Create octree (center, size, max entities per node)
let mut octree = Octree::new(Vec3::ZERO, 1000.0, 8);

// Insert entities
let entity = Entity::from_raw(1);
let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
octree.insert(entity, bounds);

// Query nearby entities
let nearby = octree.query_radius(Vec3::ZERO, 50.0);

// Query entities in a box
let query_bounds = Aabb::from_min_max(Vec3::new(-10.0, -10.0, -10.0), Vec3::new(10.0, 10.0, 10.0));
let in_box = octree.query(&query_bounds);
```

### BVH for Ray Tracing

```rust
use praxis_spatial::{Bvh, Aabb};
use praxis_ecs::Entity;
use praxis_math::Vec3;

let mut bvh = Bvh::new();

// Build BVH from entities
let entities = vec![
    (Entity::from_raw(1), Aabb::from_min_max(Vec3::ZERO, Vec3::ONE)),
    (Entity::from_raw(2), Aabb::from_min_max(Vec3::new(5.0, 0.0, 0.0), Vec3::new(6.0, 1.0, 1.0))),
];
bvh.build(entities);

// Query intersections
let query_bounds = Aabb::from_min_max(Vec3::new(-5.0, -5.0, -5.0), Vec3::new(5.0, 5.0, 5.0));
let results = bvh.query(&query_bounds);
```

### LOD System

```rust
use praxis_spatial::{LodManager, LodGroup, LodLevel};
use praxis_ecs::Entity;
use praxis_math::Vec3;

let mut lod_manager = LodManager::new();

// Register LOD group with distance thresholds
lod_manager.register_lod_group(LodGroup::new(
    "tree",
    vec![
        LodLevel::new(0.0, "tree_high"),      // 0-50 units
        LodLevel::new(50.0, "tree_medium"),   // 50-100 units
        LodLevel::new(100.0, "tree_low"),     // 100-200 units
        LodLevel::new(200.0, "tree_billboard"), // 200+ units
    ],
));

// Assign entity to LOD group
let entity = Entity::from_raw(1);
lod_manager.assign_entity(entity, "tree");

// Select appropriate LOD based on distance
let camera_pos = Vec3::new(0.0, 0.0, 0.0);
let entity_pos = Vec3::new(75.0, 0.0, 0.0);
let selection = lod_manager.select_lod(entity, camera_pos, entity_pos);

if let Some(lod) = selection {
    println!("Use mesh: {}", lod.mesh_id);
    println!("LOD level: {}", lod.level_index);
    println!("Distance: {}", lod.distance);
}
```

### Occlusion Culling

```rust
use praxis_spatial::{OcclusionCuller, Aabb};
use praxis_ecs::Entity;
use praxis_math::Vec3;
use std::sync::Arc;

// Create occlusion culler (requires Vulkan device and allocator)
let mut occlusion_culler = OcclusionCuller::new(
    device.clone(),
    queue.clone(),
    allocator.clone(),
    1024, // max queries
)?;

// Begin occlusion test for an entity
let entity = Entity::from_raw(1);
let bounds = Aabb::from_min_max(Vec3::ZERO, Vec3::ONE);
let query_index = occlusion_culler.begin_test(&mut command_buffer, entity, bounds)?;

// Render bounding box with occlusion query
// ... render bounding box geometry ...

// End occlusion test
occlusion_culler.end_test(&mut command_buffer, query_index)?;

// Later, retrieve results
occlusion_culler.update_results()?;

// Check if entity is visible
if occlusion_culler.is_visible(entity) {
    // Render the full entity
}
```

### Unified Visibility System

```rust
use praxis_spatial::{VisibilitySystem, Aabb};
use praxis_ecs::Entity;
use praxis_math::{Mat4, Vec3};

let mut visibility_system = VisibilitySystem::with_max_distance(500.0);

// Update frustum
let view_proj = proj * view;
visibility_system.update_frustum(view_proj);

// Setup LOD groups
visibility_system.lod_manager_mut().register_lod_levels(
    "tree",
    vec![
        LodLevel::new(0.0, "tree_high"),
        LodLevel::new(50.0, "tree_low"),
    ],
);

// Cull entities
let entities = vec![
    (Entity::from_raw(1), Aabb::from_min_max(Vec3::ZERO, Vec3::ONE), Vec3::ZERO),
    // ... more entities ...
];

let camera_pos = Vec3::new(0.0, 0.0, 10.0);
let (results, stats) = visibility_system.cull_entities(&entities, camera_pos);

println!("Visible: {}", stats.visible_objects);
println!("Frustum culled: {}", stats.frustum_culled);
println!("Distance culled: {}", stats.distance_culled);
println!("Cull rate: {:.1}%", stats.cull_rate());
```

## Performance Considerations

### When to Use Each Structure

**Octree:**
- Static or slowly moving objects
- Uniform spatial distribution
- Simple spatial queries (radius, box)
- Lower construction cost

**BVH:**
- Dynamic objects that move frequently
- Non-uniform spatial distribution
- Ray tracing and closest-point queries
- Better cache performance

**LOD System:**
- Large open worlds with distant objects
- Objects with multiple mesh quality levels
- Significant poly count differences between levels
- Best for outdoor environments

**Occlusion Culling:**
- Dense urban environments
- Indoor scenes with lots of occlusion
- Scenes with large occluders (buildings, terrain)
- When CPU culling alone isn't enough

### Best Practices

1. **Combine techniques:** Use frustum culling first (cheapest), then LOD, then occlusion
2. **Update spatial structures incrementally:** Only rebuild when objects move significantly
3. **Use temporal coherence:** Assume objects stay visible between frames
4. **Profile your scene:** Different techniques work better for different content
5. **Batch queries:** Query multiple objects at once for better cache utilization
6. **Configure thresholds:** Tune octree depth, LOD distances, query pool size for your scene

### Configuration Parameters

#### Octree
- **Size:** Should encompass all objects with margin
- **Max Depth:** 8-12 levels (deeper = finer subdivision)
- **Max Objects:** 4-16 per node (lower = more subdivision)

#### BVH
- Automatically balanced during construction
- No manual tuning required
- Rebuild when >10-20% of objects move

#### LOD
- **Distance Thresholds:**
  - LOD 0→1: 30-50% of max view distance
  - LOD 1→2: 50-70% of max view distance
  - LOD 2→3: 70-90% of max view distance
- **Geometric Ratios:** Each level should be 30-50% the polycount of the previous

## Integration with ECS

Add spatial components to entities:

```rust
use praxis_ecs::{World, Transform, BoundingBox, LodComponent, MeshHandle};
use praxis_math::Vec3;

let mut world = World::new();

world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("tree_high"),
    BoundingBox::from_center_half_extents(Vec3::ZERO, Vec3::splat(3.0)),
    LodComponent::new("tree"),
));
```

## Examples

### Spatial Partitioning Demo

Run the comprehensive spatial partitioning demo:

```bash
cargo run --example spatial_partitioning_demo
```

This demonstrates:
- Octree and BVH creation and usage
- Dynamic insertion, removal, and updates
- Ray queries and sorted ray results
- Spatial manager with automatic rebalancing
- ECS integration with spatial systems
- Movement threshold and dirty entity tracking

### Spatial Optimization Demo

Run the original spatial optimization demo:

```bash
cargo run --example spatial_optimization_demo
```

This demonstrates:
- Creating and populating an octree and BVH
- Frustum culling with different camera positions
- LOD selection based on distance
- Spatial queries (radius and AABB)
- Performance statistics

## Additional Documentation

- **[SPATIAL_PARTITIONING.md](SPATIAL_PARTITIONING.md)** - Detailed spatial partitioning guide
- **[QUICK_REFERENCE.md](QUICK_REFERENCE.md)** - API quick reference and common patterns
- **[docs/guides/spatial-optimization.md](../../docs/guides/spatial-optimization.md)** - High-level concepts and when to use each technique

## Testing

Run the test suite:

```bash
cargo test -p praxis_spatial
```

Tests cover:
- AABB intersection and containment
- Frustum plane extraction and culling
- Octree insertion, query, and removal
- BVH construction and queries
- LOD distance thresholds and selection
- Visibility system integration

## License

MIT
