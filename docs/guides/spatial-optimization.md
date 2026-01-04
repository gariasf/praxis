# Spatial Optimization

High-level guide to spatial optimization techniques in Praxis for improving rendering performance in large 3D scenes.

## Overview

Spatial optimization is critical for rendering performance in large 3D scenes. Without optimization, the renderer would process every object in the scene, even those that are:
- Outside the camera view (off-screen)
- Too far away to be visible
- Hidden behind other geometry
- Too detailed for their distance from camera

Praxis provides four complementary optimization techniques:

1. **Frustum Culling** - Eliminates objects outside the camera view
2. **Spatial Partitioning** (Octree/BVH) - Organizes objects for fast queries
3. **LOD System** - Reduces detail for distant objects
4. **Occlusion Culling** - Skips objects hidden behind others

## When to Use Each Technique

### Frustum Culling
**Best for:** All 3D scenes, especially outdoor environments

- **Test Cost:** ~10-20 cycles per object (very cheap)
- **Typical Cull Rate:** 40-70% of objects in most scenes
- **ROI:** Highest return on investment, should always be enabled

The view frustum is a truncated pyramid representing the visible space from a camera. Objects outside this volume are not visible and can be skipped during rendering.

### Spatial Partitioning

**Octree** - Best for:
- Static or slowly moving objects
- Uniformly distributed objects (grids, forests)
- Simple spatial queries (find nearby objects)
- Lower memory constraints

**BVH (Bounding Volume Hierarchy)** - Best for:
- Dynamic objects that move frequently
- Clustered distributions (cities, interiors)
- Ray tracing and picking
- Complex nearest-neighbor queries

Both structures organize objects spatially but use different approaches. Octrees subdivide space into 8 equal octants recursively, while BVH groups objects by proximity without fixed space subdivision.

### LOD (Level of Detail) System

**Best for:** Large open worlds with distant objects

- **Polygon Count:** Can reduce by 80-95% for distant objects
- **Draw Calls:** Fewer batches due to simpler geometry
- **Frame Rate:** 2-5x improvement in large outdoor scenes

LOD reduces rendering cost by using simpler mesh representations for distant objects. Close objects use high-polygon meshes, distant objects use low-polygon meshes.

**LOD Authoring Guidelines:**
1. Create 3-5 LOD levels per object type
2. Each level should be 30-50% the polycount of the previous
3. Maintain overall shape (silhouette preservation)
4. Use billboards for very distant objects

### Occlusion Culling

**Best for:** Dense urban environments and interiors

**Benefits:**
- Reduces fragment shading by 20-60% in dense scenes
- Most effective with large occluders (buildings, terrain)
- Minimal CPU overhead

**Costs:**
- GPU time for occlusion queries
- One frame latency (uses previous frame results)
- Extra draw calls for bounding boxes

Occlusion culling uses the GPU to test if objects are hidden behind other geometry. Fully occluded objects are skipped, saving fragment shader and bandwidth costs.

## Quick Start Examples

### Basic Frustum Culling

```rust
use praxis_spatial::{FrustumCuller, Aabb};
use praxis_math::{Mat4, Vec3};

// Create culler
let mut culler = FrustumCuller::new();

// Update with camera view-projection matrix
let view = Mat4::look_at_rh(camera_pos, target, Vec3::Y);
let proj = Mat4::perspective_rh(fov, aspect, near, far);
culler.update(proj * view);

// Test each object
for (entity, bounds) in objects {
    if culler.is_visible(&bounds) {
        render_object(entity);
    }
}
```

### Spatial Queries with Octree

```rust
use praxis_spatial::{Octree, Aabb};
use praxis_math::Vec3;

// Create octree: center, size, max_objects_per_node
let mut octree = Octree::new(Vec3::ZERO, 1000.0, 8);

// Insert objects
for entity in entities {
    let bounds = get_bounds(entity);
    octree.insert(entity, bounds);
}

// Query nearby objects (radius query)
let nearby = octree.query_radius(position, 50.0);

// Query objects in volume (AABB query)
let in_box = octree.query(&query_bounds);
```

### LOD Setup and Selection

