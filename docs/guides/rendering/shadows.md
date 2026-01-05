# Shadow Mapping

Cascaded shadow maps (CSM) with percentage closer filtering (PCF) provide realistic, soft shadows from directional lights. This guide covers shadow mapping implementation and best practices in Praxis.

## Overview

Shadow mapping is a two-pass rendering technique:

1. **Shadow Pass**: Render the scene from the light's perspective to a depth texture (shadow map)
2. **Main Pass**: Render the scene normally, sampling the shadow map to determine if fragments are in shadow

**Key Idea**: If a fragment's distance from the light is greater than the stored depth in the shadow map, it's in shadow.

## Cascaded Shadow Maps (CSM)

CSM divides the view frustum into multiple cascades at different distances from the camera, allocating higher resolution to areas closer to the camera.

### Why Multiple Cascades?

A single shadow map covering the entire view frustum would have poor resolution near the camera where detail matters most. CSM solves this by using multiple shadow maps with different coverage areas:

```
Camera                                           Far Plane
  │                                                   │
  ├──────────┬────────────────┬───────────────────────┤
  │Cascade 0 │   Cascade 1    │      Cascade 2        │
  │ (0-20m)  │   (20-100m)    │     (100-500m)        │
  │ High Res │  Medium Res    │      Lower Res        │
  └──────────┴────────────────┴───────────────────────┘
```

**Benefits**:
- Prevents shadow aliasing (blocky shadows) near camera
- Maintains reasonable performance and memory usage
- Distributes shadow map resolution effectively

## Configuration

Shadow quality is controlled via `ShadowConfig`:

```rust
use praxis_graphics::shadow::ShadowConfig;

// High-quality configuration for close-up scenes
let high_quality = ShadowConfig {
    shadow_map_size: 2048,      // 2048×2048 per cascade
    cascade_count: 4,            // 4 cascades
    cascade_distances: [10.0, 30.0, 100.0, 300.0],
    pcf_samples: 9,              // 3×3 PCF filter
    bias: 0.005,                 // Shadow acne prevention
};

// Performance-focused configuration for open worlds
let performance = ShadowConfig {
    shadow_map_size: 1024,       // 1024×1024 per cascade
    cascade_count: 2,            // 2 cascades
    cascade_distances: [30.0, 150.0, 500.0, 1000.0],
    pcf_samples: 4,              // 2×2 PCF filter
    bias: 0.01,
};

// Default configuration (balanced)
let default = ShadowConfig::default();
```

### Configuration Parameters

**`shadow_map_size`**: Resolution of each shadow map (power of 2)
- 512: Low quality, best performance
- 1024: Medium quality (default)
- 2048: High quality
- 4096: Very high quality, expensive

**`cascade_count`**: Number of shadow cascades (1-4)
- More cascades = better quality at all distances
- Fewer cascades = better performance
- Typical: 3-4 cascades

**`cascade_distances`**: Distance from camera where each cascade ends (meters)
- Must be in ascending order
- Only first `cascade_count` values are used
- Example: [20.0, 100.0, 500.0, _] for 3 cascades
- Adjust based on scene scale

**`pcf_samples`**: Number of samples for PCF filtering (1, 4, 9, or 16)
- 1: Hard shadows (no filtering)
- 4: Soft shadows (2×2 filter)
- 9: Softer shadows (3×3 filter, recommended)
- 16: Softest shadows (4×4 filter)

**`bias`**: Shadow bias to prevent self-shadowing artifacts
- Too low: Shadow acne (surfaces shadow themselves)
- Too high: Peter panning (shadows detach from objects)
- Typical: 0.001 - 0.01
- Adjust based on scene geometry

## PCF Filtering

Percentage Closer Filtering (PCF) softens shadow edges by sampling multiple points in the shadow map and averaging the results.

### How PCF Works

1. For each fragment, sample the shadow map multiple times in a kernel pattern
2. Compare fragment depth with shadow map depth for each sample
3. Average the comparison results
4. Use averaged value as shadow factor (0 = fully shadowed, 1 = fully lit)

### Sample Patterns

| Samples | Pattern | Quality | Cost |
|---------|---------|---------|------|
| 1 | Single point | Hard shadows | Fastest |
| 4 | 2×2 grid | Soft shadows | Low |
| 9 | 3×3 grid | Softer shadows | Medium |
| 16 | 4×4 grid | Softest shadows | Higher |

**Shader Implementation**:
```glsl
float pcf_shadow(vec3 shadow_coords, int samples) {
    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(shadow_map, 0);
    int half_samples = samples / 2;
    
    for (int x = -half_samples; x <= half_samples; x++) {
        for (int y = -half_samples; y <= half_samples; y++) {
            vec2 offset = vec2(x, y) * texel_size;
            float depth = texture(shadow_map, shadow_coords.xy + offset).r;
            shadow += (shadow_coords.z > depth + bias) ? 0.0 : 1.0;
        }
    }
    
    return shadow / (samples * samples);
}
```

