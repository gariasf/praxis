# Spatial Optimization Systems

This guide covers the comprehensive spatial optimization systems in Praxis, including frustum culling, spatial partitioning, LOD management, and occlusion culling.

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

## Frustum Culling

### What is Frustum Culling?

The view frustum is a truncated pyramid representing the visible space from a camera. Objects outside this volume are not visible and can be skipped.

### How It Works

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
        // Object is in view - render it
        render_object(entity);
    }
}
```

### Performance Impact

- **Test Cost:** ~10-20 cycles per object (very cheap)
- **Typical Cull Rate:** 40-70% of objects in most scenes
- **Best For:** All 3D scenes, especially outdoor environments

### Implementation Details

The frustum consists of 6 planes (near, far, left, right, top, bottom). For each object:
1. Find the "positive vertex" - the corner closest to each plane
2. If the positive vertex is behind any plane, the object is outside
3. Otherwise, the object might be visible (conservative test)

## Spatial Partitioning

### Octree vs BVH

Both structures organize objects spatially but use different approaches:

**Octree:**
- Subdivides space into 8 equal octants recursively
- Good for uniform distributions
- Fast insertion and removal
- Lower construction cost

**BVH (Bounding Volume Hierarchy):**
- Groups objects by proximity, not space
- Better for non-uniform distributions
- Excellent for ray tracing
- Faster queries in most cases

### When to Use Each

Use **Octree** for:
- Static or slowly moving objects
- Uniformly distributed objects (grids, forests)
- Simple spatial queries (find nearby objects)
- Lower memory constraints

Use **BVH** for:
- Dynamic objects that move frequently
- Clustered distributions (cities, interiors)
- Ray tracing and picking
- Complex nearest-neighbor queries

### Octree Example

```rust
use praxis_spatial::{Octree, Aabb};
use praxis_ecs::Entity;
use praxis_math::Vec3;

// Create octree: center, size, max_objects_per_node
let mut octree = Octree::new(Vec3::ZERO, 1000.0, 8);

// Insert objects
for entity in entities {
    let bounds = get_bounds(entity);
    octree.insert(entity, bounds);
}

// Update moved objects
octree.update(entity, new_bounds);

// Query nearby objects (radius query)
let nearby = octree.query_radius(position, 50.0);

// Query objects in volume (AABB query)
let in_box = octree.query(&query_bounds);
```

### BVH Example

```rust
use praxis_spatial::{Bvh, Aabb};

let mut bvh = Bvh::new();

// Collect all entities and their bounds
let entities: Vec<(Entity, Aabb)> = /* ... */;

// Build BVH (typically once per frame or when objects move)
bvh.build(entities);

// Query intersections
let results = bvh.query(&query_bounds);
let nearby = bvh.query_radius(point, radius);
```

### Choosing Parameters

**Octree:**
- **Size:** Should encompass all objects with margin
- **Max Depth:** 8-12 levels (deeper = finer subdivision)
- **Max Objects:** 4-16 per node (lower = more subdivision)

**BVH:**
- Automatically balanced during construction
- No manual tuning required
- Rebuild when >10-20% of objects move

## LOD (Level of Detail) System

### What is LOD?

LOD reduces rendering cost by using simpler mesh representations for distant objects. Close objects use high-polygon meshes, distant objects use low-polygon meshes.

### Benefits

- **Polygon Count:** Can reduce by 80-95% for distant objects
- **Draw Calls:** Fewer batches due to simpler geometry
- **Texture Memory:** Lower resolution textures for distant objects
- **Frame Rate:** 2-5x improvement in large outdoor scenes

### Setting Up LOD Groups

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
```

### Using LOD at Runtime

```rust
// Assign entity to LOD group
lod_manager.assign_entity(entity, "tree");

// Each frame, select appropriate LOD based on distance
let selection = lod_manager.select_lod(
    entity,
    camera_position,
    entity_position
);

if let Some(lod) = selection {
    // Render with the selected mesh
    render_mesh(&lod.mesh_id);
    
    // Optional: log LOD transitions
    if lod.level_index != previous_level {
        println!("Entity {} transitioned to LOD {}", entity, lod.level_index);
    }
}
```

### LOD Authoring Guidelines

1. **Create Multiple Levels:** Aim for 3-5 LOD levels per object type
2. **Geometric Ratios:** Each level should be 30-50% the polycount of the previous
3. **Silhouette Preservation:** Maintain overall shape at all levels
4. **Distance Thresholds:** 
   - LOD 0→1: 30-50% of max view distance
   - LOD 1→2: 50-70% of max view distance
   - LOD 2→3: 70-90% of max view distance
5. **Billboard LODs:** Use for very distant objects (trees, rocks, buildings)

### Batch LOD Selection

For better performance, select LODs in batches:

```rust
// Collect all entities with their positions
let entities: Vec<(Entity, Vec3)> = /* query from ECS */;

// Select LODs for all entities at once
let selections = lod_manager.select_lods(&entities, camera_position);

// Process results
for selection in selections {
    render_entity_with_lod(selection.entity, &selection.mesh_id);
}
```

## Occlusion Culling

### What is Occlusion Culling?

Occlusion culling uses the GPU to test if objects are hidden behind other geometry. Fully occluded objects are skipped, saving fragment shader and bandwidth costs.

### How It Works

1. **Render bounding boxes** with occlusion queries
2. **GPU tests** how many samples passed depth test
3. **If samples == 0:** Object is fully occluded, skip rendering
4. **If samples > 0:** Object is visible, render normally

### Implementation

