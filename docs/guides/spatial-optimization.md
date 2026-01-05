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

## Design Rationale and Tradeoffs

### Why Multiple Optimization Techniques?

**Decision**: Provide frustum culling, spatial partitioning, LOD, and occlusion culling as complementary systems

**Rationale**: No single technique solves all performance problems
- Different scene types have different bottlenecks
- Techniques address different parts of rendering pipeline (CPU vs GPU)
- Combining techniques yields multiplicative benefits

**Performance Stack**:
```
Raw scene: 100k triangles × 1M pixels = 100B fragment operations

After frustum culling (60% culled): 40k triangles
After LOD (80% reduction): 8k triangles  
After occlusion culling (30% culled): 5.6k triangles

Final: 5.6B fragment operations (18x improvement)
```

**Alternatives Considered**:
1. **Single "do everything" system**: Too complex, inflexible, one-size-fits-none
2. **Only frustum culling**: Insufficient for large worlds
3. **Only LOD**: Doesn't help with off-screen objects
4. **GPU-only culling**: CPU still needs to know what to submit

### Frustum Culling Architecture

**Decision**: Plane-based frustum representation with AABB intersection tests

**Core Algorithm**:
1. Extract 6 planes from view-projection matrix
2. Test each object's AABB against all planes
3. Object is visible if inside all planes (or intersecting)

**Why 6 Planes?**
- Near, far, left, right, top, bottom
- Plane equation: `dot(normal, point) + distance = 0`
- AABB test: Check if box is on positive side of plane

**Why Not Alternatives?**

| Alternative | Issue | Why Not Used |
|-------------|-------|--------------|
| **Ray tracing per object** | O(n²) complexity | Too slow |
| **Portal-based** | Requires level design cooperation | Not general purpose |
| **PVS (potential visibility set)** | Precomputed, no dynamic objects | Inflexible |
| **Image-based culling** | GPU readback latency | Too slow |

**Performance Characteristics**:
- Test cost: 10-20 CPU cycles per object
- Typical cull rate: 40-70% of objects
- Memory: 24 floats (96 bytes) for frustum
- Scales: O(n) with object count

**Tradeoff**: Conservative test (false positives okay, false negatives not)
- May render some partially visible objects as fully visible
- Acceptable: Better to render a few extra objects than miss visible ones

**Optimization: Early-out**
```rust
// If bounding sphere is entirely outside any plane, reject
if sphere_distance_to_plane(plane) < -radius {
    return false;  // Definitely outside
}
```

### Spatial Partitioning: Octree vs BVH

**Decision**: Provide both octree and BVH, let users choose

**Why Both?**
- Different performance characteristics for different scenes
- Different use cases (static vs dynamic)
- Educational value: Learn both classic data structures

#### Octree Design

**Structure**: Recursive subdivision into 8 equal octants

**Why Octree?**
- **Uniform subdivision**: Simple, predictable structure
- **Static geometry**: Excellent for worlds that don't change
- **Space queries**: Natural for "find nearby" operations
- **Implementation**: Straightforward recursion

**Parameters**:
- `max_depth`: 8-12 levels (deeper = finer subdivision)
- `max_objects_per_node`: 4-16 (lower = more subdivision)

**Why These Defaults?**
- 8 levels = 256×256×256 smallest cells (sufficient granularity)
- 8 objects/node balances memory vs query speed
- Prevents pathological cases (too deep or too flat)

**Performance**:
- Insert: O(log n) average
- Query: O(log n + k) where k = results
- Memory: ~80 bytes per node + 8 bytes per object reference

**Best For**:
- Static level geometry
- Uniform object distribution (grids, forests)
- Radius queries (find all within distance)

**Tradeoffs**:
- ✓ Simple to implement and debug
- ✓ Predictable performance
- ✗ Wastes memory in sparse regions (empty nodes)
- ✗ Expensive to rebuild for dynamic objects
- ✗ Assumes cubical world (anisotropic scenes inefficient)

#### BVH Design

**Structure**: Binary tree where each node has bounding volume of children

**Why BVH?**
- **Object-centric**: Adapts to object distribution
- **Dynamic scenes**: Efficient incremental updates
- **Ray tracing**: Optimal for ray intersection tests
- **Non-uniform distribution**: Handles clustered objects well

**Construction**: Surface Area Heuristic (SAH)
```rust
cost(split) = cost_traverse + 
              (left_area / parent_area) × left_cost +
              (right_area / parent_area) × right_cost
```

**Why SAH?**
- Minimizes expected ray traversal cost
- Creates balanced trees for diverse object sets
- Industry standard for ray tracing

**Performance**:
- Build: O(n log n) with SAH
- Query: O(log n + k) where k = results
- Memory: ~64 bytes per node (tighter packing than octree)

**Best For**:
- Dynamic objects (characters, vehicles)
- Ray tracing and picking
- Clustered distributions (cities, interiors)
- Nearest-neighbor queries

