# Extended GPU Culling System

The Praxis engine now supports multiple GPU culling strategies that work together to maximize rendering performance in large scenes.

## Overview

The GPU culling system offloads visibility testing from CPU to GPU using compute shaders, eliminating unnecessary draw calls before they enter the graphics pipeline. This is essential for scenes with 10,000+ objects where CPU culling becomes a bottleneck.

## Supported Culling Strategies

### 1. Frustum Culling
Tests bounding spheres against the camera's view frustum. Objects outside the frustum are culled.

**Performance**: ~0.1ms for 10,000 objects  
**Cull Rate**: 70-90% in typical scenes

### 2. Back-Face Culling (New)
Culls entire objects facing away from the camera based on their average normal direction. Unlike rasterizer back-face culling which operates on triangles, this eliminates entire objects before they enter the graphics pipeline.

**Use Cases**:
- Terrain patches
- Vegetation billboards
- One-sided geometry (walls, floors)

**Performance**: Negligible overhead (~0.01ms)  
**Cull Rate**: 10-30% additional culling for directional objects

**Configuration**:
```rust
// Calculate average normal from mesh vertices
let normals: Vec<Vec3> = mesh.vertices.iter().map(|v| v.normal).collect();
let avg_normal = calculate_average_normal(&normals);

// Create draw command with back-face culling
let cmd = GpuDrawCommand::new_with_culling_params(
    model_matrix,
    bounding_sphere,
    avg_normal,
    0.0, // threshold: 0.0 = cull when facing away, -0.1 = small tolerance
    mesh_id,
    material_id,
    min_screen_size,
    max_render_distance,
);
```

### 3. Small Object Culling (New)
Eliminates objects that project to fewer pixels than a configurable threshold. Prevents wasting GPU time on sub-pixel geometry.

**Use Cases**:
- Distant small objects
- LOD complement (eliminate tiny objects entirely)
- Reducing overdraw

**Performance**: ~0.05ms overhead  
**Cull Rate**: 5-15% additional culling for cluttered scenes

**Configuration**:
```rust
// Per-object minimum screen size in pixels
let min_screen_size = 5.0; // Cull if diameter < 5 pixels

// Use 0.0 to disable for specific objects
let important_object_size = 0.0; // Never cull based on size
```

**Recommendations**:
- `1.0-5.0 px`: Aggressive culling (cull very small objects)
- `5.0-10.0 px`: Balanced (cull sub-pixel objects)
- `10.0+ px`: Conservative (only cull tiny objects)
- `0.0 px`: Disabled (always render)

### 4. Distance-Based Culling (New)
Culls objects beyond their configured maximum render distance. Each object can have its own distance limit, allowing different object classes to have different visibility ranges.

**Use Cases**:
- Object class-based visibility (buildings vs props)
- Performance optimization (expensive objects at distance)
- Visual importance (key objects visible longer)

**Performance**: Negligible overhead (~0.01ms)  
**Cull Rate**: 20-40% in large open scenes

**Configuration**:
```rust
// Per-object maximum render distance
let max_distance = 500.0; // Cull if distance > 500m

// Use negative value to disable for specific objects
let important_distance = -1.0; // Never cull based on distance
```

**Predefined Object Classes**:
```rust
use praxis_graphics::gpu_culling::ObjectClassConfig;

// Large static objects (buildings, terrain)
ObjectClassConfig::LARGE_STATIC
// - Max Distance: 2000m
// - Min Screen Size: 2px
// - Back-face Culling: Enabled

// Medium objects (trees, vehicles)
ObjectClassConfig::MEDIUM
// - Max Distance: 500m
// - Min Screen Size: 5px
// - Back-face Culling: Enabled

// Small props (rocks, debris)
ObjectClassConfig::SMALL_PROPS
// - Max Distance: 100m
// - Min Screen Size: 8px
// - Back-face Culling: Disabled

// Detail objects (grass, vegetation)
ObjectClassConfig::DETAIL
// - Max Distance: 50m
// - Min Screen Size: 10px
// - Back-face Culling: Disabled

// Important objects (characters, objectives)
ObjectClassConfig::IMPORTANT
// - Max Distance: Unlimited
// - Min Screen Size: None
// - Back-face Culling: Disabled
```

### 5. Occlusion Culling (Existing)
Uses Hi-Z depth pyramid to test objects against the previous frame's depth buffer. Objects hidden behind other geometry are culled.

**Performance**: ~0.5-1ms for Hi-Z generation, ~0.1ms for testing  
**Cull Rate**: 10-40% in dense urban scenes with occlusion

## Usage Example

### Basic Setup

```rust
use praxis_graphics::gpu_culling::{
    GpuCullingManager, GpuDrawCommand, calculate_average_normal,
};

// Create culling manager
let mut culling_manager = GpuCullingManager::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
)?;

// Enable desired culling strategies
culling_manager.set_backface_culling(true);
culling_manager.set_small_object_culling(true);
culling_manager.set_distance_culling(true);
```

### Preparing Draw Commands

