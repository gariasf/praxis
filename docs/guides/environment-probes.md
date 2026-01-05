# Environment Probes Guide

## Overview

Environment probes provide realistic reflections and ambient lighting through image-based lighting (IBL). This guide covers the implementation, configuration, and best practices for using environment probes in Praxis.

## What are Environment Probes?

Environment probes capture the surrounding scene into a cubemap and precompute lighting data for realistic reflections and ambient lighting. They enable physically-based rendering (PBR) to accurately simulate how surfaces interact with their environment.

### Key Features

- **Cubemap Capture**: 6-face rendering from probe position
- **Diffuse Irradiance**: Precomputed ambient lighting
- **Specular Reflection**: Multi-roughness prefiltering with 5 mip levels
- **BRDF Integration**: Split-sum approximation lookup table
- **Real-time Updates**: Four update modes for different scenarios
- **Multiple Probes**: Support for up to 8 simultaneous probes
- **Spatial Queries**: Distance-based probe selection

## Core Components

### `EnvironmentProbe` Component

ECS component marking entities as environment probes:

```rust
use praxis_ecs::{World, Transform};
use praxis_graphics::EnvironmentProbe;
use praxis_math::Vec3;

// Spawn probe entity
world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    EnvironmentProbe::new("main_probe")
        .with_resolution(512)
        .with_influence_radius(50.0)
        .with_update_every_n_frames(60),
));
```

**Configuration Options**:
- `resolution`: Cubemap face resolution (256, 512, 1024)
- `near_clip` / `far_clip`: Capture frustum bounds
- `update_mode`: Update frequency/trigger
- `influence_radius`: Spatial extent of probe effect
- `intensity`: IBL contribution multiplier

### `EnvironmentProbeManager`

Central management system for all probes:

```rust
use praxis_graphics::{EnvironmentProbeManager, EnvironmentProbeConfig};

// Create probe manager
let mut probe_manager = EnvironmentProbeManager::new(
    device,
    allocator,
    command_buffer_allocator,
    queue,
)?;

// Add probe
let config = EnvironmentProbeConfig {
    position: Vec3::new(0.0, 2.0, 0.0),
    resolution: 512,
    near_clip: 0.1,
    far_clip: 100.0,
    update_mode: ProbeUpdateMode::EveryNFrames(60),
};
probe_manager.add_probe("main_probe".to_string(), config)?;
```

**Key Methods**:
- `add_probe()`: Create and configure new probe
- `update_probes()`: Process probe updates based on mode
- `get_nearest_probe()`: Find closest probe to a point
- `get_ibl_uniforms()`: Collect data from up to 8 active probes

## Update Modes

Environment probes support four update strategies:

### 1. Once

Captures environment once at creation, then never updates:

```rust
EnvironmentProbe::new("static_probe")
    .with_update_mode(ProbeUpdateMode::Once)
```

**Best for**: Static scenes, indoor environments
**Cost**: Zero runtime cost after initial capture

### 2. EveryNFrames(n)

Periodic updates every N frames:

```rust
EnvironmentProbe::new("periodic_probe")
    .with_update_every_n_frames(60)  // Update once per second at 60fps
```

**Best for**: Slowly changing scenes, outdoor lighting
**Cost**: Amortized over time (~0.3ms/frame for N=60)

### 3. Manual

Updates only when explicitly marked dirty:

```rust
let mut probe = EnvironmentProbe::new("manual_probe")
    .with_update_mode(ProbeUpdateMode::Manual);

// Later, when scene changes
probe.mark_dirty();
```

**Best for**: Event-driven updates, explicit scene changes
**Cost**: Only when marked dirty

### 4. Continuous

Updates every frame automatically:

```rust
EnvironmentProbe::new("dynamic_probe")
    .with_update_mode(ProbeUpdateMode::Continuous)
```

**Best for**: Highly dynamic scenes, hero reflective objects
**Cost**: ~30-40ms/frame (expensive, use sparingly)

## IBL Precomputation Pipeline

Environment probes use a sophisticated precomputation pipeline for realistic lighting:

### 1. Cubemap Capture

The probe renders the scene six times from its position:

```
+X (Right)   →  View matrix for positive X axis
-X (Left)    →  View matrix for negative X axis
+Y (Up)      →  View matrix for positive Y axis
-Y (Down)    →  View matrix for negative Y axis
+Z (Forward) →  View matrix for positive Z axis
-Z (Back)    →  View matrix for negative Z axis
```

Each face uses a 90-degree FOV perspective projection.

### 2. Diffuse Irradiance Convolution

Computes ambient lighting by convolving the environment over a hemisphere:

- **Resolution**: 32×32 per face (low resolution for efficiency)
- **Algorithm**: Cosine-weighted hemisphere sampling
- **Shader**: `ibl_irradiance.frag`
- **Cost**: ~2-3ms per probe

The irradiance map provides the diffuse ambient term for PBR materials.

### 3. Specular Prefiltering

Importance samples using GGX distribution for varying roughness levels:

- **Mip Levels**: 5 levels for roughness 0.0, 0.25, 0.5, 0.75, 1.0
- **Samples**: 1024 samples per pixel for quality
- **Shader**: `ibl_prefilter.frag`
- **Cost**: ~15-20ms per probe

The prefiltered map provides specular reflections that vary with material roughness.

### 4. BRDF Integration

Precomputes the specular BRDF integral using split-sum approximation:

- **Format**: 2D lookup table (512×512)
- **Indices**: NdotV (x-axis), roughness (y-axis)
- **Data**: (scale, bias) for Fresnel approximation
- **Cost**: ~50ms (computed once, shared across all probes)

## Shader Integration

### Uniforms

Environment probe data is passed to shaders via uniforms:

```glsl
struct ProbeData {
    vec4 position_and_radius;  // xyz: position, w: influence radius
};

layout(set = 0, binding = 4) uniform IblUniforms {
    ProbeData probes[8];
    uint probe_count;
    float ibl_intensity;
} ibl;
```

### Textures

Three textures per probe:

```glsl
layout(set = 0, binding = 5) uniform samplerCube irradiance_map;
layout(set = 0, binding = 6) uniform samplerCube prefiltered_map;
layout(set = 0, binding = 7) uniform sampler2D brdf_lut;
```

### PBR Integration

Sample the IBL textures in your fragment shader:

```glsl
// Diffuse ambient
vec3 irradiance = texture(irradiance_map, normal).rgb;
vec3 diffuse = irradiance * albedo;

// Specular reflection
float lod = roughness * 4.0;  // 5 mip levels (0-4)
vec3 prefiltered = textureLod(prefiltered_map, reflect_dir, lod).rgb;
vec2 brdf = texture(brdf_lut, vec2(max(dot(normal, view), 0.0), roughness)).rg;
vec3 specular = prefiltered * (F * brdf.x + brdf.y);

// Combine
vec3 ambient = (diffuse + specular) * ibl.ibl_intensity;
```

## Probe Placement Guidelines

### General Principles

1. **One probe per room**: Indoor scenes benefit from per-room probes
2. **Transition zones**: Place probes at boundaries for smooth blending
3. **Height placement**: Position at average object height (1-2m for characters)
4. **Overlapping influence**: Use overlapping radii for smooth transitions

### Example Configurations

**Small Room (5×5m)**:
```rust
EnvironmentProbe::new("room_probe")
    .with_resolution(256)
    .with_influence_radius(4.0)
    .with_update_mode(ProbeUpdateMode::Once)
```

**Large Hall (20×20m)**:
```rust
EnvironmentProbe::new("hall_probe")
    .with_resolution(512)
    .with_influence_radius(15.0)
    .with_update_every_n_frames(120)
```

**Outdoor Scene**:
```rust
EnvironmentProbe::new("outdoor_probe")
    .with_resolution(1024)
    .with_influence_radius(100.0)
    .with_update_every_n_frames(300)  // Update for time-of-day
```

## Memory and Performance

### Memory Footprint (per probe)

| Resolution | Environment | Irradiance | Prefiltered | Total |
|-----------|-------------|------------|-------------|-------|
| 256×256   | ~1.5 MB     | ~25 KB     | ~2 MB       | ~3.5 MB |
| 512×512   | ~6 MB       | ~25 KB     | ~8 MB       | ~14 MB |
| 1024×1024 | ~24 MB      | ~25 KB     | ~32 MB      | ~56 MB |

