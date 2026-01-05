# Shadow Mapping System

The Praxis engine implements a comprehensive shadow mapping system with cascaded shadow maps (CSM) and percentage closer filtering (PCF) for realistic, soft shadows from directional lights.

## Overview

Shadow mapping is a two-pass rendering technique:

1. **Shadow Pass**: Render the scene from the light's perspective to a depth texture (shadow map)
2. **Main Pass**: Render the scene normally, sampling the shadow map to determine if fragments are in shadow

## Cascaded Shadow Maps (CSM)

CSM divides the view frustum into multiple cascades at different distances from the camera. Each cascade has its own shadow map with appropriate resolution:

- **Near Cascade**: High detail for objects close to camera (e.g., 0-20 meters)
- **Mid Cascades**: Medium detail for mid-range objects (e.g., 20-100 meters)
- **Far Cascade**: Lower detail for distant objects (e.g., 100-500 meters)

This approach prevents shadow aliasing (blocky shadows) near the camera while maintaining reasonable performance and memory usage.

### Why Multiple Cascades?

A single shadow map covering the entire view frustum would have poor resolution near the camera. CSM allocates more texels to areas near the camera where detail matters most.

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
    cascade_count: 3,            // 3 cascades
    cascade_distances: [20.0, 100.0, 500.0, 1000.0],
    pcf_samples: 4,              // 2×2 PCF filter
    bias: 0.01,
};

// Default configuration (balanced)
let default = ShadowConfig::default();
```

### Configuration Parameters

- **`shadow_map_size`**: Resolution of each shadow map (power of 2)
  - 512: Low quality, best performance
  - 1024: Medium quality (default)
  - 2048: High quality
  - 4096: Very high quality, expensive

- **`cascade_count`**: Number of shadow cascades (1-4)
  - More cascades = better quality at all distances
  - Fewer cascades = better performance
  - Typical: 3-4 cascades

- **`cascade_distances`**: Distance from camera where each cascade ends
  - Must be in ascending order
  - Only first `cascade_count` values are used
  - Example: [20.0, 100.0, 500.0, _] for 3 cascades

- **`pcf_samples`**: Number of samples for PCF filtering (1, 4, 9, or 16)
  - 1: Hard shadows (no filtering)
  - 4: Soft shadows (2×2 filter)
  - 9: Softer shadows (3×3 filter)
  - 16: Softest shadows (4×4 filter)

- **`bias`**: Shadow bias to prevent self-shadowing artifacts
  - Too low: Shadow acne (surfaces shadow themselves)
  - Too high: Peter panning (shadows detach from objects)
  - Typical: 0.001 - 0.01

## PCF Filtering

Percentage Closer Filtering (PCF) softens shadow edges by sampling multiple points in the shadow map and averaging the results. This creates smooth shadow transitions instead of hard, aliased edges.

### How PCF Works

1. For each fragment, sample the shadow map multiple times in a kernel pattern
2. Compare fragment depth with shadow map depth for each sample
3. Average the comparison results
4. Use averaged value as shadow factor (0 = fully shadowed, 1 = fully lit)

### PCF Sample Patterns

- **1 sample**: Single center sample (no filtering)
- **4 samples**: 2×2 grid around fragment position
- **9 samples**: 3×3 grid around fragment position
- **16 samples**: 4×4 grid around fragment position

## Implementation Details

### Shadow Map Manager

`ShadowMapManager` handles all shadow-related resources:

```rust
use praxis_graphics::shadow::{ShadowMapManager, ShadowConfig};
use praxis_math::{Mat4, Vec3};

// Create shadow manager
let shadow_manager = ShadowMapManager::new(
    memory_allocator.clone(),
    ShadowConfig::default(),
)?;

// Update shadow matrices each frame
let light_direction = Vec3::new(0.5, -1.0, 0.3).normalize();
shadow_manager.update(light_direction, camera_view, camera_proj)?;

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
5. Calculates tight AABB around transformed corners
6. Creates orthographic projection covering the AABB
7. Combines projection and view: `light_space_matrix = proj * view`

This ensures shadow maps tightly fit each cascade, maximizing effective resolution.

### Shader Integration