```rust
let draw_commands: Vec<GpuDrawCommand> = objects.iter().map(|obj| {
    // Get object class configuration
    let config = obj.get_class_config(); // Returns ObjectClassConfig
    
    // Calculate average normal for back-face culling
    let avg_normal = calculate_average_normal(&obj.mesh.normals);
    
    GpuDrawCommand::new_with_culling_params(
        obj.transform.matrix(),
        obj.bounding_sphere,
        avg_normal,
        0.0, // backface_threshold
        obj.mesh_id,
        obj.material_id,
        config.min_screen_size,
        config.max_render_distance,
    )
}).collect();
```

### Per-Frame Culling

```rust
// Prepare frame data
culling_manager.prepare_frame(&draw_commands, &mesh_data)?;

// Dispatch extended culling with all strategies
culling_manager.dispatch_culling_extended(
    &mut cmd_builder,
    view_proj,
    frustum_planes,
    camera_position,
    camera_direction, // For back-face culling
    [screen_width, screen_height], // For small object culling
)?;

// Results are in the indirect draw buffer
// Use with vkCmdDrawIndexedIndirect for rendering
```

## Culling Strategy Order

The GPU shader executes culling tests in this order (most to least effective):

1. **Frustum Culling** - Broadest cull, eliminates ~70-90% of objects
2. **Distance Culling** - Fast distance check, eliminates ~20-40% of remaining
3. **Back-face Culling** - Quick dot product test, eliminates ~10-30% of remaining
4. **Small Object Culling** - Screen-space projection, eliminates ~5-15% of remaining
5. **Occlusion Culling** - Most expensive, tested last on surviving objects

This order maximizes early rejection and minimizes expensive tests.

## Performance Characteristics

### Combined Culling Performance (10,000 objects)

**Without GPU Culling**:
- CPU frustum culling: ~50μs
- Draw command generation: ~100μs
- 1,000 individual draw calls: ~1,000μs
- **Total CPU time: ~1.15ms**

**With Extended GPU Culling**:
- Upload draw commands: ~50μs
- Dispatch compute: ~10μs
- GPU culling (all strategies): ~150μs
- Single indirect draw: ~10μs
- **Total CPU time: ~70μs** (16× faster)
- **GPU time: ~150μs**

**Culling Effectiveness**:
- Frustum only: 70-90% culled
- + Distance: 85-95% culled
- + Back-face: 88-96% culled
- + Small object: 90-97% culled
- + Occlusion: 92-98% culled

## Best Practices

### 1. Object Classification
Group objects into classes with similar culling requirements:
```rust
enum ObjectClass {
    Buildings,     // Large, far distance
    Vegetation,    // Medium distance, back-face cull
    Props,         // Close distance, small size threshold
    Detail,        // Very close, aggressive culling
    Characters,    // Always visible
}
```

### 2. Average Normal Calculation
For static meshes, calculate and cache average normals at load time:
```rust
// Calculate once at mesh load
let avg_normal = calculate_average_normal(&mesh.normals);
obj.cached_avg_normal = avg_normal;

// Use cached value each frame
let cmd = GpuDrawCommand::new_with_culling_params(
    ...,
    obj.cached_avg_normal,
    ...
);
```

### 3. Dynamic Distance Adjustment
Adjust max distances based on performance budget:
```rust
// Lower distances under performance pressure
if fps < target_fps {
    config.max_render_distance *= 0.9;
}

// Raise distances when performance allows
if fps > target_fps + margin {
    config.max_render_distance *= 1.05;
}
```

### 4. Screen Size Tuning
Tune screen size thresholds based on scene density:
```rust
// Dense scenes: aggressive culling
let min_screen_size = 8.0;

// Sparse scenes: conservative culling
let min_screen_size = 2.0;

// Critical objects: no culling
let min_screen_size = 0.0;
```

### 5. Back-face Threshold Selection
Choose thresholds based on object type:
```rust
// Terrain: strict culling (facing down = invisible)
let threshold = 0.0;

// Vegetation: tolerant (visible at angles)
let threshold = -0.2;

// Omnidirectional objects: disabled
let threshold = -1.0;
```

## Debug and Profiling

### Checking Culling Effectiveness
```rust
// Read back visible count (for debugging only)
let visible_count = culling_manager.read_visible_count()?;
let total_count = draw_commands.len();
let cull_rate = ((total_count - visible_count) as f32 / total_count as f32) * 100.0;

println!("Culled: {}/{} ({:.1}%)", 
    total_count - visible_count, 
    total_count, 
    cull_rate
);
```

### Per-Strategy Statistics
To understand which strategies are most effective, disable them individually and measure:
```rust
// Test with all strategies
culling_manager.set_distance_culling(true);
let baseline = measure_frame_time();

// Test without distance culling
culling_manager.set_distance_culling(false);
let without_distance = measure_frame_time();

let distance_impact = baseline - without_distance;
```

## See Also

- `examples/extended_gpu_culling_demo.rs` - Comprehensive demonstration
- `crates/praxis_graphics/src/gpu_culling.rs` - Implementation
- `crates/praxis_graphics/src/shaders/gpu_culling.comp` - Compute shader
