# Spatial Structure Selection Guide

This guide helps you choose the right spatial data structure for your use case in the Praxis engine.

## Quick Decision Tree

```
Need spatial optimization?
├─ YES → Continue below
└─ NO → Skip spatial structures

Many dynamic objects?
├─ YES → Use SpatialManager with movement threshold
└─ NO → Use Octree or BVH directly

Primary use case?
├─ Ray tracing / picking → BVH
├─ Frustum culling → BVH (slight edge) or Octree
├─ Voxel data → Octree
└─ General queries → SpatialManager (configurable)

Need LOD?
└─ Use LodManager (independent of spatial structure)
```

## Structure Comparison

| Feature | Octree | BVH | SpatialManager |
|---------|--------|-----|----------------|
| **Structure** | 8-way tree | Binary tree | Wrapper (both) |
| **Space division** | Fixed grid | Adaptive | Either |
| **Best for** | Uniform distribution | Ray queries | General use |
| **Insert cost** | O(log n) | O(n log n) rebuild | Batched |
| **Query cost** | O(log n + k) | O(log n + k) | Same as underlying |
| **Update cost** | O(log n) | O(n log n) rebuild | Threshold-based |
| **Memory** | Moderate | Moderate | Higher (tracking) |
| **Cache performance** | Fair | Good (binary) | Same as underlying |

## Detailed Structure Descriptions

### Octree

**How it works:** Recursively subdivides 3D space into 8 equal octants (2×2×2 grid).

**Use when:**
- Objects are uniformly distributed in space
- Working with voxel or volumetric data
- Spatial intuition is important (debugging, visualization)
- Objects move infrequently

**Avoid when:**
- Objects are heavily clustered (cities, forests)
- Frequent insertions/removals
- Objects span multiple octants (loose octree problem)

**Example:**
```rust
use praxis_spatial::{Octree, Aabb};
use praxis_math::Vec3;

let mut octree = Octree::new(
    Vec3::ZERO,     // Center
    1000.0,         // Size
    8               // Max entities per node
);

octree.insert(entity, bounds);
let results = octree.query(&query_bounds);
```

### BVH (Bounding Volume Hierarchy)

**How it works:** Binary tree where each node's bounds tightly enclose its children.

**Use when:**
- Primary use is ray tracing or ray casting (mouse picking, line-of-sight)
- Objects are clustered or non-uniformly distributed
- Doing frustum culling for rendering
- Objects are mostly static

**Avoid when:**
- Objects change position frequently (expensive rebuild)
- Working with uniform grid-like data
- Need incremental updates

**Example:**
```rust
use praxis_spatial::{Bvh, Aabb};

let mut bvh = Bvh::new();
bvh.insert(entity, bounds);

// Ray query (BVH excels at this)
let hits = bvh.query_ray(origin, direction, max_distance);
```

### SpatialManager (Recommended for Most Use Cases)

**How it works:** Unified interface over Octree or BVH with automatic update management.

**Use when:**
- Building general-purpose systems
- Need automatic movement tracking
- Want to defer structure choice
- Need batched updates for performance

**Avoid when:**
- Need fine-grained control over rebuilds
- Overhead is critical (use direct Octree/BVH)
- Implementing specialized spatial algorithms

**Example:**
```rust
use praxis_spatial::{SpatialManager, SpatialConfig, SpatialStructureType, Aabb};
use praxis_math::Vec3;

let config = SpatialConfig {
    center: Vec3::ZERO,
    size: 1000.0,
    max_entities_per_node: 8,
    movement_threshold: 0.5,  // Only update if moved >0.5 units
    rebalance_interval: 100,   // Rebuild after 100 updates
};

// Choose structure type at creation
let mut manager = SpatialManager::new(config, SpatialStructureType::Bvh);

manager.insert(entity, bounds);
manager.update(entity, new_bounds);  // Automatic threshold check
manager.flush_updates();              // Process batched updates
```

### LodManager (Separate System)

**How it works:** Maps entities to LOD groups and selects mesh detail based on distance.

**Use when:**
- Scene has many distant objects
- Objects have multiple detail levels
- Rendering performance is critical
- Working with outdoor/large environments

**Combine with:** Any spatial structure for complete optimization pipeline.

**Example:**
```rust
use praxis_spatial::{LodManager, LodGroup, LodLevel};

let mut lod_manager = LodManager::new();

// Define LOD levels (distance thresholds and mesh IDs)
lod_manager.register_lod_levels(
    "tree",
    vec![
        LodLevel::new(50.0, "tree_high"),    // 0-50 units
        LodLevel::new(100.0, "tree_medium"), // 50-100 units
        LodLevel::new(200.0, "tree_low"),    // 100+ units
    ],
);

lod_manager.assign_entity(entity, "tree");
let lod = lod_manager.select_lod(entity, camera_pos, entity_pos);
```

## Performance Characteristics

### Octree Performance

**Strengths:**
- Predictable performance with uniform distribution
- Simple insertion: O(log n) average
- Good spatial locality for nearby queries

