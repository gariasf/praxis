# GPU LOD Integration Guide

Complete integration guide for GPU-driven LOD selection with the rendering pipeline.

## Overview

GPU LOD selection uses compute shaders to calculate appropriate LOD levels for objects based on distance from camera. This guide covers integration with GPU culling and indirect drawing.

## Architecture

```
CPU Side:
  1. Create GpuObjectData (transforms, LOD metadata)
  2. Define GpuLodLevel array (distance thresholds)
  3. Upload to GPU buffers
  4. Dispatch LOD selection compute shader

GPU Side:
  1. LOD Selection Pass (compute):
     - Calculate distance to camera
     - Select appropriate LOD level
     - Write selected mesh IDs to buffer
  
  2. Culling Pass (compute):
     - Read selected LOD levels
     - Perform frustum culling
     - Generate indirect draw commands
  
  3. Indirect Draw (graphics):
     - Execute visible objects
```

## Integration Steps

### Step 1: Define LOD Levels

```rust
use praxis_graphics::lod::{GpuLodLevel, GpuObjectData};

// Define LOD levels for each object type
let tree_lods = vec![
    GpuLodLevel {
        mesh_id: 0,              // High detail
        min_distance_sq: 0.0,
        max_distance_sq: 100.0,  // 0-10 units
        padding: 0,
    },
    GpuLodLevel {
        mesh_id: 1,              // Medium detail
        min_distance_sq: 100.0,
        max_distance_sq: 625.0,  // 10-25 units
        padding: 0,
    },
    GpuLodLevel {
        mesh_id: 2,              // Low detail
        min_distance_sq: 625.0,
        max_distance_sq: f32::MAX, // 25+ units
        padding: 0,
    },
];
```

### Step 2: Create Object Data

```rust
let mut objects = Vec::new();
let mut all_lod_levels = Vec::new();
let mut lod_offset = 0u32;

for (position, object_type) in scene_objects {
    let model = Mat4::from_translation(position);
    let lods = get_lods_for_type(object_type);
    let lod_count = lods.len() as u32;
    
    // Add LOD definitions to global array
    all_lod_levels.extend_from_slice(&lods);
    
    // Create object data with LOD metadata
    objects.push(GpuObjectData::new(
        model,
        [0.0, 0.0, 0.0, 1.0],  // Bounding sphere
        lods[0].mesh_id,        // Base mesh ID
        lod_count,
        lod_offset,
    ));
    
    lod_offset += lod_count;
}
```

### Step 3: Initialize GPU LOD Selector

```rust
use praxis_graphics::lod::GpuLodSelector;

let mut lod_selector = GpuLodSelector::new(
    device,
    memory_allocator,
    descriptor_set_allocator,
)?;
```

### Step 4: Dispatch LOD Selection Per Frame

```rust
// Prepare frame data
lod_selector.prepare_frame(&objects, &all_lod_levels)?;

// Dispatch LOD selection
lod_selector.dispatch_lod_selection(
    command_buffer_builder,
    camera_position,  // Vec3
    lod_bias,         // f32: -1.0 to 1.0
    enable_lod,       // bool
)?;

// Get selected LOD buffer for culling
let selected_lod_buffer = lod_selector.selected_lod_buffer();
```

### Step 5: Integrate with GPU Culling

Modify GPU culling shader to read selected LODs:

```glsl
// Add binding for selected LODs
layout(set = 0, binding = 6, std430) readonly buffer SelectedLods {
    uint selected_mesh_ids[];
} selected_lods;

void main() {
    uint object_index = gl_GlobalInvocationID.x;
    
    // Read selected LOD level for this object
    uint mesh_id = selected_lods.selected_mesh_ids[object_index];
    
    // Use correct mesh for culling and draw commands
    MeshData mesh = mesh_data.meshes[mesh_id];
    
    // Perform frustum culling...
    if (is_visible(mesh)) {
        // Generate indirect draw command
    }
}
```

### Step 6: Synchronization

Vulkano automatically handles pipeline barriers based on buffer usage:

```rust
// LOD selection buffer has:
// - STORAGE_BUFFER usage (compute writes)
// - Bound as input to culling compute shader

// Vulkano inserts barrier:
//   srcStage:  COMPUTE_SHADER
//   srcAccess: SHADER_WRITE
//   dstStage:  COMPUTE_SHADER
//   dstAccess: SHADER_READ
```

No manual synchronization required.

## Complete Integration Example