```rust
use praxis_spatial::OcclusionCuller;

// Create culler (once during initialization)
let mut occlusion_culler = OcclusionCuller::new(
    device.clone(),
    queue.clone(),
    allocator.clone(),
    1024, // max concurrent queries
)?;

// Each frame:

// 1. Begin queries for all objects
for (entity, bounds) in potentially_visible_objects {
    let query_id = occlusion_culler.begin_test(
        &mut command_buffer,
        entity,
        bounds
    )?;
    
    // Render bounding box with color writes disabled
    render_bounding_box(&bounds, query_id);
    
    occlusion_culler.end_test(&mut command_buffer, query_id)?;
}

// 2. Submit command buffer and wait
submit_and_wait(command_buffer);

// 3. Retrieve results
occlusion_culler.update_results()?;

// 4. Render visible objects
for entity in entities {
    if occlusion_culler.is_visible(entity) {
        render_entity(entity);
    }
}

// 5. Reset for next frame
occlusion_culler.reset();
```

### Performance Considerations

**Benefits:**
- Reduces fragment shading by 20-60% in dense scenes
- Most effective in urban environments and interiors
- Minimal CPU overhead

**Costs:**
- GPU time for occlusion queries
- Extra draw calls for bounding boxes
- One frame latency (uses previous frame results)

**Best Practices:**
- Use **conservative** bounding boxes (slightly larger than object)
- **Batch queries** - process many objects per frame
- **Temporal coherence** - assume objects stay visible
- **Hierarchical queries** - test large groups before individuals
- **Combine with frustum culling** - only query frustum-visible objects

## Unified Visibility System

The `VisibilitySystem` combines all optimization techniques:

```rust
use praxis_spatial::{VisibilitySystem, Aabb};

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
println!("Frustum culled: {}", stats.frustum_culled);
println!("Distance culled: {}", stats.distance_culled);
```

## Integration with ECS

### Adding Components

```rust
use praxis_ecs::{World, Transform, BoundingBox, LodComponent, MeshHandle};
use praxis_math::Vec3;

let mut world = World::new();

// Spawn entity with spatial components
world.spawn((
    Transform::from_xyz(10.0, 0.0, 5.0),
    MeshHandle::new("tree_high"),
    BoundingBox::from_center_half_extents(Vec3::ZERO, Vec3::splat(3.0)),
    LodComponent::new("tree"),
));
```

### Creating a Culling System

```rust
use praxis_ecs::{Query, Res, ResMut, Transform, BoundingBox, LodComponent};
use praxis_spatial::{VisibilitySystem, Aabb};

fn spatial_culling_system(
    entities_query: Query<(Entity, &Transform, &BoundingBox, Option<&LodComponent>)>,
    mut visibility: ResMut<VisibilitySystem>,
    camera_pos: Res<CameraPosition>,
) {
    // Collect entities for culling
    let entities: Vec<_> = entities_query
        .iter()
        .map(|(entity, transform, bounds, _)| {
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

## Performance Tuning

### Benchmarking

```rust
use std::time::Instant;

let start = Instant::now();

// Frustum culling
let frustum_visible = frustum_culler.is_visible(&bounds);

let frustum_time = start.elapsed();

// Octree query
let octree_results = octree.query(&bounds);

let octree_time = start.elapsed() - frustum_time;

println!("Frustum culling: {:?}", frustum_time);
println!("Octree query: {:?}", octree_time);
```

### Optimization Checklist

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

## Debugging

### Visualization

```rust
// Visualize frustum planes
for plane in frustum.planes.iter() {
    debug_draw_plane(plane);
}

// Visualize octree bounds
debug_draw_octree_bounds(&octree);

// Visualize LOD transitions
for entity in entities {
    let color = lod_level_to_color(get_lod_level(entity));
    debug_draw_bounds(entity, color);
}

// Show occlusion query results
for entity in entities {
    if occlusion_culler.is_visible(entity) {
        debug_draw_bounds(entity, GREEN);
    } else {
        debug_draw_bounds(entity, RED);
    }
}
```

### Statistics

```rust
println!("=== Spatial Optimization Stats ===");
println!("Total objects: {}", stats.total_objects);
println!("Visible: {} ({:.1}%)", 
    stats.visible_objects,
    (stats.visible_objects as f32 / stats.total_objects as f32) * 100.0
);
println!("Frustum culled: {} ({:.1}%)",
    stats.frustum_culled,
    (stats.frustum_culled as f32 / stats.total_objects as f32) * 100.0
);
println!("Occlusion culled: {} ({:.1}%)",
    stats.occlusion_culled,
    (stats.occlusion_culled as f32 / stats.total_objects as f32) * 100.0
);
println!("Distance culled: {} ({:.1}%)",
    stats.distance_culled,
    (stats.distance_culled as f32 / stats.total_objects as f32) * 100.0
);
```

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

### Not Using Temporal Coherence
**Problem:** Query latency causes stutter  
**Solution:** Assume objects visible from previous frame

## Further Reading

- [Frustum Culling in Detail](https://www.lighthouse3d.com/tutorials/view-frustum-culling/)
- [Octree Spatial Partitioning](https://en.wikipedia.org/wiki/Octree)
- [BVH Construction Algorithms](https://jcgt.org/published/0004/03/02/)
- [LOD Best Practices (GDC)](https://www.gdcvault.com/play/1020451/Level-of-Detail)
- [GPU Occlusion Queries](https://www.khronos.org/opengl/wiki/Query_Object#Occlusion_queries)

## Examples

See `examples/spatial_optimization_demo.rs` for a complete working example demonstrating all techniques.