**Weaknesses:**
- Degrades with clustered objects
- Empty space wastes tree nodes
- Objects spanning octants stored at higher levels (tested more often)

**Typical scenario (10,000 uniformly distributed objects):**
- Insertion: ~20-50 node tests per object
- Point query: ~10-20 node tests
- Range query: ~50-200 node tests (depends on query size)

### BVH Performance

**Strengths:**
- Near-optimal ray tracing: O(log n) average
- Tight bounds: no wasted space
- Binary branching: better CPU cache performance
- Naturally adapts to clustering

**Weaknesses:**
- Full rebuild on any change: O(n log n)
- Construction more complex than octree
- Not suitable for frequent updates

**Typical scenario (10,000 clustered objects):**
- Build time: ~5-20ms (one-time)
- Ray query: ~10-30 node tests
- Frustum culling: 20-40% faster than octree (tighter bounds)

### SpatialManager Performance

**Overhead:**
- Movement tracking: HashMap lookup per update
- Threshold checking: Vec3 distance calculation
- Batching: HashSet insertions for dirty tracking

**Benefits:**
- Avoids unnecessary rebuilds (movement threshold)
- Batches updates for efficiency
- Amortizes rebuild cost over many changes

**Typical overhead: 5-10% vs direct structure use, but often faster overall due to avoided rebuilds.**

## Common Optimization Patterns

### Pattern 1: Static Scene Rendering

**Scenario:** Rendering a mostly-static scene with frustum culling.

**Solution:** BVH + LodManager
```rust
let mut bvh = Bvh::new();
let mut lod_manager = LodManager::new();

// Build once
for (entity, bounds) in entities {
    bvh.insert(entity, bounds);
    lod_manager.assign_entity(entity, "object_type");
}

// Each frame
let visible = bvh.query_with_predicate(&|bounds| frustum.intersects(bounds));
let lods = lod_manager.select_lods(&visible, camera_pos);
```

**Why:** BVH's tight bounds minimize false positives in frustum culling. One-time build cost is acceptable for static scenes.

### Pattern 2: Dynamic Objects with Movement

**Scenario:** Objects move frequently but not every frame.

**Solution:** SpatialManager with movement threshold
```rust
let mut manager = SpatialManager::new_bvh(SpatialConfig {
    movement_threshold: 1.0,  // Only update if moved >1 unit
    rebalance_interval: 50,   // Rebuild after 50 significant moves
    ..Default::default()
});

// Each frame
for (entity, new_bounds) in moved_objects {
    manager.update(entity, new_bounds);  // Checks threshold
}
manager.flush_updates();  // Batched rebuild if needed
```

**Why:** Threshold avoids rebuilding for tiny movements. Batching amortizes rebuild cost.

### Pattern 3: Ray Casting (Mouse Picking)

**Scenario:** User clicks, need to find clicked object.

**Solution:** BVH direct
```rust
let mut bvh = Bvh::new();
// ... populate BVH ...

let (origin, direction) = camera.screen_to_ray(mouse_pos);
let hits = bvh.query_ray_sorted(origin, direction, 1000.0);

if let Some((entity, distance)) = hits.first() {
    // Clicked entity found
}
```

**Why:** BVH is near-optimal for ray queries. Sorted results give closest object first.

### Pattern 4: Voxel World

**Scenario:** Minecraft-like voxel terrain.

**Solution:** Octree direct
```rust
let mut octree = Octree::new(Vec3::ZERO, 1024.0, 16);

// Voxels naturally map to octree structure
for voxel in active_voxels {
    octree.insert(voxel.entity, voxel.bounds);
}

// Efficient spatial queries
let nearby_voxels = octree.query_radius(player_pos, 32.0);
```

**Why:** Voxels are uniformly distributed, natural fit for octree's grid-based subdivision.

## Naming Convention Compliance

This crate follows Praxis naming conventions:

- **`SpatialManager`**: Manages spatial structures (Octree/BVH), handles updates/queries
  - Retains "Spatial" prefix because it manages multiple structure types
- **`LodManager`**: Manages LOD groups and entity assignments
  - Renamed from `SpatialLodManager` to avoid redundant "Spatial" prefix
- **`Octree`/`Bvh`**: Pure data structures (no suffix needed)
- **`FrustumCuller`**: Performs culling operations (verb-based name acceptable)

## Migration Guide

If you were using `SpatialLodManager`, update your code:

```rust
// Old (deprecated)
use praxis_spatial::SpatialLodManager;
let mut manager = SpatialLodManager::new();

// New (recommended)
use praxis_spatial::LodManager;
let mut manager = LodManager::new();
```

The API is identical, only the name changed. `SpatialLodManager` remains as a deprecated type alias for backwards compatibility.

## See Also

- [SPATIAL_PARTITIONING.md](SPATIAL_PARTITIONING.md) - Detailed octree/BVH implementation
- [QUICK_REFERENCE.md](QUICK_REFERENCE.md) - API patterns and examples
- [../../docs/guides/spatial-optimization.md](../../docs/guides/spatial-optimization.md) - Complete optimization guide
