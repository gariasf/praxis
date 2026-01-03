# Environment Probe System

The environment probe system provides image-based lighting (IBL) for realistic reflections and ambient lighting in the Praxis engine. This document describes the architecture, usage, and implementation details of the system.

## Overview

Environment probes capture the surrounding scene as a cubemap and precompute lighting data for physically-based rendering. They enable:

- **Reflections**: Accurate reflections on metallic and glossy surfaces
- **Ambient Lighting**: Environment-aware ambient lighting that matches the scene
- **Indirect Lighting**: Approximation of indirect lighting without expensive path tracing

## Architecture

### Components

#### `EnvironmentProbe` (ECS Component)

The `EnvironmentProbe` component marks an entity as an environment probe. It stores configuration data:

```rust
use praxis_ecs::{EnvironmentProbe, Transform};
use praxis_math::Vec3;

world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    EnvironmentProbe::new("main_probe")
        .with_resolution(512)
        .with_influence_radius(50.0)
        .with_update_every_n_frames(60),
));
```

**Properties:**
- `id`: Unique identifier for the probe
- `resolution`: Cubemap resolution per face (256, 512, 1024, etc.)
- `near_clip`/`far_clip`: Capture frustum bounds
- `update_mode`: How the probe updates (Once, EveryNFrames, Manual, Continuous)
- `enabled`: Whether the probe is active
- `influence_radius`: Distance at which the probe affects objects
- `intensity`: Multiplier for the probe's contribution

#### `EnvironmentProbeManager` (Graphics System)

The `EnvironmentProbeManager` handles probe creation, cubemap capture, and IBL precomputation:

```rust
use praxis_graphics::{EnvironmentProbeManager, EnvironmentProbeConfig};

let mut probe_manager = EnvironmentProbeManager::new(
    device,
    allocator,
    command_buffer_allocator,
    queue,
)?;

// Add a probe
let config = EnvironmentProbeConfig {
    position: Vec3::new(0.0, 2.0, 0.0),
    resolution: 256,
    near_clip: 0.1,
    far_clip: 100.0,
    update_mode: ProbeUpdateMode::Once,
};

probe_manager.add_probe("main_probe".to_string(), config)?;
```

### IBL Data Structure

Each probe generates three key resources:

1. **Environment Map**: Original HDR cubemap captured from the scene
2. **Irradiance Map**: Precomputed diffuse irradiance (32x32 resolution)
3. **Prefiltered Map**: Specular reflection map with roughness mipmaps (5 levels)
4. **BRDF LUT**: Shared 2D lookup table for split-sum approximation (512x512)

## Update Modes

### Once
Captures the environment once when created. Best for static scenes.

```rust
EnvironmentProbe::new("static_probe").with_update_once()
```

### EveryNFrames
Updates periodically every N frames. Good for slowly changing environments.

```rust
EnvironmentProbe::new("periodic_probe").with_update_every_n_frames(60)
```

### Manual
Updates only when explicitly requested. Useful for event-driven updates.

```rust
let mut probe = EnvironmentProbe::new("manual_probe").with_update_manual();
// Later, in code:
probe.mark_dirty(); // Will update on next frame
```

### Continuous
Updates every frame. Expensive but necessary for highly dynamic scenes.

```rust
EnvironmentProbe::new("dynamic_probe").with_update_continuous()
```

## Capture Process

The environment probe capture follows these steps:

### 1. Cubemap Capture

The scene is rendered from the probe's position to 6 cubemap faces:
- +X, -X (right, left)
- +Y, -Y (top, bottom)
- +Z, -Z (forward, back)

Each face uses a 90-degree FOV perspective projection.

```rust
use praxis_graphics::EnvironmentProbeCapture;

let capture = EnvironmentProbeCapture::new(
    probe_position,
    near_clip,
    far_clip,
);

// Render each face
for face in 0..6 {
    let view = capture.get_face_view(face);
    let proj = capture.get_projection();
    // Render scene with view and proj matrices
}
```

### 2. Irradiance Convolution

The environment map is convolved over a hemisphere to compute diffuse irradiance:

```glsl
// Sample the hemisphere around the normal
for (float phi = 0.0; phi < 2.0 * PI; phi += sample_delta) {
    for (float theta = 0.0; theta < 0.5 * PI; theta += sample_delta) {
        vec3 sample_vec = compute_hemisphere_sample(phi, theta, N);
        irradiance += texture(environment_map, sample_vec).rgb * cos(theta) * sin(theta);
    }
}
```

**Result**: Low-resolution (32x32) cubemap containing ambient lighting for each direction.

### 3. Specular Prefiltering

The environment map is prefiltered with multiple roughness levels using importance sampling:

```glsl
for (uint i = 0; i < SAMPLE_COUNT; i++) {
    vec2 xi = hammersley(i, SAMPLE_COUNT);
    vec3 H = importance_sample_ggx(xi, N, roughness);
    vec3 L = reflect(-V, H);
    
    if (dot(N, L) > 0.0) {
        prefiltered_color += texture(environment_map, L).rgb * dot(N, L);
    }
}
```

**Result**: Full-resolution cubemap with 5 mipmap levels representing roughness values [0.0, 0.25, 0.5, 0.75, 1.0].

### 4. BRDF Integration

A 2D lookup table is generated for the split-sum approximation of the BRDF integral:

```rust
for y in 0..LUT_SIZE {
    for x in 0..LUT_SIZE {
        let roughness = x / LUT_SIZE;
        let ndotv = y / LUT_SIZE;
        let (scale, bias) = integrate_brdf(ndotv, roughness);
        lut_data[y][x] = (scale, bias);
    }
}
```

**Result**: 512x512 texture storing (scale, bias) values for different roughness and viewing angles.

## Shader Integration

### Vertex Shader

Pass world-space position and normal to fragment shader:

```glsl
layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_world_normal;

void main() {
    v_world_pos = (u_model * vec4(position, 1.0)).xyz;
    v_world_normal = normalize((u_model * vec4(normal, 0.0)).xyz);
    gl_Position = u_view_proj * vec4(v_world_pos, 1.0);
}
```

### Fragment Shader

Sample IBL textures for diffuse and specular contributions:

```glsl
// Diffuse irradiance
vec3 irradiance = texture(u_irradiance_map, N).rgb;
vec3 diffuse = irradiance * albedo;

// Specular reflection
vec3 R = reflect(-V, N);
float lod = roughness * MAX_REFLECTION_LOD;
vec3 prefiltered_color = textureLod(u_prefiltered_map, R, lod).rgb;

// BRDF lookup
vec2 brdf = texture(u_brdf_lut, vec2(max(dot(N, V), 0.0), roughness)).rg;
vec3 specular = prefiltered_color * (F0 * brdf.x + brdf.y);

// Combine
vec3 ambient = (diffuse + specular) * ao;
```

## Performance Considerations

### Resolution Trade-offs

| Resolution | Memory Usage | Quality | Use Case |
|------------|--------------|---------|----------|
| 128x128    | ~0.5 MB      | Low     | Background objects, distant probes |
| 256x256    | ~2 MB        | Medium  | Standard quality, general use |
| 512x512    | ~8 MB        | High    | Hero objects, close-up reflections |
| 1024x1024  | ~32 MB       | Very High | Showcase scenes, high-end systems |

### Update Frequency

- **Once**: No runtime cost after initial capture
- **EveryNFrames(60)**: ~1/60th of capture cost per frame
- **Manual**: Cost only when explicitly triggered
- **Continuous**: Full capture cost every frame (expensive!)

### Optimization Tips

1. **Use appropriate resolutions**: Start with 256x256 and increase only where needed
2. **Limit update frequency**: Use EveryNFrames for slowly changing scenes
3. **Spatial partitioning**: Place probes strategically, don't over-populate
4. **Influence radius**: Set appropriate bounds to avoid overlapping probes
5. **LOD system**: Use lower-resolution probes for distant objects

## Probe Placement Guidelines

### Static Scenes
- Place probes at key locations with distinct lighting
- One probe per room/area is often sufficient
- Higher resolution for important areas

### Dynamic Scenes
- Use Manual update mode and trigger on significant changes
- Continuous mode only for hero objects in highly dynamic environments
- Consider temporal interpolation for smooth transitions

### Open Environments
- Place probes in a grid pattern (10-20 units apart)
- Use larger influence radius
- Lower resolution acceptable for large outdoor areas

