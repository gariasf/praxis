# Shadow System

Comprehensive guide to shadow mapping implementation in the Praxis engine, including cascaded shadow maps (CSM), percentage closer filtering (PCF), and configuration options.

## Table of Contents

1. [Overview](#overview)
2. [Technical Architecture](#technical-architecture)
3. [Configuration](#configuration)
4. [Implementation Details](#implementation-details)
5. [Usage Guide](#usage-guide)
6. [Performance Optimization](#performance-optimization)
7. [Troubleshooting](#troubleshooting)
8. [Future Enhancements](#future-enhancements)

---

## Overview

The Praxis shadow system implements cascaded shadow maps (CSM) with percentage closer filtering (PCF) to provide realistic, soft shadows from directional lights. The system is designed for:

- **Quality**: Smooth shadows with minimal aliasing
- **Performance**: Configurable quality levels for different hardware
- **Flexibility**: Easy integration with existing rendering pipeline
- **Scalability**: Support for multiple cascades and configurable resolution

### Key Features

- **Cascaded Shadow Maps**: Multiple shadow map resolutions at different distances
- **PCF Filtering**: Configurable soft shadow edges (1, 4, 9, or 16 samples)
- **Automatic Light-Space Calculation**: Optimal shadow map fit for each cascade
- **Bias Configuration**: Adjustable bias to eliminate shadow acne
- **Integration**: Seamless integration with existing graphics pipeline

---

## Technical Architecture

### Two-Pass Rendering

Shadow mapping uses a two-pass approach:

#### Pass 1: Shadow Pass

Render the scene from the light's perspective to generate depth maps:

```
For each cascade:
  1. Calculate light-space view and projection matrices
  2. Set viewport to shadow map resolution
  3. Bind shadow framebuffer
  4. Render scene geometry (depth only, no colors)
  5. Store depth values in shadow map texture
```

#### Pass 2: Main Pass

Render the scene normally, sampling shadow maps for shadow determination:

```
For each fragment:
  1. Calculate fragment's view-space depth
  2. Select appropriate cascade based on depth
  3. Transform fragment position to light-space
  4. Perform PCF sampling on selected shadow map
  5. Apply shadow factor to lighting calculation
```

### Cascade System

The view frustum is divided into multiple cascades:

```
Camera Near Plane                                  Camera Far Plane
        │                                                  │
        ├──────────┬────────────────┬─────────────────────┤
        │Cascade 0 │   Cascade 1    │      Cascade 2      │
        │ (0-20m)  │   (20-100m)    │     (100-500m)      │
        └──────────┴────────────────┴─────────────────────┘
         High Res    Medium Res         Lower Res
```

Each cascade:
- Has its own shadow map texture (same resolution)
- Covers a different portion of the view frustum
- Has tighter world-space bounds → better effective resolution
- Uses its own light-space transformation matrix

### Data Flow

```
┌─────────────────────────────────────────────────┐
│          ShadowMapManager                       │
│  - Configuration (resolution, cascades, etc.)   │
│  - Shadow map textures (array of images)        │
│  - Shadow framebuffers (for rendering)          │
│  - Uniform buffer (matrices + config)           │
└──────────────┬──────────────────────────────────┘
               │
               │ update()
               ├─> Calculate cascade distances
               ├─> Compute light-space matrices
               ├─> Write uniforms to GPU buffer
               │
               ▼
┌─────────────────────────────────────────────────┐
│         Shadow Pass Rendering                   │
│  For each cascade:                              │
│    - Set viewport                               │
│    - Bind framebuffer                           │
│    - Draw scene geometry                        │
└──────────────┬──────────────────────────────────┘
               │
               ▼
┌─────────────────────────────────────────────────┐
│         Main Pass Rendering                     │
│  Bind shadow maps as textures                   │
│  Bind shadow uniform buffer                     │
│  Fragment shader:                               │
│    - Select cascade                             │
│    - Transform to light space                   │
│    - PCF sample shadow map                      │
│    - Apply shadow factor                        │
└─────────────────────────────────────────────────┘
```

---

## Configuration

### ShadowConfig Structure

```rust
pub struct ShadowConfig {
    /// Resolution of each shadow map (power of 2)
    pub shadow_map_size: u32,
    
    /// Number of cascades (1-4)
    pub cascade_count: u32,
    
    /// Distance from camera where each cascade ends (meters)
    pub cascade_distances: [f32; 4],
    
    /// Number of PCF samples (1, 4, 9, or 16)
    pub pcf_samples: u32,
    
    /// Shadow bias to prevent acne
    pub bias: f32,
}
```

### Default Configuration

```rust
ShadowConfig::default() = ShadowConfig {
    shadow_map_size: 1024,
    cascade_count: 3,
    cascade_distances: [20.0, 100.0, 500.0, 1000.0],
    pcf_samples: 4,
    bias: 0.005,
}
```

This provides a good balance between quality and performance for most scenes.

### Quality Presets

#### Low Quality (Mobile/Low-End)

```rust
ShadowConfig {
    shadow_map_size: 512,
    cascade_count: 2,
    cascade_distances: [30.0, 150.0, 500.0, 1000.0],
    pcf_samples: 1,
    bias: 0.01,
}
```

**Characteristics:**
- Memory: ~2 MB
- Performance: Excellent
- Quality: Hard shadows, visible aliasing

#### Medium Quality (Default)

```rust
ShadowConfig {
    shadow_map_size: 1024,
    cascade_count: 3,
    cascade_distances: [20.0, 100.0, 500.0, 1000.0],
    pcf_samples: 4,
    bias: 0.005,
}
```

**Characteristics:**
- Memory: ~12 MB
- Performance: Good
- Quality: Smooth shadows, minimal aliasing

#### High Quality (High-End Desktop)

```rust
ShadowConfig {
    shadow_map_size: 2048,
    cascade_count: 4,
    cascade_distances: [10.0, 50.0, 200.0, 800.0],
    pcf_samples: 9,
    bias: 0.003,
}
```

**Characteristics:**
- Memory: ~64 MB
- Performance: Moderate
- Quality: Very smooth shadows, excellent detail

#### Ultra Quality (Screenshots/Cinematics)

```rust
ShadowConfig {
    shadow_map_size: 4096,
    cascade_count: 4,
    cascade_distances: [10.0, 50.0, 200.0, 800.0],
    pcf_samples: 16,
    bias: 0.002,
}
```

**Characteristics:**
- Memory: ~256 MB
- Performance: Heavy
- Quality: Exceptional shadows

---

## Implementation Details

### Shadow Uniform Buffer

The shadow system uses a uniform buffer to pass data to shaders:

```rust
#[repr(C)]
struct ShadowUniforms {
    /// Light-space matrices for each cascade
    light_space_matrices: [Mat4; 4],
    
    /// Cascade split distances in view space
    cascade_splits: [f32; 4],
    
    /// Number of active cascades
    cascade_count: u32,
    
    /// Shadow bias
    bias: f32,
    
    /// PCF sample count
    pcf_samples: u32,
    
    _padding: u32,
}
```

### Light-Space Matrix Calculation

For each cascade, the system computes an optimal light-space transformation:

```rust
fn compute_light_space_matrix(
    light_direction: Vec3,
    cascade_near: f32,
    cascade_far: f32,
    camera_view: Mat4,
    camera_proj: Mat4,
) -> Mat4 {
    // 1. Extract frustum sub-volume for this cascade
    let frustum_corners = extract_frustum_corners(
        camera_view,
        camera_proj,
        cascade_near,
        cascade_far,
    );
    
    // 2. Create light view matrix
    let light_pos = frustum_center - light_direction * 100.0;
    let light_view = Mat4::look_at_rh(
        light_pos,
        frustum_center,
        Vec3::Y,
    );
    
    // 3. Transform corners to light space
    let light_space_corners: Vec<Vec3> = frustum_corners
        .iter()
        .map(|&corner| (light_view * corner.extend(1.0)).truncate())
        .collect();
    
    // 4. Calculate tight AABB in light space
    let (min_bounds, max_bounds) = calculate_aabb(&light_space_corners);
    
    // 5. Create orthographic projection
    let light_proj = Mat4::orthographic_rh(
        min_bounds.x, max_bounds.x,
        min_bounds.y, max_bounds.y,
        min_bounds.z, max_bounds.z,
    );
    
    // 6. Combine
    light_proj * light_view
}
```

### PCF Sampling

The fragment shader implements PCF with configurable sample counts:

```glsl
float shadow_pcf(vec3 light_space_pos, int cascade_index, int sample_count) {
    float shadow = 0.0;
    vec2 texel_size = 1.0 / textureSize(shadow_maps[cascade_index], 0);
    
    // Determine kernel size based on sample count
    int kernel_size = int(sqrt(float(sample_count)));
    float kernel_half = float(kernel_size) / 2.0;
    
    // Sample in a grid pattern
    for (int x = 0; x < kernel_size; ++x) {
        for (int y = 0; y < kernel_size; ++y) {
            vec2 offset = vec2(
                float(x) - kernel_half,
                float(y) - kernel_half
            ) * texel_size;
            
            float depth = texture(
                shadow_maps[cascade_index],
                light_space_pos.xy + offset
            ).r;
            
            // Compare with bias
            shadow += (light_space_pos.z - bias) > depth ? 0.0 : 1.0;
        }
    }
    
    return shadow / float(sample_count);
}
```

### Cascade Selection

The fragment shader selects the appropriate cascade based on view-space depth:

```glsl
int select_cascade(float view_depth) {
    for (int i = 0; i < cascade_count; ++i) {
        if (view_depth < cascade_splits[i]) {
            return i;
        }
    }
    return cascade_count - 1;
}
```

---

## Usage Guide

### Basic Setup

```rust
use praxis_graphics::shadow::{ShadowMapManager, ShadowConfig};
use praxis_math::{Mat4, Vec3};

// 1. Create shadow manager during initialization
let shadow_manager = ShadowMapManager::new(
    memory_allocator.clone(),
    ShadowConfig::default(),
)?;

// 2. Each frame: update with light direction
let light_direction = Vec3::new(0.3, -0.8, 0.5).normalize();
shadow_manager.update(
    light_direction,
    camera_view_matrix,
    camera_projection_matrix,
)?;

// 3. Shadow pass: render scene to shadow maps
for cascade_idx in 0..shadow_manager.cascade_count() {
    let framebuffer = shadow_manager.shadow_framebuffers()[cascade_idx];
    
    // Begin render pass with shadow framebuffer
    // Render scene geometry (depth only)
    // End render pass
}

// 4. Main pass: shadows are automatically applied
// The shadow maps are bound at descriptor set bindings 4-8
// The shader automatically samples them
```

### Integration with Existing Rendering

The shadow system integrates with the main rendering pipeline through descriptor sets:

```rust
// Shadow resources are bound to set 0:
// Binding 4: Shadow uniform buffer
// Binding 5: Shadow map cascade 0
// Binding 6: Shadow map cascade 1
// Binding 7: Shadow map cascade 2
// Binding 8: Shadow map cascade 3

let descriptor_set = create_descriptor_set(
    &descriptor_set_allocator,
    descriptor_set_layout.clone(),
)?;

// Bind shadow uniform buffer
descriptor_set.write(&[
    WriteDescriptorSet::buffer(4, shadow_manager.uniform_buffer()),
])?;

// Bind shadow map textures
for (i, shadow_map_view) in shadow_manager.shadow_map_views().iter().enumerate() {
    descriptor_set.write(&[
        WriteDescriptorSet::image_view_sampler(
            5 + i as u32,
            shadow_map_view.clone(),
            shadow_sampler.clone(),
        ),
    ])?;
}
```

---

## Performance Optimization

### Memory Usage

**Formula:** `cascade_count × shadow_map_size² × 4 bytes`

Examples:
- 3 cascades @ 1024×1024: 12 MB
- 4 cascades @ 2048×2048: 64 MB
- 2 cascades @ 512×512: 2 MB

**Optimization strategies:**
1. Reduce cascade count for distant views
2. Lower resolution for far cascades (requires multiple shadow map sizes)
3. Use texture compression (not currently implemented)

### Rendering Performance

**Shadow pass cost factors:**
- Triangle count in scene
- Number of cascades
- Shadow map resolution
- Overdraw (objects appearing in multiple cascades)

**Optimization strategies:**
1. **Culling:** Don't render objects outside cascade bounds
2. **LOD:** Use lower detail meshes for shadow rendering
3. **Static caching:** Pre-render shadows for static objects
4. **Temporal reuse:** Update cascades at different rates

### Fragment Shader Performance

**Main pass cost factors:**
- Screen resolution (fragments processed)
- PCF sample count (1, 4, 9, or 16)
- Cascade selection overhead

**Optimization strategies:**
1. **Reduce PCF samples:** Use 4 instead of 9 or 16
2. **Early out:** Skip shadow calculation for unlit areas
3. **Half-resolution:** Render shadows at half resolution and upscale
4. **Screen-space optimization:** Skip distant pixels

### Recommended Settings by Target

| Target        | Resolution | Cascades | PCF | Total Cost |
|--------------|------------|----------|-----|------------|
| Mobile       | 512        | 2        | 1   | ~0.5-1ms   |
| Low Desktop  | 1024       | 2        | 4   | ~1-2ms     |
| Mid Desktop  | 1024       | 3        | 4   | ~2-3ms     |
| High Desktop | 2048       | 3        | 9   | ~4-6ms     |
| Ultra        | 2048       | 4        | 16  | ~8-12ms    |

---

## Troubleshooting

### Shadow Acne

**Symptoms:**
- Striped/moire pattern on surfaces
- Self-shadowing artifacts
- Flickering shadows

**Solutions:**
1. Increase `bias` in `ShadowConfig`
2. Increase shadow map resolution
3. Use slope-scale bias (hardware feature, enabled by default)
4. Adjust near/far planes of light projection

### Peter Panning

**Symptoms:**
- Shadows detached from objects
- Gap between object and shadow
- Shadows appear to float

**Solutions:**
1. Decrease `bias` in `ShadowConfig`
2. Find balance between acne and peter panning
3. Use normal offset bias (not currently implemented)

### Blocky/Pixelated Shadows

**Symptoms:**
- Jagged shadow edges
- Visible shadow map pixels
- Poor quality at close range

**Solutions:**
1. Increase shadow map resolution
2. Add more cascades
3. Adjust cascade distances for better near-camera coverage
4. Increase PCF sample count

### Cascade Transitions Visible

**Symptoms:**
- Visible line where cascades meet
- Sudden shadow quality change
- Popping when moving between cascades

**Solutions:**
1. Increase cascade overlap (not currently configurable)
2. Blend between cascades in shader (not currently implemented)
3. Adjust cascade distances for smoother transition

### Performance Issues

**Symptoms:**
- Low frame rate with shadows enabled
- GPU bottleneck
- Long shadow pass time

**Solutions:**
1. Reduce shadow map resolution
2. Decrease cascade count
3. Reduce PCF sample count
4. Implement shadow caster culling
5. Use lower LOD meshes for shadow rendering

---

## Future Enhancements

### Planned Features

1. **Point Light Shadows**
   - Omnidirectional shadow maps (cube maps)
   - Up to 4-6 point lights with shadows
   - Performance: ~1-2ms per light

2. **Cascade Blending**
   - Smooth transition between cascades
   - Eliminates visible cascade boundaries
   - Cost: +0.2-0.5ms

3. **PCSS (Percentage Closer Soft Shadows)**
   - Variable penumbra based on distance
   - Contact hardening effect
   - More realistic soft shadows
   - Cost: +1-3ms depending on quality

4. **VSM (Variance Shadow Maps)**
   - Alternative to PCF
   - Faster soft shadows
   - Can introduce light bleeding
   - Cost: ~0.5-1ms

5. **Adaptive Bias**
   - Automatic bias calculation based on surface angle
   - Eliminates manual bias tuning
   - Cost: negligible

6. **Shadow Atlases**
   - Multiple lights sharing shadow map space
   - Better memory efficiency
   - Support for many shadowed lights

7. **Temporal Filtering**
   - Reduce shadow flickering in dynamic scenes
   - Accumulate samples across frames
   - Requires motion vectors

8. **Ray-Traced Shadows**
   - Ultimate quality shadows
   - Requires RTX hardware
   - Cost: variable, ~2-10ms

### Research Areas

- **Machine Learning Denoising:** Use AI to reduce sample count
- **Hybrid Ray Tracing:** Combine shadow maps with selective ray tracing
- **Voxel-Based Shadows:** For large-scale scenes
- **Signed Distance Fields:** For sharp geometric shadows

---

## References

- [Microsoft DirectX Cascaded Shadow Maps](https://docs.microsoft.com/en-us/windows/win32/dxtecharts/cascaded-shadow-maps)
- [NVIDIA Parallel-Split Shadow Maps](https://developer.nvidia.com/gpugems/gpugems3/part-ii-light-and-shadows/chapter-10-parallel-split-shadow-maps-programmable-gpus)
- [Learn OpenGL - Shadow Mapping](https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping)
- [Percentage-Closer Filtering](https://developer.nvidia.com/gpugems/gpugems/part-ii-lighting-and-shadows/chapter-11-shadow-map-antialiasing)

---

## See Also

- [Beginners Guide](BEGINNERS_GUIDE.md) - Introduction to shadow mapping concepts
- [Rendering Explained](RENDERING_EXPLAINED.md) - Complete rendering pipeline
- [Shadow Demo Example](../examples/shadow_demo.rs) - Working code example