**Tradeoffs**:
- ✓ Adapts to object distribution
- ✓ Efficient for ray queries
- ✓ Good cache locality (tight AABBs)
- ✗ More complex to implement correctly
- ✗ Rebuild cost higher than octree insert
- ✗ Tree can become unbalanced without maintenance

**Decision Matrix**:
```
Scene Type           | Static | Dynamic | Recommendation
---------------------|--------|---------|---------------
Indoor level         |   ✓    |         | Octree or BVH
Open world terrain   |   ✓    |         | Octree
City with traffic    |   ✓    |    ✓    | Octree (static) + BVH (dynamic)
Space sim (sparse)   |        |    ✓    | BVH
RTS with 1000 units  |        |    ✓    | BVH
```

### LOD System Design

**Decision**: Distance-based mesh selection with discrete LOD levels

**Why Distance-Based?**
- Simple: Single float comparison per object
- Predictable: Artists set exact transition distances
- Cheap: No complex visibility metrics needed

**Why Discrete Levels?**
- Artist controlled: Specific mesh for each LOD
- GPU efficient: No runtime simplification
- Memory predictable: Know exact count of meshes

**Alternatives Considered**:

| Approach | Pros | Cons | Why Not Used |
|----------|------|------|--------------|
| **Continuous LOD** | Smooth transitions | Complex, GPU cost | Runtime simplification too slow |
| **Screen-space error** | More accurate | Expensive metric | Simple distance 90% as good |
| **Geometric error** | Mathematically sound | Hard to tune | Artists prefer distance |
| **View-dependent** | Optimal quality | Complex implementation | Diminishing returns |

**LOD Level Guidelines**:
1. **LOD 0** (0-30% distance): Full detail, hero quality
2. **LOD 1** (30-50% distance): ~50% polygons, noticeable only on close inspection  
3. **LOD 2** (50-75% distance): ~25% polygons, silhouette preserved
4. **LOD 3** (75-90% distance): ~10% polygons, simplified shapes
5. **Billboard** (90-100% distance): 2 triangles, textured quad

**Why 4-5 Levels?**
- More levels = more artist work, minimal benefit
- Fewer levels = visible popping artifacts
- 4-5 is sweet spot for quality vs effort

**Transition Handling**:
- **Current**: Instant swap (pop)
- **Future**: Cross-fade over distance range
- **Why instant for now**: Simple, works for most cases

**Tradeoff**: Popping artifacts vs implementation complexity
- Mitigation: Tune distances to pop during movement/low attention

**Performance Impact**:
```
Scene: 1000 trees, 5000 tris each (5M tris total)

Without LOD: 5M tris
With LOD (25% at LOD0, 50% at LOD1, 25% at LOD2):
  250 × 5000 + 500 × 2500 + 250 × 1250 = 2.8M tris
  
Savings: 44% polygon reduction
Frame time: 16ms → 10ms (37% faster)
```

### Occlusion Culling Design

**Decision**: Hardware occlusion queries with temporal coherence

**Why Hardware Queries?**
- GPU knows exactly what's visible (post-depth-test)
- Accurate: No false positives/negatives
- Parallel: All queries execute simultaneously

**Algorithm**:
1. Render bounding boxes with occlusion query
2. Disable color writes, enable depth writes
3. GPU counts fragments that pass depth test
4. If count > 0, object is visible
5. Conditionally render full object

**Why This Approach?**

| Alternative | Issue |
|-------------|-------|
| **Software rasterization** | Too slow, inaccurate |
| **CPU ray tracing** | O(n²) complexity |
| **Precomputed PVS** | No dynamic objects/occluders |
| **Image-based** | GPU-CPU sync latency |

**Temporal Coherence Optimization**:
```rust
// Use previous frame's visibility for current frame
if was_visible_last_frame {
    render_full_object();
    start_occlusion_query();  // Async check for next frame
} else {
    render_bounding_box_only();
    start_occlusion_query();
}
```

**Why Previous Frame Results?**
- Avoids GPU-CPU sync stall (1-2ms latency)
- Objects rarely change visibility frame-to-frame
- False positive (render one extra frame) is cheap
- False negative (skip one frame) is rare

**When Occlusion Culling Helps**:
- Dense urban environments (buildings occlude each other)
- Indoor scenes (walls, rooms)
- Large occluders (terrain, mountains)

**When It Doesn't Help**:
- Open fields with sparse objects (nothing to occlude)
- Transparent scenes (everything visible)
- Skyboxes and distant scenery

**Performance Characteristics**:
- Query cost: ~5-10μs per object (GPU time)
- Typical cull rate: 20-60% in dense scenes
- Best case: 30-40ms saved per frame (city scene)
- Worst case: 1-2ms overhead (open scene)

**Tradeoff**: Query overhead vs fragment savings
- Rule of thumb: Only query medium/large objects (>100 triangles)
- Small objects: Query cost exceeds render cost

### Unified Visibility System Design

**Decision**: Combine all techniques in single `VisibilitySystem` API

