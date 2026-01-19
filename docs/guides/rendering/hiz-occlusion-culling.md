# Hi-Z Occlusion Culling

Hi-Z (Hierarchical Z-buffer) occlusion culling is an advanced GPU-driven technique that eliminates objects hidden behind other geometry. Combined with frustum culling, it can achieve 30-50% additional culling in dense scenes with significant occlusion.

## Overview

Traditional frustum culling removes objects outside the camera view, but many objects inside the frustum may be completely hidden behind other geometry (occluded). Rendering these occluded objects wastes GPU fill rate and fragment shader time.

Hi-Z occlusion culling uses a hierarchical depth buffer (mipmap pyramid) from the previous frame to test object visibility before rendering. This allows the GPU to skip occluded objects entirely.

### Performance Benefits

**Without Hi-Z Occlusion Culling:**
- Frustum culling: 70-90% of objects culled
- Remaining objects all rendered (many occluded)
- High overdraw from hidden geometry
- Wasted fragment shader time

**With Hi-Z Occlusion Culling:**
- Frustum culling: 70-90% of objects culled
- Occlusion culling: 30-50% additional culling
- Only visible objects rendered
- Minimal overdraw

**Example Scene (10,000 objects):**
- Frustum culling only: 3,000 objects rendered (7,000 culled)
- + Hi-Z occlusion: 1,500 objects rendered (8,500 culled)
- Result: 50% reduction in draw calls and fragment workload

## Architecture

### Pipeline Overview

```text
Frame N-1:
  1. Render scene to depth buffer
  2. Generate Hi-Z pyramid (depth mipmap chain)
  3. Store for next frame

Frame N:
  1. Frustum culling (compute shader)
     - Test bounding volumes against frustum planes
  2. Occlusion culling (compute shader)
     - Project bounding volumes to screen space
     - Sample Hi-Z pyramid at appropriate mip level
     - Compare object depth with sampled Hi-Z depth
  3. Generate indirect draw buffer (GPU)
  4. Render visible objects (graphics pipeline)
```

### Hi-Z Pyramid Generation

The Hi-Z pyramid is a mipmap chain where each level contains the **maximum** depth values from the previous level. This creates a conservative depth hierarchy:

```text
Level 0 (Base): 1920×1080 - Original depth buffer
Level 1:        960×540   - Max of 2×2 blocks
Level 2:        480×270   - Max of 2×2 blocks
Level 3:        240×135   - Max of 2×2 blocks
...
Level N:        1×1       - Single max depth value
```

**Why maximum depth?**
Using maximum depth ensures conservative culling. An object is only culled if it's behind the **farthest** visible point in its screen-space region. This prevents false culling of visible objects.

### Occlusion Test Algorithm

For each object (after passing frustum culling):

1. **Project to screen space:**
   - Transform bounding sphere center to clip space
   - Convert to NDC coordinates [-1, 1]
   - Convert to screen UV coordinates [0, 1]

2. **Calculate screen-space bounding box:**
   - Project sphere radius to screen space
   - Calculate AABB in UV coordinates
   - Clamp to [0, 1] range

3. **Select appropriate mip level:**
   ```glsl
   vec2 pixel_size = aabb_size * hiz_resolution;
   float mip_level = max(0.0, log2(max_dimension) - 1.0);
   ```
   This ensures we sample a mip level where the object covers ~2×2 pixels

4. **Sample Hi-Z pyramid:**
   - Sample at 5 points: 4 corners + center
   - Take maximum depth (conservative)

5. **Depth comparison:**
   ```glsl
   float object_depth = closest_point_depth;
   float hiz_depth = max(all_sampled_depths);
   
   if (object_depth <= hiz_depth + BIAS) {
       // Object is in front of Hi-Z depth → visible
   } else {
       // Object is behind Hi-Z depth → occluded
   }
   ```

## Implementation

### Basic Setup

```rust
use praxis_graphics::gpu_culling::{
    GpuCullingManager, GpuDrawCommand, GpuMeshData, extract_frustum_planes
};

// 1. Create GPU culling manager
let mut culling_manager = GpuCullingManager::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
)?;

// 2. Initialize Hi-Z pyramid (once or on window resize)
culling_manager.initialize_hiz_pyramid([window_width, window_height])?;

// 3. Enable occlusion culling
culling_manager.set_occlusion_culling(true);

info!("Hi-Z occlusion culling initialized");
```

### Per-Frame Usage

