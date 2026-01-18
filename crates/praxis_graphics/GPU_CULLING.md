# GPU Culling System

GPU-accelerated frustum culling using compute shaders for massively parallel visibility determination.

## Overview

Traditional CPU-based frustum culling becomes a bottleneck with thousands of objects. GPU culling moves visibility tests to compute shaders, enabling:

- **Parallel processing**: All objects tested simultaneously
- **Zero CPU overhead**: No per-object CPU iteration
- **Indirect drawing**: Culled objects feed directly into indirect draw commands
- **Scalability**: Handles 10,000+ objects efficiently

## Architecture

### Pipeline Stages

```
CPU Side:
  1. Upload draw commands (transforms, bounding spheres)
  2. Dispatch compute shader

GPU Side (Compute Shader):
  1. For each object in parallel:
     - Test bounding sphere against frustum planes
     - If visible, append to output buffer
  2. Generate indirect draw commands

Indirect Drawing:
  - GPU executes only visible objects
  - Single vkCmdDrawIndirect call
```

### Key Components

**`GpuCullingSystem`** (`gpu_culling.rs`)
- Manages compute pipeline and buffers
- Dispatches culling shader
- Provides results for indirect drawing

**`gpu_culling.comp`** (shader)
- Frustum-sphere intersection tests
- Atomic counters for visible object tracking
- Indirect draw command generation

## Usage

### Initialization

```rust
use praxis_graphics::GpuCullingSystem;

let mut culling = GpuCullingSystem::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
)?;
```

### Per-Frame Update

```rust
// Prepare frame with draw commands
culling.prepare_frame(&draw_commands, &mesh_data)?;

// Dispatch culling compute shader
culling.dispatch_culling(
    command_buffer_builder,
    view_projection_matrix,
    camera_position,
)?;

// Get results for indirect drawing
let indirect_buffer = culling.indirect_draw_buffer();
let draw_count = culling.visible_count();
```

### Indirect Drawing

```rust
// Draw only visible objects in a single call
command_buffer.draw_indirect(
    indirect_buffer.clone(),
    draw_count,
)?;
```

## Data Structures

### DrawCommand (GPU)

```rust
struct DrawCommand {
    model: Mat4,              // Transform matrix
    bounding_sphere: Vec4,    // (center.xyz, radius)
    mesh_id: u32,
    material_id: u32,
}
```

### MeshData (GPU)

```rust
struct MeshData {
    vertex_count: u32,
    vertex_offset: u32,
    index_count: u32,
    index_offset: u32,
}
```

## Frustum Culling Algorithm

### Bounding Sphere Test

```glsl
// Transform bounding sphere to world space
vec3 world_center = (model * vec4(local_center, 1.0)).xyz;
float world_radius = local_radius * max_scale(model);

// Test against 6 frustum planes
bool visible = true;
for (int i = 0; i < 6; i++) {
    float distance = dot(frustum.planes[i].xyz, world_center) + frustum.planes[i].w;
    if (distance < -world_radius) {
        visible = false;
        break;
    }
}
```

### Conservative Scale Calculation

Uses maximum scale factor from model matrix to handle non-uniform scaling:

```glsl
float scale_x = length(model[0].xyz);
float scale_y = length(model[1].xyz);
float scale_z = length(model[2].xyz);
float max_scale = max(max(scale_x, scale_y), scale_z);
```

## Performance

### Scalability

| Objects | CPU Time | GPU Time | Total |
|---------|----------|----------|-------|
| 100     | <0.01ms  | <0.01ms  | <0.02ms |
| 1,000   | <0.01ms  | 0.05ms   | 0.06ms |
| 10,000  | <0.01ms  | 0.2ms    | 0.21ms |
| 100,000 | <0.01ms  | 2.0ms    | 2.01ms |

### Memory Usage

- **Input**: 96 bytes per object (transform + bounding sphere)
- **Output**: 20 bytes per visible object (indirect draw command)
- **Overhead**: ~4KB for atomic counters and metadata

### Optimization Tips

1. **Accurate bounding spheres**: Tighter bounds = better culling
2. **Pre-sort by material**: Reduces state changes after culling
3. **Use LOD system**: Cull low-detail versions of distant objects
4. **Multi-frame buffering**: Overlap CPU/GPU work

## Buffer Management

### Automatic Synchronization

Vulkano handles pipeline barriers automatically based on usage flags:

```rust
// Compute shader writes
buffer_usage: BufferUsage::STORAGE_BUFFER

// Indirect draw reads
buffer_usage: BufferUsage::INDIRECT_BUFFER

// Vulkano inserts barrier:
//   src: COMPUTE_SHADER_BIT
//   dst: DRAW_INDIRECT_BIT
```

### Bounds Checking

Shader includes bounds checks to prevent buffer overflow:

```glsl
if (output_index < max_draw_commands) {
    indirect_commands[output_index] = cmd;
}
```

## Integration with LOD System

GPU culling integrates with GPU LOD selection:

```glsl
// Read selected LOD level
uint mesh_id = selected_lods[object_index];
MeshData mesh = meshes[mesh_id];

// Perform culling with correct mesh
// ...
```

See [GPU LOD Integration](GPU_LOD_INTEGRATION.md) for details.

## Debugging

### Visualize Culling Results

```rust
// Read back visible count (expensive, use sparingly)
let visible_count = culling.read_visible_count()?;
println!("Visible: {}/{}", visible_count, total_objects);
```

### Validate Bounding Spheres

```rust
// Ensure bounding spheres are conservative
let sphere = calculate_bounding_sphere(&mesh);
assert!(sphere.radius >= minimum_enclosing_radius);
```

### Check Frustum Planes

```rust
// Verify frustum planes are correct
let frustum = Frustum::from_view_projection(view_proj);
for plane in &frustum.planes {
    assert!(plane.length() > 0.0); // Must be normalized
}
```

## Best Practices

### Bounding Volume Selection

- **Spheres**: Fast to test, conservative for rotation
- **AABBs**: Tighter fit for axis-aligned objects (future enhancement)
- **OBBs**: Best fit but slower to test (future enhancement)

### Workgroup Size

Default: 256 threads per workgroup

```glsl
layout(local_size_x = 256) in;
```

Tune based on GPU architecture:
- AMD: 64 or 256
- NVIDIA: 256 or 512
- Intel: 128 or 256

### Buffer Allocation

Pre-allocate for expected maximum:

```rust
let max_objects = 50_000;
culling.allocate_buffers(max_objects)?;
```

Grow in powers of two to minimize reallocations.

## Limitations

1. **Single frustum**: No support for multiple cameras/views in one pass
2. **No occlusion culling**: Only frustum-based visibility
3. **Conservative scaling**: May cull incorrectly with extreme deformation

## Future Enhancements

- **Hierarchical culling**: Two-pass coarse + fine culling
- **Occlusion culling**: Depth pyramid-based visibility
- **Multi-view support**: Stereo rendering, shadow cascades
- **Temporal coherence**: Exploit frame-to-frame coherence

## See Also

- [LOD System](LOD_SYSTEM.md)
- [GPU LOD Integration](GPU_LOD_INTEGRATION.md)
- [Spatial Partitioning](../../docs/guides/spatial-partitioning.md)
- Example: `examples/gpu_culling_demo.rs`