**Why Unified?**
- Simplifies user code (one API instead of four)
- Ensures correct order (frustum → LOD → occlusion)
- Shares data structures (one AABB per object)
- Batches operations for cache efficiency

**Pipeline Order**:
```rust
1. Frustum culling (cheapest, highest cull rate)
   ↓ 40% remain
2. Distance culling (trivial cost, removes far objects)
   ↓ 30% remain
3. LOD selection (cheap, improves GPU cost)
   ↓ 30% remain at reduced detail
4. Occlusion culling (expensive, further reduces GPU cost)
   ↓ 20% remain
```

**Why This Order?**
- Frustum first: Cheapest test, highest rejection rate
- Distance second: Also cheap, removes far objects
- LOD third: Operates on survivors, no rejection
- Occlusion last: Most expensive, only on survivors

**Alternative Order (rejected)**:
```
Occlusion → Frustum → LOD
Problem: Waste GPU queries on off-screen objects
```

**Batching Benefits**:
- Single loop over all objects
- Cache-friendly: Sequential AABB access
- SIMD potential: Test multiple AABBs at once (future)

### Spatial Manager Design

**Decision**: Movement threshold tracking with incremental updates

**Problem**: Rebuilding spatial structures every frame is expensive
- Octree rebuild: O(n log n)
- BVH rebuild: O(n log n)

**Solution**: Track "dirty" entities, batch updates
```rust
if entity_moved_distance > threshold {
    mark_dirty(entity);
}

// Later (e.g., every N frames)
for entity in dirty_entities {
    spatial_structure.remove(entity);
    spatial_structure.insert(entity, new_bounds);
}
```

**Why Movement Threshold?**
- Avoids thrashing from small movements (jitter, float precision)
- Typical threshold: 0.5 units (half a grid cell)
- Adjustable based on scene scale

**Rebalance Strategy**:
```rust
if frame_count % rebalance_interval == 0 {
    if dirty_entities.len() > 0.1 × total_entities {
        rebuild_structure();  // More than 10% changed
    }
}
```

**Why 10% Threshold?**
- Less than 10%: Incremental updates cheaper
- More than 10%: Full rebuild amortizes better
- Tunable based on profiling

**Performance**:
- Incremental update: O(k log n) for k dirty entities
- Full rebuild: O(n log n) for n total entities
- Break-even: k = 0.1n (typically)

### ECS Integration Philosophy

**Decision**: Spatial components are optional, system-driven

**Why Optional Components?**
- Not all entities need spatial optimization (UI, audio sources)
- Explicit opt-in reduces overhead
- Clear which entities participate in spatial queries

**Component Design**:
```rust
#[derive(Component)]
struct SpatialTracking {
    last_position: Vec3,
    bounds: Aabb,
}
```

**System Ordering**:
```
transform_propagation → update_spatial_entities → flush_spatial_updates
```

**Why This Order?**
- Transform propagation: Calculate GlobalTransform
- Update spatial: Detect moved entities
- Flush: Apply changes to spatial structure

**Alternative (rejected)**: Automatic tracking via Transform changes
- Problem: Change detection has overhead for ALL entities
- Better: Explicit component for entities that need it

### Performance Profiling Integration

**Decision**: Built-in statistics tracking for all culling operations

**Statistics Collected**:
- Total objects
- Frustum culled
- Distance culled  
- LOD selections per level
- Occlusion culled
- Final rendered objects
- Time spent in each stage

**Why Built-in?**
- Users need data to tune parameters
- Overhead is negligible (<1μs per frame)
- Enables runtime adaptation (future)

**Example Output**:
```
Spatial Optimization Statistics:
  Total objects: 10,000
  Frustum culled: 6,000 (60%)
  Distance culled: 1,500 (15%)
  Occlusion culled: 500 (5%)
  Rendered: 2,000 (20%)
  
  LOD distribution:
    LOD 0: 500 objects (25%)
    LOD 1: 800 objects (40%)
    LOD 2: 700 objects (35%)
```

**Tradeoff**: Tiny overhead for invaluable data
- Overhead: ~0.5μs per frame (negligible)
- Benefit: Can identify bottlenecks and tune

### Future Optimizations

**Planned Enhancements**:

1. **GPU culling**: Compute shader culling
   - Benefit: Massively parallel (10-100x faster)
   - Challenge: Indirect draw integration
   
2. **Hierarchical Z-buffer**: Mipmap-based occlusion
   - Benefit: No per-object queries
   - Challenge: Conservative rasterization
   
3. **Clustered culling**: Tile-based light culling
   - Benefit: Scales to 1000+ lights
   - Challenge: Complex implementation
   
4. **Temporal reprojection**: Reuse previous frame
   - Benefit: ~50% fewer culling tests
   - Challenge: Handle disocclusion

**Why Not Now?**
- Current implementation handles target scales (indie games)
- These optimizations add significant complexity
- Can profile and add incrementally based on need
- Focus on correctness and API stability first

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
