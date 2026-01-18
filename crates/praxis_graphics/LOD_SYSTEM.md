# Level of Detail (LOD) System

Comprehensive LOD system with both CPU-based and GPU-driven implementations for scalable scene rendering.

## Overview

The Praxis LOD system provides two complementary implementations:

**CPU-Based LOD** (`LodManager`, `LodGroup`)
- Per-entity distance checks
- Smooth alpha-blended transitions
- ECS integration
- Best for: <5,000 objects with transition effects

**GPU-Driven LOD** (`GpuLodSelector`)
- Massively parallel compute shader selection
- Zero CPU per-object overhead
- Direct integration with GPU culling
- Best for: 5,000+ objects requiring maximum scalability

## Quick Start

### CPU-Based LOD

```rust
use praxis_graphics::lod::{LodGroup, LodLevel};

let mut lod_group = LodGroup::new(vec![
    LodLevel::new("tree_high", 0.0, 15.0),
    LodLevel::new("tree_medium", 15.0, 40.0),
    LodLevel::new("tree_low", 40.0, 100.0),
]);

// Configure smooth transitions
lod_group.set_transition_duration(0.5);
lod_group.enable_transitions(true);

// Update each frame
lod_manager.update_lod_group(
    &mut lod_group,
    object_position,
    camera_position,
    delta_time,
);

// Render with alpha blending
for (mesh_id, alpha) in lod_group.get_render_meshes() {
    render_mesh(mesh_id, alpha);
}
```

### GPU-Driven LOD

```rust
use praxis_graphics::lod::{GpuLodSelector, GpuObjectData, GpuLodLevel};

let mut selector = GpuLodSelector::new(
    device,
    memory_allocator,
    descriptor_set_allocator,
)?;

// Define LOD levels
let lod_levels = vec![
    GpuLodLevel { mesh_id: 0, min_distance_sq: 0.0, max_distance_sq: 100.0, padding: 0 },
    GpuLodLevel { mesh_id: 1, min_distance_sq: 100.0, max_distance_sq: 625.0, padding: 0 },
    GpuLodLevel { mesh_id: 2, min_distance_sq: 625.0, max_distance_sq: f32::MAX, padding: 0 },
];

// Prepare frame
selector.prepare_frame(&objects, &lod_levels)?;

// Dispatch LOD selection
selector.dispatch_lod_selection(
    command_buffer_builder,
    camera_position,
    lod_bias,
    enable_lod,
)?;
```

## CPU-Based LOD

### Components

**`LodLevel`**: Single LOD with distance thresholds
**`LodGroup`**: Multiple LOD levels for an entity
**`LodManager`**: System-wide LOD management

### Features

**Distance-Based Selection**
- Uses squared distance (avoids sqrt)
- Configurable thresholds per level
- Support for LOD bias

**Smooth Transitions**
- Alpha-blended crossfading
- Configurable duration
- Optional immediate switching

**ECS Integration**
```rust
#[derive(Component)]
struct LodGroupComponent {
    lod_group: LodGroup,
}

fn update_lod_system(
    mut query: Query<(&Transform, &mut LodGroupComponent)>,
    camera: Res<Camera>,
    time: Res<Time>,
) {
    for (transform, mut lod_comp) in query.iter_mut() {
        lod_manager.update_lod_group(
            &mut lod_comp.lod_group,
            transform.translation,
            camera.position,
            time.delta_seconds(),
        );
    }
}
```

### Performance

- **Per-object cost**: ~5-10ns
- **Scalability**: Good up to ~1,000 objects
- **Overhead**: O(n) CPU iteration

## GPU-Driven LOD

### Data Structures

```rust
// Per-object data (96 bytes)
struct GpuObjectData {
    model: Mat4,           // 64 bytes
    bounding_sphere: Vec4, // 16 bytes: (center.xyz, radius)
    mesh_id: u32,          // Base mesh ID
    lod_count: u32,        // Number of LOD levels
    lod_offset: u32,       // Offset in LOD array
    padding: u32,
}

// LOD level definition (16 bytes)
struct GpuLodLevel {
    mesh_id: u32,          // Mesh ID for this level
    min_distance_sq: f32,  // Min squared distance
    max_distance_sq: f32,  // Max squared distance
    padding: u32,
}
```

### Compute Shader Pipeline

```
Input:
  - Object data buffer (transforms, LOD metadata)
  - LOD level definitions
  - Camera position
  - LOD bias

Processing (Parallel):
  - Calculate distance to camera
  - Select appropriate LOD level
  - Apply LOD bias adjustment

Output:
  - Selected mesh IDs (u32 per object)
  - Distances (f32 per object, optional)
```