BRDF LUT: ~512 KB (shared across all probes)

### Capture Cost (512×512 resolution)

- **Cubemap Capture**: 6 render passes (~8-12ms)
- **Irradiance Convolution**: ~2-3ms
- **Prefiltering**: ~15-20ms (5 mip levels)
- **Total**: ~30-40ms per full update

### Optimization Tips

1. **Use appropriate resolution**: 256-512 is sufficient for most scenes
2. **Minimize update frequency**: Use Once or EveryNFrames for static scenes
3. **Limit active probes**: Up to 8 probes per frame
4. **Share probes**: Reuse probes for similar areas
5. **LOD strategy**: Higher resolution for nearby probes, lower for distant

## Advanced Techniques

### Probe Blending

When multiple probes overlap, blend based on distance:

```rust
let probe1_data = probe_manager.get_nearest_probe(position);
let probe2_data = probe_manager.get_second_nearest_probe(position);

let weight1 = 1.0 - (distance1 / probe1.influence_radius);
let weight2 = 1.0 - (distance2 / probe2.influence_radius);
let total = weight1 + weight2;
let blend = weight1 / total;
```

### Parallax Correction

For indoor environments, use box-projected cubemaps for accurate reflections:

```glsl
vec3 parallax_correct_direction(vec3 dir, vec3 pos, vec3 box_min, vec3 box_max) {
    vec3 first = (box_max - pos) / dir;
    vec3 second = (box_min - pos) / dir;
    vec3 furthest = max(first, second);
    float distance = min(min(furthest.x, furthest.y), furthest.z);
    return dir * distance + pos;
}
```

### Dynamic Probe Updates

Selectively update probes based on scene changes:

```rust
// Mark probe dirty when objects move
if entity_moved_significantly {
    if let Some(probe) = probe_manager.get_nearest_probe(entity.position()) {
        probe.mark_dirty();
    }
}
```

## Troubleshooting

### Dark or Missing Reflections

- **Check intensity**: Increase `intensity` parameter
- **Verify probe placement**: Ensure probes cover the area
- **Update mode**: Use Continuous temporarily to debug
- **Shader binding**: Verify IBL textures are bound correctly

### Performance Issues

- **Reduce resolution**: Use 256 or 512 instead of 1024
- **Increase update interval**: Use higher N in EveryNFrames
- **Limit active probes**: Disable distant probes
- **Profile capture**: Check if capture is the bottleneck

### Seams or Artifacts

- **Use overlapping influence**: Increase influence radius
- **Enable probe blending**: Interpolate between probes
- **Check normal maps**: Ensure normals are in correct space
- **Verify filtering**: Use linear filtering on IBL textures

## Example Usage

Complete example demonstrating environment probes:

```rust
use praxis_ecs::{World, Transform};
use praxis_graphics::{EnvironmentProbe, EnvironmentProbeManager};
use praxis_math::Vec3;

// Setup
let mut world = World::new();
let mut probe_manager = EnvironmentProbeManager::new(
    device, allocator, cmd_allocator, queue
)?;

// Spawn probes
world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    EnvironmentProbe::new("center_probe")
        .with_resolution(512)
        .with_influence_radius(20.0)
        .with_update_every_n_frames(60),
));

world.spawn((
    Transform::from_xyz(30.0, 2.0, 0.0),
    EnvironmentProbe::new("side_probe")
        .with_resolution(256)
        .with_influence_radius(15.0)
        .with_update_mode(ProbeUpdateMode::Once),
));

// In render loop
probe_manager.update_probes();
let ibl_uniforms = probe_manager.get_ibl_uniforms();
// Pass ibl_uniforms to shaders
```

## See Also

- [Rendering Guide](rendering.md)
- [PBR Materials](../reference/materials.md)
- [Advanced Lighting](../advanced_lighting.md)

## References

- Karis, Brian (2013). "Real Shading in Unreal Engine 4"
- Lazarov, Dimitar (2013). "Getting More Physical in Call of Duty: Black Ops II"
- Lagarde, Sébastien (2014). "Moving Frostbite to Physically Based Rendering"