```rust
// Each frame:

// 1. Render scene to depth buffer (geometry pass)
render_scene_geometry(&mut command_buffer)?;

// 2. Generate Hi-Z pyramid from depth buffer
culling_manager.generate_hiz_pyramid(
    &mut command_buffer,
    depth_image_view.clone(),
)?;

// 3. Prepare draw commands with bounding spheres
let draw_commands: Vec<GpuDrawCommand> = objects
    .iter()
    .map(|obj| {
        GpuDrawCommand::new(
            obj.model_matrix,
            obj.bounding_sphere, // Vec4(center.xyz, radius)
            obj.mesh_id,
            obj.material_id,
        )
    })
    .collect();

let mesh_data: Vec<GpuMeshData> = meshes
    .iter()
    .map(|mesh| GpuMeshData {
        index_count: mesh.index_count,
        first_index: mesh.first_index,
        vertex_offset: mesh.vertex_offset,
        _padding: 0,
    })
    .collect();

// 4. Upload to GPU
culling_manager.prepare_frame(&draw_commands, &mesh_data)?;

// 5. Dispatch culling compute shader (frustum + occlusion)
let view_proj = projection * view;
let frustum_planes = extract_frustum_planes(view_proj);

culling_manager.dispatch_culling(
    &mut command_buffer,
    view_proj,
    frustum_planes,
    camera_position,
)?;

// 6. Render visible objects using indirect draw buffer
let indirect_buffer = culling_manager.indirect_draw_buffer().unwrap();
let draw_count_buffer = culling_manager.draw_count_buffer().unwrap();

command_buffer.draw_indexed_indirect_count(
    indirect_buffer.clone(),
    0,
    draw_count_buffer.clone(),
    0,
    draw_commands.len() as u32,
)?;
```

### Toggle Occlusion Culling

For debugging or performance comparison:

```rust
// Disable occlusion culling (frustum culling only)
culling_manager.set_occlusion_culling(false);

// Enable occlusion culling
culling_manager.set_occlusion_culling(true);

// Check if enabled
if culling_manager.is_occlusion_culling_enabled() {
    println!("Occlusion culling is active");
}
```

### Read Back Statistics

For debugging and profiling:

```rust
// Read visible count (CPU-GPU sync required)
let visible_count = culling_manager.read_visible_count()?;
let total_count = draw_commands.len();

println!("Visible: {} / {}", visible_count, total_count);
println!("Culled: {} ({:.1}%)", 
    total_count - visible_count,
    100.0 * (1.0 - visible_count as f32 / total_count as f32)
);
```

## Bounding Volumes

### Calculating Bounding Spheres

Hi-Z occlusion culling uses **bounding spheres** for efficiency:

**From mesh vertices:**
```rust
fn calculate_bounding_sphere(vertices: &[Vec3]) -> (Vec3, f32) {
    // Calculate centroid
    let center = vertices.iter().sum::<Vec3>() / vertices.len() as f32;
    
    // Find maximum distance from centroid
    let radius = vertices
        .iter()
        .map(|v| (*v - center).length())
        .fold(0.0f32, f32::max);
    
    (center, radius)
}
```

**From AABB:**
```rust
fn aabb_to_bounding_sphere(aabb_min: Vec3, aabb_max: Vec3) -> (Vec3, f32) {
    let center = (aabb_min + aabb_max) * 0.5;
    let radius = (aabb_max - center).length();
    (center, radius)
}
```

**Transform to world space:**
```rust
fn transform_bounding_sphere(
    local_center: Vec3,
    local_radius: f32,
    model_matrix: Mat4,
) -> (Vec3, f32) {
    // Transform center
    let world_center = (model_matrix * Vec4::new(
        local_center.x,
        local_center.y,
        local_center.z,
        1.0,
    )).xyz();
    
    // Scale radius by maximum scale factor (conservative)
    let scale_x = model_matrix.x_axis.xyz().length();
    let scale_y = model_matrix.y_axis.xyz().length();
    let scale_z = model_matrix.z_axis.xyz().length();
    let max_scale = scale_x.max(scale_y).max(scale_z);
    let world_radius = local_radius * max_scale;
    
    (world_center, world_radius)
}
```

## Performance Optimization

### 1. Scene Characteristics

Hi-Z occlusion culling provides the most benefit in scenes with:

**High occlusion:**
- Dense urban environments (buildings blocking buildings)
- Forests with overlapping foliage
- Indoor scenes with walls and rooms
- Terrain with hills and valleys

**Large occluders:**
- Buildings, walls, large rocks
- Terrain features
- Any large opaque geometry

**Many small objects:**
- Vegetation (trees, bushes, grass)
- Props and details
- Particle effects
- Decals

### 2. Mip Level Selection

The compute shader automatically selects the appropriate mip level:

```glsl
// Screen-space size of object
vec2 pixel_size = aabb_size * hiz_resolution;

// Select mip level where object covers ~2×2 pixels
float mip_level = max(0.0, log2(max(pixel_size.x, pixel_size.y)) - 1.0);
```

