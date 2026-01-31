# praxis_spatial

Spatial data structures for Praxis engine: octrees, BVH.

## Overview

Provides spatial partitioning data structures for efficient queries and culling.

## Features

### Octree

- Hierarchical spatial partitioning
- Dynamic insertion and removal
- Frustum culling queries
- Radius queries
- Ray casting

### BVH (Bounding Volume Hierarchy)

- Fast ray-triangle intersection
- Static mesh optimization
- AABB queries

### Spatial Hash

- Fixed-size grid for uniform distributions
- O(1) insertion and removal
- Fast neighbor queries

## Example

```rust
use praxis_spatial::{Octree, AABB};

// Create octree
let mut octree = Octree::new(AABB::from_center_size(
    Vec3::ZERO,
    Vec3::splat(100.0),
));

// Insert objects
octree.insert(entity, aabb);

// Query visible objects
let visible = octree.frustum_query(&frustum);

// Radius query
let nearby = octree.radius_query(position, radius);
```

## Use Cases

- Frustum culling for rendering
- Broad-phase collision detection
- Neighbor queries for AI
- LOD selection
- Spatial audio

## Dependencies

- `serde`: Serialization support

## Usage

```toml
praxis_spatial = { path = "../praxis_spatial", version = "0.1.0" }
```