## Implementation

### Shadow Map Manager

`ShadowMapManager` handles all shadow-related resources:

```rust
use praxis_graphics::shadow::{ShadowMapManager, ShadowConfig};
use praxis_math::Vec3;

// Create shadow manager
let shadow_manager = ShadowMapManager::new(
    memory_allocator.clone(),
    ShadowConfig::default(),
)?;

// Update shadow matrices each frame
let light_direction = Vec3::new(0.3, -0.8, 0.5).normalize();
shadow_manager.update(
    light_direction,
    camera_view_matrix,
    camera_projection_matrix,
)?;

// Access shadow resources
let shadow_uniform_buffer = shadow_manager.uniform_buffer();
let shadow_maps = shadow_manager.shadow_map_views();
let shadow_framebuffers = shadow_manager.shadow_framebuffers();
```

### Light-Space Matrix Calculation

For each cascade, the system:

1. Calculates the frustum sub-volume for this cascade
2. Extracts the 8 corners of the frustum sub-volume in world space
3. Creates a light view matrix looking from light toward camera
4. Transforms frustum corners to light space
5. Calculates tight AABB (axis-aligned bounding box) around transformed corners
6. Creates orthographic projection covering the AABB
7. Combines: `light_space_matrix = proj * view`

This ensures shadow maps tightly fit each cascade, maximizing effective resolution.

### Render Loop Integration

```rust
// 1. Shadow pass: Render to shadow maps
for cascade_idx in 0..shadow_manager.cascade_count() {
    let framebuffer = shadow_manager.shadow_framebuffers()[cascade_idx];
    let light_space_matrix = shadow_manager.light_space_matrices()[cascade_idx];
    
    // Begin render pass
    // Render geometry from light's perspective
    // End render pass
}

// 2. Main pass: Shadows applied automatically via descriptor bindings
render_main_scene_with_shadows()?;
```

### Shader Integration

The fragment shader automatically:

1. Calculates fragment's distance from camera
2. Selects appropriate cascade based on distance
3. Transforms fragment to light space using cascade's matrix
4. Performs PCF sampling on the selected shadow map
5. Returns shadow factor (0 = shadowed, 1 = lit)
6. Modulates lighting by shadow factor

```glsl
// Shadow calculation in fragment shader
vec3 world_pos = v_world_pos;
float view_distance = length(camera_pos - world_pos);

// Select cascade
int cascade_index = 0;
for (int i = 0; i < cascade_count; i++) {
    if (view_distance < cascade_distances[i]) {
        cascade_index = i;
        break;
    }
}

// Transform to light space
vec4 light_space_pos = light_space_matrices[cascade_index] * vec4(world_pos, 1.0);
vec3 shadow_coords = light_space_pos.xyz / light_space_pos.w;
shadow_coords = shadow_coords * 0.5 + 0.5;  // [-1,1] to [0,1]

// Sample shadow map with PCF
float shadow_factor = pcf_shadow(shadow_coords, cascade_index);

// Apply to lighting
vec3 lit_color = calculate_lighting(...);
vec3 final_color = lit_color * shadow_factor;
```

### Descriptor Set Layout

Shadow data is bound to the graphics pipeline at:

- **Set 0, Binding 4**: Shadow uniform buffer (matrices, config)
- **Set 0, Binding 5**: Shadow map cascade 0
- **Set 0, Binding 6**: Shadow map cascade 1
- **Set 0, Binding 7**: Shadow map cascade 2
- **Set 0, Binding 8**: Shadow map cascade 3

## Performance

### Memory Usage

Each shadow map uses `shadow_map_size² × 4 bytes` (D32_SFLOAT format).

| Configuration | Memory |
|---------------|--------|
| 2 cascades × 512² | 2 MB |
| 3 cascades × 1024² | 12 MB |
| 4 cascades × 2048² | 64 MB |

**Formula**: `cascade_count × shadow_map_size² × 4 bytes`

### Rendering Cost

**Shadow pass cost** depends on:
- Scene complexity (triangle count)
- Number of cascades (render pass per cascade)
- Shadow map resolution
- Culling efficiency

**Main pass cost** depends on:
- Screen resolution (fragments processed)
- PCF sample count (1, 4, 9, or 16)
- Shader complexity

### Recommended Settings

| Target | Resolution | Cascades | PCF | Frame Cost |
|--------|------------|----------|-----|------------|
| Mobile | 512 | 2 | 1 | ~0.5-1ms |
| Low Desktop | 1024 | 2 | 4 | ~1-2ms |
| Mid Desktop | 1024 | 3 | 4 | ~2-3ms |
| High Desktop | 2048 | 3 | 9 | ~4-6ms |

## Optimization Tips

### 1. Reduce Cascade Count