**Why this matters:**
- Small objects sample higher mip levels (coarser, faster)
- Large objects sample lower mip levels (more accurate)
- Balances accuracy and performance

### 3. Temporal Coherence

Hi-Z uses the **previous frame's** depth buffer:

**Benefits:**
- No circular dependency (can generate Hi-Z and use it in same frame)
- Leverages temporal coherence (objects don't suddenly appear/disappear)

**Limitations:**
- Fast-moving objects may cause 1-frame lag
- Newly visible areas may show brief pop-in

**Solutions:**
- Add depth bias to be conservative
- Use velocity-based prediction for fast objects
- Accept 1-frame lag (generally imperceptible at 60+ FPS)

### 4. Conservative Culling

The system uses several techniques to prevent false culling:

1. **Maximum depth sampling**: Takes max of 5 samples (corners + center)
2. **Depth bias**: Adds small epsilon to prevent precision issues
3. **Sphere vs. box**: Uses bounding spheres (more conservative than AABB)
4. **Mip selection**: Samples appropriate detail level

## Visual Verification

The `hiz_occlusion_demo` example provides tools for manual verification:

```bash
cargo run --example hiz_occlusion_demo
```

### Verification Tests

**1. Toggle occlusion culling (O key):**
- With culling OFF: All objects render (lower FPS)
- With culling ON: Occluded objects culled (higher FPS)
- Expected: 30-50% FPS improvement in test scene

**2. Wireframe mode (P key):**
- Enable wireframe rendering
- Move camera behind occluder walls
- Verify: Objects behind walls are NOT rendered
- Verify: Visible objects ARE rendered

**3. Camera presets (1-5 keys):**
- Preset 1 (Front): See occluders from front
- Preset 2 (Side): See occluders from side
- Preset 3 (Behind): See all occluded objects
- Preset 4 (Top): See scene layout from above
- Preset 5 (Edge): Test partial occlusion

**4. Free camera movement:**
- Move around scene with WASD
- Verify: No visible objects are falsely culled
- Verify: Occluded objects don't pop in/out unexpectedly

**5. Statistics (I key):**
- Print visible/culled object counts
- Compare FPS with occlusion ON vs OFF
- Verify culling percentage is reasonable

### Example Output

```
=== Scene Statistics ===
  Total objects: 1506
  Visible objects: 745
  Culled objects: 761
  Culling percentage: 50.5%
  Occlusion culling: ENABLED
  Average FPS: 127.3 (7.85 ms/frame)
```

With occlusion disabled:
```
  Total objects: 1506
  Visible objects: 1506
  Culled objects: 0
  Culling percentage: 0.0%
  Occlusion culling: DISABLED
  Average FPS: 78.2 (12.78 ms/frame)
```

## Advanced Topics

### Hybrid Culling Strategy

Combine spatial structures with Hi-Z occlusion:

```rust
// 1. Coarse culling with octree (CPU)
let potentially_visible = octree.query_frustum(&frustum);

// 2. Fine-grained frustum culling (GPU)
let gpu_commands: Vec<_> = potentially_visible
    .iter()
    .map(|entity| create_gpu_draw_command(entity))
    .collect();

// 3. Hi-Z occlusion culling (GPU)
culling_manager.prepare_frame(&gpu_commands, &mesh_data)?;
culling_manager.dispatch_culling(cmd_buffer, view_proj, frustum, camera_pos)?;

// Result: Minimal CPU overhead, maximum GPU parallelism
```

### Two-Phase Occlusion

For very dense scenes, use two-phase culling:

```rust
// Phase 1: Cull small objects against Hi-Z
cull_small_objects(&mut culling_manager, &small_objects)?;

// Phase 2: Render large objects (potential occluders)
render_large_objects(&large_objects);

// Phase 3: Update Hi-Z with new occluders
culling_manager.generate_hiz_pyramid(cmd_buffer, depth_image)?;

// Phase 4: Re-cull with updated Hi-Z (optional)
cull_remaining_objects(&mut culling_manager, &remaining_objects)?;
```

### LOD + Occlusion Integration

Combine LOD selection with occlusion culling:

```rust
// Objects far away are more likely to be occluded
// Use LOD to reduce geometry complexity before occlusion test

for object in objects {
    let distance = (object.position - camera_position).length();
    
    // Select LOD based on distance
    let lod_level = select_lod_level(distance);
    
    // Use LOD's bounding sphere for occlusion test
    let bounding_sphere = lod_meshes[lod_level].bounding_sphere;
    
    draw_commands.push(GpuDrawCommand::new(
        object.model,
        bounding_sphere,
        lod_meshes[lod_level].mesh_id,
        object.material_id,
    ));
}
```

## Troubleshooting

### Issue: No Performance Improvement

**Symptoms:** Occlusion culling enabled but FPS unchanged

**Solutions:**
1. Check scene has significant occlusion (needs large occluders)
2. Verify many objects are actually behind occluders
3. Ensure depth buffer is being generated correctly
4. Verify Hi-Z pyramid is being updated each frame
5. Check if scene is bottlenecked elsewhere (not by overdraw)

### Issue: False Culling (Pop-in)

**Symptoms:** Visible objects disappear incorrectly

**Solutions:**
1. Increase depth bias in shader (add more epsilon)
2. Check bounding sphere is large enough (includes all geometry)
3. Verify depth buffer format is sufficient (D32_SFLOAT recommended)
4. Ensure Hi-Z pyramid generation is conservative (uses max depth)
5. Check for floating-point precision issues

### Issue: Objects Popping In When Camera Moves

**Symptoms:** Objects appear suddenly when moving camera

**Solutions:**
1. This is expected with temporal Hi-Z (uses previous frame)
2. Acceptable at 60+ FPS (1-frame lag imperceptible)
3. Add velocity-based prediction for fast camera movement
4. Reduce depth bias if it's too conservative
5. Consider using 2-frame history for more stability

### Issue: Low Culling Percentage

**Symptoms:** Only 10-20% additional culling (expected 30-50%)

**Solutions:**
1. Check scene has large occluders blocking view
2. Verify objects are positioned behind occluders
3. Ensure Hi-Z pyramid has correct resolution
4. Check mip level selection is appropriate
5. Verify depth buffer is being properly generated

## Performance Metrics

### Overhead

**Hi-Z Generation:**
- ~0.5-1.0ms for 1920×1080 depth buffer
- Scales with resolution (4K: ~1.5-2.0ms)
- One-time cost per frame

**Occlusion Testing:**
- ~0.1-0.2ms for 10,000 objects
- Parallelized across GPU cores
- Scales with object count

**Total Overhead:**
- ~0.6-1.2ms per frame
- Amortized across all objects

**Benefit:**
- Eliminates 30-50% of objects
- Saves 5-10ms+ in fragment shading
- Net gain: 4-9ms per frame (30-50% FPS boost)

### Memory Usage

| Component | Size (1080p) | Size (4K) |
|-----------|-------------|-----------|
| Hi-Z Pyramid | ~8 MB | ~32 MB |
| Depth Buffer | ~8 MB | ~32 MB |
| Draw Commands | ~96 bytes × object_count | ~96 bytes × object_count |
| **Total** | ~16 MB + draw cmds | ~64 MB + draw cmds |

## References

- **[Hi-Z Occlusion Culling (GPU Gems 2)](https://developer.nvidia.com/gpugems/gpugems2/part-i-geometric-complexity/chapter-6-hardware-occlusion-queries-made-useful)**
- **[Practical Occlusion Culling (SIGGRAPH 2013)](http://twvideo01.ubm-us.net/o1/vault/gdc2013/slides/822403Pranckeviciene_Paulius_Occlusion_Culling.pdf)**
- **[GPU-Driven Rendering Pipelines (SIGGRAPH 2015)](https://advances.realtimerendering.com/s2015/aaltonenhaar_siggraph2015_combined_final_footer_220dpi.pdf)**
- **[Hierarchical Z-Buffer Visibility (SIGGRAPH 1993)](http://www.cs.cmu.edu/~garth/sig93.pdf)**

## Examples

### Basic Example

See `examples/hiz_occlusion_demo.rs` for a complete demonstration:

```bash
cargo run --example hiz_occlusion_demo
```

Features:
- Large occluder walls
- Grid of objects behind occluders
- Toggle occlusion culling on/off
- Wireframe mode for verification
- Preset camera positions
- Real-time statistics

### Integration Example

See `examples/gpu_culling_demo.rs` for GPU culling without occlusion:

```bash
cargo run --example gpu_culling_demo
```

Compare performance with and without occlusion culling.

## Summary

Hi-Z occlusion culling is a powerful optimization for scenes with significant occlusion. When properly implemented, it can provide 30-50% additional culling beyond frustum culling, leading to substantial FPS improvements in dense scenes.

**Key Takeaways:**
- Use for scenes with large occluders blocking many small objects
- Requires previous frame's depth buffer (1-frame lag)
- Conservative by design (prevents false culling)
- Overhead: ~1ms, benefit: 5-10ms+ (net positive)
- Best combined with frustum culling and spatial structures
- Verify visually with the demo example
