# Environment Probes

Environment probes provide realistic reflections and ambient lighting through image-based lighting (IBL). This guide covers environment probe implementation, configuration, and best practices in Praxis.

## Overview

Environment probes capture the surrounding scene into a cubemap and precompute lighting data for physically-based rendering (PBR).

### Key Features

- **Cubemap Capture**: 6-face rendering from probe position
- **Diffuse Irradiance**: Precomputed ambient lighting
- **Specular Reflection**: Multi-roughness prefiltering (5 mip levels)
- **BRDF Integration**: Split-sum approximation lookup table
- **Real-time Updates**: Four update modes for different scenarios
- **Multiple Probes**: Support for up to 8 simultaneous probes
- **Spatial Queries**: Distance-based probe selection

## Core Components

### EnvironmentProbe Component

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

### EnvironmentProbeManager

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

### 1. Once

Captures environment once at creation, never updates:

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
    .with_update_every_n_frames(60)  // Once per second at 60fps
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

### 1. Cubemap Capture

Renders the scene six times from probe position:

```
+X (Right)   →  View matrix for positive X axis
-X (Left)    →  View matrix for negative X axis
+Y (Up)      →  View matrix for positive Y axis
-Y (Down)    →  View matrix for negative Y axis
+Z (Forward) →  View matrix for positive Z axis
-Z (Back)    →  View matrix for negative Z axis
```

Each face uses 90-degree FOV perspective projection.

### 2. Diffuse Irradiance Convolution

Computes ambient lighting by convolving environment over hemisphere:

**Algorithm**:
```glsl
vec3 irradiance = vec3(0.0);
int samples = 0;

// Integrate over hemisphere around normal N
for (float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
    for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
        vec3 sample_vec = /* hemisphere sample */;
        vec3 env_color = texture(environment_map, sample_vec).rgb;
        irradiance += env_color * cos(theta) * sin(theta);
        samples++;
    }
}

irradiance = PI * irradiance / float(samples);
```

**Output**: Low-resolution (32×32) cubemap containing ambient lighting for each direction  
**Cost**: ~2-3ms per probe

### 3. Specular Prefiltering

Importance samples using GGX distribution for varying roughness:

**Algorithm**:
```glsl
vec3 prefiltered_color = vec3(0.0);
float total_weight = 0.0;

for (uint i = 0; i < SAMPLE_COUNT; i++) {
    vec2 xi = hammersley(i, SAMPLE_COUNT);
    vec3 H = importance_sample_ggx(xi, N, roughness);
    vec3 L = reflect(-V, H);
    
    float n_dot_l = dot(N, L);
    if (n_dot_l > 0.0) {
        vec3 env_color = texture(environment_map, L).rgb;
        prefiltered_color += env_color * n_dot_l;
        total_weight += n_dot_l;
    }
}

prefiltered_color /= total_weight;
```

**Output**: Full-resolution cubemap with 5 mip levels for roughness [0.0, 0.25, 0.5, 0.75, 1.0]  
**Cost**: ~15-20ms per probe

### 4. BRDF Integration

Precomputes specular BRDF integral using split-sum approximation:

**Format**: 2D lookup table (512×512)
- **X-axis**: NdotV (angle between normal and view)
- **Y-axis**: Roughness
- **Data**: (scale, bias) for Fresnel approximation

**Output**: Shared across all probes (computed once)  
**Cost**: ~50ms (one-time)

## Shader Integration

### PBR Integration

Sample IBL textures in fragment shader:

```glsl
// Diffuse ambient
vec3 irradiance = texture(irradiance_map, normal).rgb;
vec3 k_d = (1.0 - F) * (1.0 - metallic);
vec3 diffuse = irradiance * albedo * k_d;

// Specular reflection
vec3 R = reflect(-view_dir, normal);
float lod = roughness * MAX_REFLECTION_LOD;  // 5 mip levels (0-4)
vec3 prefiltered = textureLod(prefiltered_map, R, lod).rgb;

// BRDF lookup
vec2 brdf = texture(brdf_lut, vec2(max(dot(normal, view_dir), 0.0), roughness)).rg;
vec3 specular = prefiltered * (F0 * brdf.x + brdf.y);

// Combine
vec3 ambient = (diffuse + specular) * ao * ibl_intensity;
```

### Uniforms

Environment probe data passed to shaders:

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

**BRDF LUT**: ~512 KB (shared across all probes)

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

For indoor environments, project reflection ray to room bounding box:

```glsl
vec3 parallax_correction(vec3 ray_dir, vec3 probe_pos, vec3 world_pos, 
                         vec3 box_min, vec3 box_max) {
    vec3 first = (box_max - world_pos) / ray_dir;
    vec3 second = (box_min - world_pos) / ray_dir;
    vec3 furthest = max(first, second);
    float distance = min(min(furthest.x, furthest.y), furthest.z);
    
    vec3 intersection = world_pos + ray_dir * distance;
    return normalize(intersection - probe_pos);
}
```

### Dynamic Probe Updates

Selectively update probes based on scene changes:

```rust
// Mark probe dirty when objects move significantly
if entity_moved_significantly {
    if let Some(probe) = probe_manager.get_nearest_probe(entity.position()) {
        probe.mark_dirty();
    }
}
```

## Troubleshooting

### Dark or Missing Reflections

- Check intensity parameter
- Verify probe placement covers area
- Use Continuous mode temporarily to debug
- Verify IBL textures are bound correctly

### Performance Issues

- Reduce resolution (use 256 or 512 instead of 1024)
- Increase update interval (higher N in EveryNFrames)
- Limit active probes (disable distant probes)
- Profile capture to identify bottleneck

### Seams or Artifacts

- Use overlapping influence radii
- Enable probe blending (interpolate between probes)
- Check normal maps are in correct space
- Verify linear filtering on IBL textures

## Complete Example

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

## Examples

```bash
# Environment probe demo
cargo run --example environment_probe_demo

# Advanced lighting with IBL
cargo run --example advanced_lighting_demo
```

## See Also

- [Forward Rendering](forward-rendering.md) - Basic rendering pipeline
- [Deferred Rendering](deferred-rendering.md) - Multi-pass rendering
- [HDR and Tone Mapping](hdr-tonemapping.md) - High dynamic range

## References

- [Karis, Brian (2013). "Real Shading in Unreal Engine 4"](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)
- [Lagarde, Sébastien (2014). "Moving Frostbite to PBR"](https://seblagarde.files.wordpress.com/2015/07/course_notes_moving_frostbite_to_pbr_v32.pdf)
- [Learn OpenGL - IBL](https://learnopengl.com/PBR/IBL/Diffuse-irradiance)
- [Real-Time Rendering, 4th Edition](http://www.realtimerendering.com/) - Chapter 11: Image-Based Effects