```rust
use praxis_spatial::{LodManager, LodGroup, LodLevel};

let mut lod_manager = LodManager::new();

// Register LOD group for trees
lod_manager.register_lod_group(LodGroup::new(
    "tree",
    vec![
        LodLevel::new(0.0, "tree_high"),      // 0-50 units: 5000 tris
        LodLevel::new(50.0, "tree_medium"),   // 50-100 units: 1500 tris
        LodLevel::new(100.0, "tree_low"),     // 100-200 units: 500 tris
        LodLevel::new(200.0, "tree_billboard"), // 200+ units: 2 tris
    ],
));

// Assign entity to LOD group
lod_manager.assign_entity(entity, "tree");

// Select appropriate LOD based on distance
let selection = lod_manager.select_lod(
    entity,
    camera_position,
    entity_position
);
```

### Unified Visibility System

The `VisibilitySystem` combines all optimization techniques:

```rust
use praxis_spatial::VisibilitySystem;

// Create system with max render distance
let mut visibility = VisibilitySystem::with_max_distance(500.0);

// Setup LOD groups
visibility.lod_manager_mut().register_lod_levels(/* ... */);

// Each frame:
visibility.update_frustum(view_projection);

// Cull all entities at once
let entities: Vec<(Entity, Aabb, Vec3)> = /* ... */;
let (results, stats) = visibility.cull_entities(&entities, camera_position);

// Process visible entities
for result in results {
    if result.is_visible {
        let mesh_id = result.lod
            .as_ref()
            .map(|lod| lod.mesh_id.as_str())
            .unwrap_or("default");
        
        render_entity(result.entity, mesh_id);
    }
}

// Log performance stats
println!("Visible: {}/{}", stats.visible_objects, stats.total_objects);
println!("Cull rate: {:.1}%", stats.cull_rate());
```

## Performance Tuning Checklist

- [ ] Use frustum culling for all 3D scenes (highest ROI)
- [ ] Add LOD for objects rendered many times (trees, rocks, etc.)
- [ ] Use octree/BVH for scenes with >1000 objects
- [ ] Enable occlusion culling for dense urban/interior scenes
- [ ] Profile to find which technique helps most for your scene
- [ ] Batch spatial queries when possible
- [ ] Update spatial structures incrementally
- [ ] Use temporal coherence (previous frame results)
- [ ] Tune LOD distances based on visual quality needs
- [ ] Monitor culling statistics each frame

## Common Pitfalls

### Bounding Box Too Small
**Problem:** Objects get culled when they should be visible  
**Solution:** Use conservative (slightly larger) bounding boxes

### LOD Distances Too Aggressive
**Problem:** Noticeable popping as objects transition  
**Solution:** Use wider transition zones, add fade-in

### Octree Too Deep
**Problem:** High construction cost, deep traversal  
**Solution:** Limit to 8-10 levels, increase max objects per node

### Occlusion Queries on Small Objects
**Problem:** Query overhead exceeds savings  
**Solution:** Only query medium/large objects, batch small objects

### Rebuilding BVH Every Frame
**Problem:** High CPU cost  
**Solution:** Only rebuild when objects move significantly

## Integration with ECS

```rust
use praxis_ecs::{Query, Res, ResMut, Transform, BoundingBox};
use praxis_spatial::{VisibilitySystem, Aabb};

fn spatial_culling_system(
    entities_query: Query<(Entity, &Transform, &BoundingBox)>,
    mut visibility: ResMut<VisibilitySystem>,
    camera_pos: Res<CameraPosition>,
) {
    // Collect entities for culling
    let entities: Vec<_> = entities_query
        .iter()
        .map(|(entity, transform, bounds)| {
            let world_bounds = Aabb::from_min_max(
                bounds.min + transform.translation,
                bounds.max + transform.translation,
            );
            (entity, world_bounds, transform.translation)
        })
        .collect();
    
    // Perform culling
    let (results, stats) = visibility.cull_entities(&entities, camera_pos.0);
    
    // Results can be stored in a resource for the render system
}
```

## See Also

- **[`praxis_spatial` crate README](../../crates/praxis_spatial/README.md)** - Detailed API documentation and implementation details
- **[Spatial Partitioning Documentation](../../crates/praxis_spatial/SPATIAL_PARTITIONING.md)** - In-depth guide to octrees and BVH
- **[LOD System](../lod_system.md)** - Detailed LOD configuration and best practices
- **[Frustum Culling](../frustum_culling.md)** - Mathematical details of frustum culling

## Examples

Run the spatial optimization demo to see all techniques in action:

```bash
cargo run --example spatial_optimization_demo
cargo run --example spatial_partitioning_demo
```

These examples demonstrate:
- Creating and populating spatial structures
- Frustum culling with different camera positions
- LOD selection based on distance
- Spatial queries (radius and AABB)
- Performance statistics and profiling