```rust
use praxis_graphics::lod::{GpuLodSelector, GpuObjectData, GpuLodLevel};
use praxis_graphics::GpuCullingSystem;

struct Renderer {
    lod_selector: GpuLodSelector,
    culling_system: GpuCullingSystem,
}

impl Renderer {
    fn render_frame(&mut self, objects: &[Object], camera: &Camera) -> Result<()> {
        // 1. Prepare LOD data
        let gpu_objects: Vec<GpuObjectData> = objects.iter()
            .map(|obj| obj.to_gpu_data())
            .collect();
        
        let lod_levels: Vec<GpuLodLevel> = objects.iter()
            .flat_map(|obj| obj.lod_levels.clone())
            .collect();
        
        self.lod_selector.prepare_frame(&gpu_objects, &lod_levels)?;
        
        // 2. Begin command buffer
        let mut builder = AutoCommandBufferBuilder::primary(...)?;
        
        // 3. Dispatch LOD selection
        self.lod_selector.dispatch_lod_selection(
            &mut builder,
            camera.position,
            0.0,   // lod_bias
            true,  // enable_lod
        )?;
        
        // 4. Dispatch GPU culling (reads LOD buffer)
        self.culling_system.dispatch_culling(
            &mut builder,
            camera.view_projection,
            camera.position,
        )?;
        
        // 5. Execute indirect draw
        let indirect_buffer = self.culling_system.indirect_draw_buffer();
        builder.draw_indirect(indirect_buffer, ...)?;
        
        // 6. Execute command buffer
        let command_buffer = builder.build()?;
        command_buffer.execute(queue)?;
        
        Ok(())
    }
}
```

## LOD Bias

Runtime detail adjustment:

```rust
// Positive: prefer higher detail
lod_bias = 0.5;

// Negative: prefer lower detail
lod_bias = -0.5;

// Neutral: default thresholds
lod_bias = 0.0;
```

**Shader implementation:**
```glsl
float bias_scale = (lod_bias > 0.0)
    ? (1.0 - lod_bias * 0.5)
    : (1.0 + (-lod_bias) * 0.5);
float adjusted_distance_sq = distance_sq * bias_scale * bias_scale;
```

## Performance

| Objects | CPU Time | GPU Time | Total |
|---------|----------|----------|-------|
| 1,000   | <0.01ms  | 0.02ms   | 0.03ms |
| 10,000  | <0.01ms  | 0.15ms   | 0.16ms |
| 100,000 | <0.01ms  | 1.5ms    | 1.51ms |

**CPU overhead**: O(1) - independent of object count  
**GPU scalability**: Linear with object count

## Debugging

### Read Back Selected LODs

```rust
let selected_lods = lod_selector.read_selected_lods()?;
let distances = lod_selector.read_distances()?;

// Visualize LOD distribution
for (i, &lod) in selected_lods.iter().enumerate() {
    let color = match lod {
        0 => Vec3::new(0.0, 1.0, 0.0), // Green = high
        1 => Vec3::new(1.0, 1.0, 0.0), // Yellow = medium
        2 => Vec3::new(1.0, 0.0, 0.0), // Red = low
        _ => Vec3::new(1.0, 0.0, 1.0), // Magenta = error
    };
    draw_debug_sphere(objects[i].position, color);
}
```

### Validate Distance Calculations

```rust
let gpu_distances = lod_selector.read_distances()?;
for (i, &gpu_dist) in gpu_distances.iter().enumerate() {
    let cpu_dist = (obj_pos - cam_pos).length_squared();
    let error = (gpu_dist - cpu_dist).abs();
    assert!(error < 0.01, "Distance mismatch at object {}", i);
}
```

## Best Practices

### LOD Level Design

**Distance thresholds:**
```rust
// Use squared distances (avoid sqrt)
LodLevel { min_distance_sq: 0.0, max_distance_sq: 100.0 },    // 0-10 units
LodLevel { min_distance_sq: 100.0, max_distance_sq: 625.0 },  // 10-25 units
```

**Mesh reduction:**
```
LOD 0: 100% triangles
LOD 1: 50-70% triangles
LOD 2: 20-30% triangles
LOD 3: <10% triangles
```

### Buffer Management

```rust
// Pre-allocate for max expected objects
let max_objects = 50_000;
lod_selector.allocate_buffers(max_objects)?;

// Grow in powers of two
if objects.len() > max_objects {
    max_objects = objects.len().next_power_of_two();
    lod_selector.reallocate(max_objects)?;
}
```

### Memory Organization

```rust
// Group LOD levels by object type
struct ObjectType {
    lod_levels: Vec<GpuLodLevel>,
}

// Share LOD definitions across instances
let tree_lods = object_types["tree"].lod_levels.clone();
for tree_position in tree_positions {
    // All trees share same LOD definitions
}
```

## Troubleshooting

### LOD Popping

**Symptom**: Visible switches between LOD levels

**Solutions**:
- Increase distance thresholds
- Add hysteresis (delay switching)
- Use screen-space LOD instead of distance-based

### Performance Issues

**Symptom**: High GPU time in LOD selection

**Solutions**:
- Reduce number of LOD levels per object
- Profile with GPU debugging tools
- Consider CPU LOD for small scenes (<5,000 objects)

### Incorrect LOD Selection

**Symptom**: Wrong LOD displayed for distance

**Solutions**:
- Verify distance thresholds are squared
- Check camera position passed correctly
- Debug with readback and CPU comparison

## See Also

- [LOD System](LOD_SYSTEM.md) - Overview of CPU and GPU LOD
- [GPU Culling](GPU_CULLING.md) - GPU frustum culling
- Example: `examples/lod_gpu_demo.rs`
- Implementation: `crates/praxis_graphics/src/lod.rs`
- Shader: `crates/praxis_graphics/src/shaders/lod_selection.comp`