### Integration with GPU Culling

```glsl
// LOD selection writes mesh IDs
layout(binding = 0) buffer SelectedLods {
    uint selected_mesh_ids[];
};

// Culling reads selected LODs
void main() {
    uint object_index = gl_GlobalInvocationID.x;
    uint mesh_id = selected_mesh_ids[object_index];
    
    MeshData mesh = meshes[mesh_id];
    // Perform frustum culling...
}
```

See [GPU LOD Integration](GPU_LOD_INTEGRATION.md) for complete guide.

### Performance

| Objects | CPU Time | GPU Time | Total |
|---------|----------|----------|-------|
| 1,000   | <0.01ms  | 0.02ms   | 0.03ms |
| 10,000  | <0.01ms  | 0.15ms   | 0.16ms |
| 100,000 | <0.01ms  | 1.5ms    | 1.51ms |

**Crossover point**: GPU becomes more efficient at ~5,000 objects

## LOD Bias

Runtime adjustment of detail levels:

```rust
// Positive: prefer higher detail (objects appear closer)
lod_bias = 0.5;

// Negative: prefer lower detail (objects appear farther)
lod_bias = -0.5;

// Neutral: use default thresholds
lod_bias = 0.0;
```

**Shader implementation:**
```glsl
float bias_scale = (lod_bias > 0.0)
    ? (1.0 - lod_bias * 0.5)
    : (1.0 + (-lod_bias) * 0.5);
float adjusted_distance_sq = distance_sq * bias_scale * bias_scale;
```

## Best Practices

### Distance Thresholds

```rust
// Good: Clear separation
LodLevel::new("high", 0.0, 10.0),    // 0-10 units
LodLevel::new("medium", 10.0, 25.0), // 10-25 units
LodLevel::new("low", 25.0, 50.0),    // 25-50 units

// Bad: Overlapping
LodLevel::new("high", 0.0, 15.0),
LodLevel::new("medium", 10.0, 30.0), // Overlap at 10-15
```

### Mesh Complexity

```
LOD 0: 10,000 tris (100%)
LOD 1:  5,000 tris (50%)
LOD 2:  2,000 tris (20%)
LOD 3:    500 tris (5%)
```

### Bounding Spheres

- Use conservative spheres
- Account for animation/deformation
- Test coverage in editor tools

### Buffer Management

```rust
// Pre-allocate for max expected
let max_objects = 50_000;
selector.allocate_buffers(max_objects)?;

// Avoid frequent reallocations
if objects.len() > max_objects {
    max_objects = objects.len().next_power_of_two();
    selector.reallocate(max_objects)?;
}
```

## Hybrid Approach

Combine both for optimal results:

```rust
// Important objects: CPU LOD with smooth transitions
for character in characters {
    cpu_lod_manager.update_lod_group(
        &mut character.lod_group,
        character.position,
        camera_position,
        delta_time,
    );
}

// Background objects: GPU LOD for scalability
gpu_lod_selector.prepare_frame(&background_objects, &lod_levels)?;
gpu_lod_selector.dispatch_lod_selection(
    command_buffer,
    camera_position,
    lod_bias,
    true,
)?;
```

## Debugging

### Visualize LOD Selections

```rust
// Read back selections (expensive)
let selected_lods = selector.read_selected_lods()?;

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

### Validate Distances

```rust
let gpu_distances = selector.read_distances()?;
for (i, &gpu_dist) in gpu_distances.iter().enumerate() {
    let cpu_dist = (obj_pos - cam_pos).length_squared();
    assert!((gpu_dist - cpu_dist).abs() < 0.01);
}
```

## Comparison Summary

| Feature | CPU LOD | GPU LOD |
|---------|---------|---------|
| Max objects | ~1,000 | 100,000+ |
| Transitions | Smooth alpha blend | Instant |
| Per-object cost | 5-10ns CPU | 0.001ns CPU |
| Memory | Minimal | ~96 bytes/object |
| ECS integration | Native | Manual |
| Setup complexity | Simple | Moderate |

## See Also

- [GPU LOD Integration](GPU_LOD_INTEGRATION.md) - Complete integration guide
- [GPU Culling](GPU_CULLING.md) - Visibility determination
- Example: `examples/lod_gpu_demo.rs`
- Implementation: `crates/praxis_graphics/src/lod.rs`
- Shader: `crates/praxis_graphics/src/shaders/lod_selection.comp`
