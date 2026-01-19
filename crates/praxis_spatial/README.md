# Praxis Spatial

Spatial optimization with culling, LOD, octree, and BVH for the Praxis game engine.

## Overview

Comprehensive spatial data structures and optimization systems for efficient rendering and queries.

**Key Features:**
- Frustum culling with AABB/sphere tests
- Octree and BVH spatial partitioning
- Distance-based LOD system
- Hardware occlusion queries (Vulkan)
- Unified spatial manager with automatic updates
- ECS integration with component-based API

## Quick Start

### Spatial Manager

```rust
use praxis_spatial::{SpatialManager, SpatialConfig, Aabb};

let config = SpatialConfig {
    center: Vec3::ZERO,
    size: 1000.0,
    max_entities_per_node: 8,
    movement_threshold: 0.5,
    ..Default::default()
};

let mut manager = SpatialManager::new_octree(config);

// Insert entities
manager.insert(entity, bounds);

// Query
let nearby = manager.query_radius(Vec3::ZERO, 50.0);
let ray_hits = manager.query_ray_sorted(origin, direction, max_distance);

// Update when moved
manager.update(entity, new_bounds);
manager.flush_updates();
```

### Frustum Culling

```rust
use praxis_spatial::{FrustumCuller, Aabb};

let mut culler = FrustumCuller::new();
culler.update(view_proj_matrix);

if culler.is_visible(&entity_bounds) {
    // Render entity
}
```

### LOD System

```rust
use praxis_spatial::{LodManager, LodGroup, LodLevel};

lod_manager.register_lod_group(LodGroup::new(
    "tree",
    vec![
        LodLevel::new(0.0, "tree_high"),
        LodLevel::new(50.0, "tree_medium"),
        LodLevel::new(100.0, "tree_low"),
    ],
));

lod_manager.assign_entity(entity, "tree");
let lod = lod_manager.select_lod(entity, camera_pos, entity_pos);
```

## When to Use Each Structure

### Octree
**Best For:**
- Static/slow-moving objects
- Uniform object distribution
- Voxel or volumetric data
- Simple spatial queries

**Pros:**
- Intuitive spatial subdivision
- Good for evenly distributed objects
- Predictable memory layout

**Cons:**
- Poor with non-uniform distribution
- Inefficient for dynamic objects

### BVH (Bounding Volume Hierarchy)
**Best For:**
- Ray tracing and ray casting
- Non-uniform object distribution
- Clustered objects (cities, forests)
- Static mesh rendering

**Pros:**
- Near-optimal for ray queries
- Tight-fitting bounds (no wasted space)
- Better cache performance (binary tree)
- Adapts to object clustering

**Cons:**
- Requires full rebuild on changes
- More complex construction

### SpatialManager (Recommended)
**Best For:**
- General-purpose spatial management
- When you need automatic updates
- Applications that may switch between Octree/BVH

**Pros:**
- Unified API for both structures
- Automatic movement tracking
- Batched updates and rebalancing
- Simple to use

**Cons:**
- Slight overhead vs direct structure use

### LodManager (Distance-Based Detail)
**Best For:**
- Large outdoor environments
- High object counts
- Distant objects
- Performance-critical rendering

**Use With:**
- Combine with spatial structures for complete optimization
- Works independently of culling system

### Occlusion Culling
**Best For:**
- Dense urban/indoor scenes
- Large occluders (buildings, walls)
- Complex geometry

**Note:** Hardware-dependent, requires GPU support

## Documentation

**Comprehensive Guide:**
- [Spatial Optimization Guide](../../docs/guides/spatial-optimization.md) - Complete guide and best practices

**Crate Documentation:**
- [Structure Selection Guide](STRUCTURE_SELECTION.md) - Choosing the right spatial structure
- [Spatial Partitioning](SPATIAL_PARTITIONING.md) - Octree/BVH details
- [Quick Reference](QUICK_REFERENCE.md) - API patterns

## Examples

```bash
cargo run --example spatial_partitioning_demo
cargo run --example spatial_optimization_demo
```

## Dependencies

- `bevy_ecs` 0.14: ECS integration
- `vulkano`: Occlusion queries (optional)