### Indoor Environments
- One probe per room minimum
- Additional probes near light sources
- Higher resolution for reflective surfaces (metal, glass)

## Blending and Interpolation

When multiple probes overlap, their contributions should be blended:

```rust
// Find nearest probes
let probes = probe_manager.get_probes_affecting(position);

// Weight by inverse distance
let mut total_weight = 0.0;
let mut blended_ibl = IblContribution::default();

for probe in probes {
    let distance = probe.position.distance(position);
    let weight = 1.0 / (distance + 1.0);
    
    if distance < probe.influence_radius {
        blended_ibl += probe.ibl_data() * weight;
        total_weight += weight;
    }
}

blended_ibl /= total_weight;
```

## Example Usage

### Basic Setup

```rust
use praxis_ecs::{World, EnvironmentProbe, Transform};
use praxis_graphics::EnvironmentProbeManager;
use praxis_math::Vec3;

// Create probe manager
let mut probe_manager = EnvironmentProbeManager::new(
    device,
    allocator,
    command_buffer_allocator,
    queue,
)?;

// Spawn probe entity
world.spawn((
    Transform::from_xyz(0.0, 2.0, 0.0),
    EnvironmentProbe::new("main_probe")
        .with_resolution(512)
        .with_update_once(),
));

// In render loop:
probe_manager.update_probes();
let ibl_uniforms = probe_manager.get_ibl_uniforms();
// Upload ibl_uniforms to GPU and use in shaders
```

### Multiple Probes

```rust
// Spawn multiple probes for different areas
let probe_positions = [
    Vec3::new(-10.0, 2.0, 0.0),
    Vec3::new(0.0, 2.0, 0.0),
    Vec3::new(10.0, 2.0, 0.0),
];

for (i, pos) in probe_positions.iter().enumerate() {
    world.spawn((
        Transform::from_translation(*pos),
        EnvironmentProbe::new(format!("probe_{}", i))
            .with_resolution(256)
            .with_influence_radius(15.0)
            .with_update_every_n_frames(120),
    ));
}
```

### Dynamic Updates

```rust
// Manual update trigger
fn on_scene_change(world: &mut World) {
    let mut query = world.query::<&mut EnvironmentProbe>();
    for mut probe in query.iter_mut(world) {
        probe.mark_dirty();
    }
}
```

## Technical Details

### PBR Integration

Environment probes use the split-sum approximation for the specular integral:

```
L_specular = ∫ L(l) * f(l, v) * cos(θ) dl
           ≈ (∫ L(l) * cos(θ) dl) * (∫ f(l, v) * cos(θ) dl)
           = prefiltered_color * (F0 * brdf.x + brdf.y)
```

### GGX Distribution

Importance sampling uses the GGX (Trowbridge-Reitz) distribution:

```
D(h) = α² / (π * ((n·h)² * (α² - 1) + 1)²)

where α = roughness²
```

### Memory Layout

Per-probe memory usage:
- Environment map: resolution² × 6 × 4 channels × 2 bytes (FP16)
- Irradiance map: 32² × 6 × 4 × 2 = ~25 KB
- Prefiltered map: resolution² × 6 × 4 × 2 × 1.33 (mipmaps)
- BRDF LUT: 512² × 2 = ~512 KB (shared across all probes)

Example for 512x512 probe:
- Environment: 6 MB
- Irradiance: 25 KB
- Prefiltered: 8 MB
- **Total: ~14 MB per probe**

## Future Enhancements

Potential improvements for the environment probe system:

1. **Probe Interpolation**: Smooth blending between multiple probes
2. **Parallax Correction**: Box-projected cubemaps for indoor scenes
3. **Temporal Filtering**: Smooth updates over multiple frames
4. **Probe Arrays**: Volumetric light probes for better spatial resolution
5. **Compression**: Use BC6H compression for HDR cubemaps
6. **Streaming**: Load/unload probes based on camera position

## References

- [Real Shading in Unreal Engine 4](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf) - Epic Games
- [Image-Based Lighting](https://learnopengl.com/PBR/IBL/Diffuse-irradiance) - LearnOpenGL
- [Moving Frostbite to PBR](https://www.gdcvault.com/play/1023512/Physically-Based-Shading-in-Unity) - EA DICE