For distant views where detail isn't needed:

```rust
config.cascade_count = 2;  // Instead of 4
config.cascade_distances = [50.0, 200.0, 500.0, 1000.0];
```

### 2. Lower Shadow Map Resolution

For performance-critical scenarios:

```rust
config.shadow_map_size = 1024;  // Instead of 2048
```

### 3. Use Fewer PCF Samples

```rust
config.pcf_samples = 4;  // Instead of 9 or 16
```

### 4. Implement Shadow Caster Culling

Only render objects that cast visible shadows:

```rust
fn should_cast_shadow(object: &Object, light_frustum: &Frustum) -> bool {
    // Cull objects outside light frustum
    light_frustum.contains(&object.bounding_box())
}
```

### 5. Adjust Bias Carefully

Find the minimum bias that prevents shadow acne:

```rust
// Start high, then reduce until artifacts appear
config.bias = 0.01;  // Try 0.005, 0.001, etc.
```

## Common Issues and Solutions

### Shadow Acne

**Symptoms**: Surfaces incorrectly shadow themselves with moiré patterns

**Causes**:
- Insufficient shadow bias
- Shadow map resolution too low
- Large polygons

**Solutions**:
1. Increase `bias` in `ShadowConfig`
2. Increase shadow map resolution
3. Enable slope-scale depth bias (already enabled in Praxis)

```rust
config.bias = 0.01;  // Increase from 0.005
```

### Peter Panning

**Symptoms**: Shadows appear detached from objects, floating shadows

**Causes**:
- Shadow bias too high

**Solutions**:
1. Decrease `bias` in `ShadowConfig`
2. Find balance between acne and peter panning

```rust
config.bias = 0.005;  // Decrease from 0.01
```

### Blocky Shadows

**Symptoms**: Jagged, pixelated shadow edges

**Causes**:
- Shadow map resolution too low
- Cascade too large
- No PCF filtering

**Solutions**:
1. Increase shadow map resolution
2. Adjust cascade distances to be more granular
3. Increase PCF sample count

```rust
config.shadow_map_size = 2048;  // From 1024
config.pcf_samples = 9;  // From 4
config.cascade_distances = [10.0, 30.0, 100.0, 300.0];  // More granular
```

### Cascade Seams

**Symptoms**: Visible boundaries between cascades

**Causes**:
- Abrupt transition between cascades
- Different shadow quality per cascade

**Solutions**:
1. Implement cascade blending (blend shadows near boundaries)
2. Use consistent PCF sample count across cascades
3. Adjust cascade distances for smoother transitions

### Poor Performance

**Symptoms**: Low frame rate with shadows enabled

**Causes**:
- Too many cascades
- Shadow maps too large
- High PCF sample count
- Complex geometry in shadow pass

**Solutions**:
1. Reduce cascade count (3 → 2)
2. Reduce shadow map size (2048 → 1024)
3. Reduce PCF samples (9 → 4)
4. Implement shadow caster culling
5. Use LOD for distant shadow casters

## Integration with Other Systems

### With Forward Rendering

```rust
// Shadows integrate seamlessly with forward rendering
render_context.render(&RenderCommands {
    view,
    proj,
    draw_commands,
    lighting: Some(&lighting),
    shadows: Some(&shadow_manager),  // Shadows enabled
})?;
```

### With Deferred Rendering

```rust
// Shadows applied in lighting pass
deferred_renderer.render_with_shadows(
    builder,
    output_framebuffer,
    viewport,
    draw_commands,
    view_proj_buffer,
    lighting_buffer,
    &shadow_manager,  // Shadow resources
)?;
```

### With HDR

Shadows work naturally with HDR rendering:

```rust
// Render to HDR target with shadows
render_to_hdr_with_shadows(&hdr_target, &shadow_manager)?;

// Tone map result
tone_mapper.apply(&hdr_target, &output)?;
```

## Examples

```bash
# Shadow mapping demo
cargo run --example shadow_demo

# Advanced lighting with shadows
cargo run --example advanced_lighting_demo

# Comprehensive scene with shadows
cargo run --example comprehensive_scene_demo
```

## See Also

- [Forward Rendering](forward-rendering.md) - Basic rendering pipeline
- [Deferred Rendering](deferred-rendering.md) - Multi-pass rendering
- [HDR and Tone Mapping](hdr-tonemapping.md) - High dynamic range
- [Post-Processing](post-processing.md) - Screen-space effects

## References

- [Microsoft - Cascaded Shadow Maps](https://docs.microsoft.com/en-us/windows/win32/dxtecharts/cascaded-shadow-maps)
- [NVIDIA - Parallel-Split Shadow Maps](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus)
- [Learn OpenGL - Shadow Mapping](https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping)
- [Real-Time Rendering, 4th Edition](http://www.realtimerendering.com/) - Chapter 7: Shadows