The fragment shader automatically:

1. Calculates fragment's distance from camera
2. Selects appropriate cascade based on distance
3. Transforms fragment to light space using cascade's matrix
4. Performs PCF sampling on the selected shadow map
5. Returns shadow factor (0 = shadowed, 1 = lit)
6. Modulates lighting by shadow factor

## Descriptor Set Layout

Shadow data is bound to the graphics pipeline at:

- **Set 0, Binding 4**: Shadow uniform buffer (matrices, config)
- **Set 0, Binding 5**: Shadow map cascade 0
- **Set 0, Binding 6**: Shadow map cascade 1
- **Set 0, Binding 7**: Shadow map cascade 2
- **Set 0, Binding 8**: Shadow map cascade 3

## Performance Considerations

### Memory Usage

Each shadow map uses `shadow_map_size² × 4 bytes` (D32_SFLOAT format).

Example with 3 cascades at 1024×1024:
- Per cascade: 1024 × 1024 × 4 = 4 MB
- Total: 3 × 4 MB = 12 MB

### Rendering Cost

Shadow pass cost depends on:
- Scene complexity (triangle count)
- Number of cascades
- Shadow map resolution
- Culling efficiency

Main pass cost depends on:
- Screen resolution (fragments processed)
- PCF sample count (1, 4, 9, or 16)
- Shader complexity

### Optimization Tips

1. **Reduce cascade count** for distant views where detail isn't needed
2. **Lower shadow map resolution** for performance-critical scenarios
3. **Use fewer PCF samples** (4 instead of 9 or 16)
4. **Increase bias** slightly to allow more aggressive depth culling
5. **Cull shadow casters** outside light frustum
6. **Use static shadow maps** for stationary objects

## Common Issues

### Shadow Acne

**Symptom**: Surfaces incorrectly shadow themselves with moiré patterns

**Causes**:
- Insufficient shadow bias
- Shadow map resolution too low
- Large polygons

**Solutions**:
- Increase `bias` in `ShadowConfig`
- Enable depth bias in shadow pipeline (already enabled)
- Increase shadow map resolution
- Use slope-scale depth bias (already enabled)

### Peter Panning

**Symptom**: Shadows appear detached from objects

**Causes**:
- Shadow bias too high

**Solutions**:
- Decrease `bias` in `ShadowConfig`
- Find balance between acne and peter panning

### Blocky Shadows

**Symptom**: Jagged, pixelated shadow edges

**Causes**:
- Shadow map resolution too low
- Cascade too large
- No PCF filtering

**Solutions**:
- Increase shadow map resolution
- Adjust cascade distances to be more granular
- Increase PCF sample count (4, 9, or 16)

### Poor Performance

**Symptom**: Low frame rate with shadows enabled

**Causes**:
- Too many cascades
- Shadow maps too large
- High PCF sample count
- Complex geometry in shadow pass

**Solutions**:
- Reduce cascade count (3 → 2)
- Reduce shadow map size (2048 → 1024)
- Reduce PCF samples (9 → 4)
- Implement shadow caster culling

## Examples

See `examples/shadow_demo.rs` for a complete demonstration including:
- Shadow map configuration
- Animated directional light
- Multiple objects casting shadows
- Real-time camera control
- Dynamic shadow updates

Run the example with:
```bash
cargo run --example shadow_demo
```

## Future Enhancements

Potential improvements for the shadow system:

1. **Point Light Shadows**: Omnidirectional shadow maps (cube maps)
2. **Soft Shadows**: Penumbra simulation with PCSS or VSM
3. **Shadow Fading**: Smooth transition at cascade boundaries
4. **Automatic Bias**: Adaptive bias based on surface angle
5. **Contact Hardening**: Sharper shadows near contact points
6. **Shadow Atlases**: Multiple lights sharing shadow map space
7. **Temporal Filtering**: Reduce flickering in dynamic scenes

## References

- [Microsoft DirectX Shadow Mapping](https://docs.microsoft.com/en-us/windows/win32/dxtecharts/cascaded-shadow-maps)
- [NVIDIA Cascaded Shadow Maps](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus)
- [Learn OpenGL - Shadow Mapping](https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping)
