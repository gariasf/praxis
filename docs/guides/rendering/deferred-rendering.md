# Deferred Rendering

Deferred rendering separates geometry processing from lighting calculations by storing surface properties in intermediate buffers (G-buffer). This guide covers the complete deferred rendering system in Praxis.

## Overview

Deferred rendering uses a two-pass approach:

```text
Pass 1 (Geometry): For each object:
    Write surface properties to G-buffer

Pass 2 (Lighting): For each pixel:
    For each light:
        Calculate lighting from G-buffer data
```

**Complexity**: O(objects × triangles) + O(pixels × lights)

**Best for**: Scenes with many lights (5+).

## Why Deferred Rendering?

### Comparison with Forward Rendering

**Forward Rendering**:
```
For each object:
    For each light:
        Calculate lighting contribution
```
Cost: O(objects × triangles × lights)

**Deferred Rendering**:
```
Pass 1: Write surface data to G-buffer
Pass 2: Calculate lighting per visible pixel
```
Cost: O(objects × triangles) + O(pixels × lights)

### When to Use Deferred

| Scenario | Benefit |
|----------|---------|
| 5+ dynamic lights | Lighting cost scales with pixels, not triangles |
| Complex geometry | Geometry only processed once |
| Many occluded objects | Hidden objects don't compute lighting |
| Screen-space effects | G-buffer provides rich data (normals, depth, etc.) |

## G-Buffer Layout

The G-buffer stores per-pixel surface properties across multiple render targets:

### Attachment 0: Albedo Buffer

- **Format**: `R8G8B8A8_UNORM`
- **Size**: 4 bytes per pixel
- **Content**: Base color (RGB) + unused (A)
- **Range**: [0.0, 1.0] normalized values

```glsl
layout(location = 0) out vec4 o_albedo;
o_albedo = vec4(base_color, 1.0);
```

**Purpose**: Stores the surface's intrinsic color without lighting.

### Attachment 1: Normal Buffer

- **Format**: `R16G16B16A16_SFLOAT`
- **Size**: 8 bytes per pixel
- **Content**: World-space normal (RGB) + unused (A)
- **Range**: [-1.0, 1.0] floating-point values

```glsl
layout(location = 1) out vec4 o_normal;
o_normal = vec4(normalize(world_normal), 0.0);
```

**Purpose**: Stores surface normals for lighting calculations.

**Note**: World-space normals (not view-space) for flexibility with multiple cameras and effects.

### Attachment 2: Metallic-Roughness Buffer

- **Format**: `R8G8B8A8_UNORM`
- **Size**: 4 bytes per pixel
- **Content**: Metallic (R), Roughness (G), Emissive (B), unused (A)
- **Range**: [0.0, 1.0] normalized values

```glsl
layout(location = 2) out vec4 o_metallic_roughness;
o_metallic_roughness = vec4(
    material.metallic,
    material.roughness,
    material.emissive,
    0.0
);
```

**Channel Breakdown**:
- **R (Metallic)**: 0.0 = dielectric, 1.0 = metal
- **G (Roughness)**: 0.0 = smooth/glossy, 1.0 = rough/matte
- **B (Emissive)**: Emissive intensity for self-illuminated surfaces
- **A (Unused)**: Reserved for future material properties

### Attachment 3: Depth Buffer

- **Format**: `D32_SFLOAT`
- **Size**: 4 bytes per pixel
- **Content**: Standard depth values
- **Range**: [0.0, 1.0] normalized depth

**Purpose**: 
- Depth testing during geometry pass
- Position reconstruction in lighting pass
- Compatibility with depth-based effects (SSAO, shadows)

### Memory Usage

**Per-Pixel Breakdown**:
- Albedo: 4 bytes
- Normal: 8 bytes
- Metallic-Roughness: 4 bytes
- Depth: 4 bytes
- **Total**: 20 bytes per pixel

**Resolution Examples**:

| Resolution | Total Memory | Albedo | Normal | Metal-Rough | Depth |
|------------|-------------|--------|--------|-------------|-------|
| 1280×720   | ~18.4 MB    | 3.7 MB | 7.4 MB | 3.7 MB      | 3.7 MB |
| 1920×1080  | ~41.0 MB    | 8.3 MB | 16.6 MB| 8.3 MB      | 8.3 MB |
| 2560×1440  | ~72.9 MB    | 14.7 MB| 29.5 MB| 14.7 MB     | 14.7 MB |
| 3840×2160  | ~164.0 MB   | 33 MB  | 66 MB  | 33 MB       | 33 MB |

## Pass 1: Geometry Pass

Renders scene geometry to the G-buffer, writing surface properties to multiple render targets.

### Vertex Shader

```glsl
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 color;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_world_normal;
layout(location = 2) out vec2 v_uv;
layout(location = 3) out vec3 v_color;

layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
} view_proj;

layout(set = 0, binding = 1) uniform Transform {
    mat4 model;
} transform;

void main() {
    vec4 world_pos = transform.model * vec4(position, 1.0);
    v_world_pos = world_pos.xyz;
    v_world_normal = mat3(transform.model) * normal;
    v_uv = uv;
    v_color = color;
    
    gl_Position = view_proj.proj * view_proj.view * world_pos;
}
```

### Fragment Shader

```glsl
#version 450

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_world_normal;
layout(location = 2) in vec2 v_uv;
layout(location = 3) in vec3 v_color;

layout(location = 0) out vec4 o_albedo;
layout(location = 1) out vec4 o_normal;
layout(location = 2) out vec4 o_metallic_roughness;

layout(set = 0, binding = 2) uniform sampler2D u_texture;

layout(set = 1, binding = 0) uniform Material {
    float metallic;
    float roughness;
    float emissive;
} material;

void main() {
    // Sample texture and combine with vertex color
    vec4 tex_color = texture(u_texture, v_uv);
    vec3 base_color = tex_color.rgb * v_color;
    
    // Write to G-buffer
    o_albedo = vec4(base_color, 1.0);
    o_normal = vec4(normalize(v_world_normal), 0.0);
    o_metallic_roughness = vec4(
        material.metallic,
        material.roughness,
        material.emissive,
        0.0
    );
}
```

### Pipeline Configuration

```rust
use vulkano::pipeline::graphics::{
    depth_stencil::{DepthStencilState, DepthState, CompareOp},
    color_blend::{ColorBlendState, ColorBlendAttachmentState},
};

// Enable depth testing and writing
depth_stencil_state: Some(DepthStencilState {
    depth: Some(DepthState {
        compare_op: CompareOp::Less,
        write_enable: true,
    }),
    ..Default::default()
}),

// Three color attachments (albedo, normal, metallic-roughness)
color_blend_state: Some(ColorBlendState::with_attachment_states(
    3,  // Number of color attachments
    ColorBlendAttachmentState::default(),
)),
```

## Pass 2: Lighting Pass

Full-screen pass that reads G-buffer and accumulates lighting contributions.

### Full-Screen Quad

The lighting pass renders a full-screen quad:

```rust
let vertices = [
    // position      // uv
    [-1.0, -1.0],    [0.0, 0.0],  // Bottom-left
    [ 1.0, -1.0],    [1.0, 0.0],  // Bottom-right
    [ 1.0,  1.0],    [1.0, 1.0],  // Top-right
    [-1.0,  1.0],    [0.0, 1.0],  // Top-left
];

let indices = [0, 1, 2, 0, 2, 3];  // Two triangles
```

### Vertex Shader

```glsl
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 v_uv;

void main() {
    v_uv = uv;
    gl_Position = vec4(position, 0.0, 1.0);
}
```

### Fragment Shader

```glsl
#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 o_color;

// G-buffer textures
layout(set = 0, binding = 0) uniform sampler2D u_albedo;
layout(set = 0, binding = 1) uniform sampler2D u_normal;
layout(set = 0, binding = 2) uniform sampler2D u_metallic_roughness;
layout(set = 0, binding = 3) uniform sampler2D u_depth;

// Camera uniforms
layout(set = 0, binding = 4) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 view_pos;
} camera;

// Lighting uniforms
layout(set = 0, binding = 5) uniform LightingData {
    DirectionalLight directional_lights[8];
    PointLight point_lights[16];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
} lighting;

void main() {
    // Sample G-buffer
    vec3 albedo = texture(u_albedo, v_uv).rgb;
    vec3 normal = texture(u_normal, v_uv).rgb;
    vec4 material_data = texture(u_metallic_roughness, v_uv);
    float depth = texture(u_depth, v_uv).r;
    
    // Extract material properties
    float metallic = material_data.r;
    float roughness = material_data.g;
    float emissive = material_data.b;
    
    // Reconstruct world position from depth
    vec3 world_pos = reconstruct_position(v_uv, depth);
    
    // Calculate view direction
    vec3 view_dir = normalize(camera.view_pos - world_pos);
    
    // Accumulate lighting
    vec3 final_color = vec3(0.0);
    
    // Ambient lighting
    vec3 ambient = lighting.ambient_color.rgb * albedo;
    final_color += ambient;
    
    // Directional lights
    for (uint i = 0; i < lighting.directional_light_count; i++) {
        final_color += calculate_directional_light(
            lighting.directional_lights[i],
            normal,
            view_dir,
            albedo,
            metallic,
            roughness
        );
    }
    
    // Point lights
    for (uint i = 0; i < lighting.point_light_count; i++) {
        final_color += calculate_point_light(
            lighting.point_lights[i],
            world_pos,
            normal,
            view_dir,
            albedo,
            metallic,
            roughness
        );
    }
    
    // Add emissive contribution
    final_color += albedo * emissive;
    
    o_color = vec4(final_color, 1.0);
}
```

### Position Reconstruction

Reconstruct world-space position from depth to save G-buffer memory:

```glsl
vec3 reconstruct_position(vec2 uv, float depth) {
    // Convert UV and depth to NDC space
    vec4 ndc = vec4(
        uv.x * 2.0 - 1.0,  // [-1, 1]
        uv.y * 2.0 - 1.0,  // [-1, 1]
        depth * 2.0 - 1.0, // [-1, 1]
        1.0
    );
    
    // Transform to world space
    vec4 world_pos = inverse(camera.proj * camera.view) * ndc;
    world_pos /= world_pos.w;  // Perspective divide
    
    return world_pos.xyz;
}
```

**Alternative**: Store world position in G-buffer (expensive, requires additional 16-byte buffer).

### SSAO Integration (Optional)

Screen-space ambient occlusion can be integrated into the lighting pass:

```glsl
// SSAO texture (optional)
layout(set = 0, binding = 6) uniform sampler2D u_ssao;

void main() {
    // ... sample G-buffer ...
    
    // Sample SSAO (1.0 = no occlusion, 0.0 = full occlusion)
    float ao = texture(u_ssao, v_uv).r;
    
    // Apply to ambient lighting
    vec3 ambient = lighting.ambient_color.rgb * albedo * ao;
    
    // ... rest of lighting ...
}
```

## PBR Lighting Model

Praxis uses the Cook-Torrance BRDF (same as forward rendering):

```
BRDF = k_d * f_lambert + k_s * f_cook_torrance

where:
  k_d = diffuse factor (1 - metallic)
  k_s = specular factor (Fresnel)
  f_lambert = albedo / π
  f_cook_torrance = (D * F * G) / (4 * (n·l) * (n·v))
```

### PBR Functions

#### Distribution (GGX)

```glsl
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float n_dot_h = max(dot(N, H), 0.0);
    float n_dot_h2 = n_dot_h * n_dot_h;
    
    float numerator = a2;
    float denominator = (n_dot_h2 * (a2 - 1.0) + 1.0);
    denominator = PI * denominator * denominator;
    
    return numerator / denominator;
}
```

#### Geometry (Smith)

```glsl
float geometry_smith(vec3 N, vec3 V, vec3 L, float roughness) {
    float n_dot_v = max(dot(N, V), 0.0);
    float n_dot_l = max(dot(N, L), 0.0);
    float ggx_v = geometry_schlick_ggx(n_dot_v, roughness);
    float ggx_l = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx_v * ggx_l;
}

float geometry_schlick_ggx(float n_dot_v, float roughness) {
    float r = roughness + 1.0;
    float k = (r * r) / 8.0;
    return n_dot_v / (n_dot_v * (1.0 - k) + k);
}
```

#### Fresnel (Schlick)

```glsl
vec3 fresnel_schlick(float cos_theta, vec3 albedo, float metallic) {
    vec3 F0 = mix(vec3(0.04), albedo, metallic);
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}
```

See [forward-rendering.md](forward-rendering.md) for additional PBR implementation details.

## Usage Example

```rust
use praxis_graphics::{DeferredRenderer, DrawCommand};
use praxis_math::Mat4;

// Create deferred renderer
let deferred_renderer = DeferredRenderer::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
    1920,
    1080,
)?;

// Define draw commands
let draw_commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::IDENTITY,
        texture_name: Some("brick".to_string()),
        material_properties: Some(MaterialProperties::new()
            .with_metallic(0.0)
            .with_roughness(0.6)),
    },
];

// Render using deferred pipeline
deferred_renderer.render(
    &mut command_buffer_builder,
    output_framebuffer,
    viewport,
    &draw_commands,
    view_proj_buffer,
    &dynamic_uniform_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
)?;
```

## Benefits and Limitations

### Benefits

**1. Many Lights Performance**

Forward rendering cost increases with every light for every triangle:
- 100 objects with 20 lights = 2000 lighting calculations per fragment

Deferred rendering cost is constant per light based on visible pixels:
- 100 objects with 20 lights = 100 geometry passes + 20 lighting passes over visible pixels

**2. Efficient Light Culling**

Only shades visible pixels—occluded geometry contributes no cost.

**3. Decoupled Shading**

- Change lighting without re-rendering geometry
- Apply post-processing effects easily
- Add light types without modifying material shaders

**4. Consistent Material System**

All materials go through the same G-buffer—unified PBR lighting model.

### Limitations

**1. Memory Bandwidth**

Writing and reading 20 bytes per pixel multiple times:
- 1080p = ~41 MB G-buffer
- 4K = ~164 MB G-buffer

**Mitigation**: Use appropriate formats, consider resolution scaling.

**2. Transparency**

G-buffer stores only one value per pixel (no blending).

**Solutions**:
- **Forward pass**: Render transparent objects separately after deferred lighting
- **Hybrid approach**: Deferred for opaque, forward for transparent

**3. MSAA Compatibility**

MSAA with multiple render targets is expensive (memory multiplies by sample count).

**Solutions**:
- Use post-process AA (FXAA, SMAA, TAA)
- Forward pass with MSAA for edges only

**4. Material Variety**

All materials must write to the same G-buffer layout.

**Solutions**:
- **Material ID buffer**: Different lighting paths per material type
- **Hybrid approach**: Special materials use forward rendering

## Performance Optimization

### 1. Light Culling

Only process lights that affect the current pixel:

```glsl
float distance = length(light.position - world_pos);
if (distance > light.range) {
    continue;  // Skip this light
}
```

### 2. Quality Tiers

Adjust G-buffer precision based on performance target:

```rust
pub enum DeferredQuality {
    Low,      // Lower precision normals, simpler lighting
    Medium,   // Standard quality (default)
    High,     // Higher precision, advanced effects
}
```

### 3. Resolution Scaling

Render G-buffer at lower resolution:
- Geometry pass: 1440p (75% scale)
- Lighting pass: 1440p → upscale to 2160p
- Saves bandwidth and fillrate

### 4. Compute-Based Lighting

Use compute shaders for lighting:
- Better occupancy on modern GPUs
- Shared memory for G-buffer data
- Flexible work group sizes

### 5. Bandwidth Concerns

Deferred rendering has significant bandwidth requirements:

```
1920×1080 × 60 fps × 20 bytes × 2 (read + write) = ~4.7 GB/s
```

**Mitigation Strategies**:
- Tile-based rendering (mobile GPUs handle this automatically)
- Lower G-buffer resolution for some render targets
- Compute-based deferred with Local Data Share (LDS)
- Use lower precision formats where appropriate

## Integration with Other Systems

### SSAO (Screen-Space Ambient Occlusion)

SSAO fits naturally with deferred rendering:

```glsl
// Multiply ambient lighting by SSAO factor
vec3 ambient = lighting.ambient_color.rgb * albedo * ssao_factor;
```

### Shadows

Shadows can be computed in the lighting pass:

```glsl
float shadow_factor = calculate_shadow(world_pos, light);
vec3 light_contribution = /* ... */ * shadow_factor;
```

See [shadows.md](shadows.md) for details.

### HDR and Tone Mapping

Deferred rendering output can be HDR:

```rust
// Lighting pass outputs to HDR framebuffer
let hdr_framebuffer = HdrRenderTarget::new(...)?;

// Apply tone mapping after lighting
tone_mapper.apply(builder, &hdr_framebuffer, swapchain_framebuffer, ...)?;
```

See [hdr-tonemapping.md](hdr-tonemapping.md) for details.

## Debugging G-Buffer

Visualize individual G-buffer components for debugging:

```glsl
// Albedo visualization
o_color = vec4(albedo, 1.0);

// Normal visualization (remap from [-1,1] to [0,1])
o_color = vec4(normal * 0.5 + 0.5, 1.0);

// Metallic visualization
o_color = vec4(vec3(metallic), 1.0);

// Roughness visualization
o_color = vec4(vec3(roughness), 1.0);

// Depth visualization (non-linear)
float depth_vis = 1.0 - depth;
o_color = vec4(vec3(depth_vis), 1.0);

// Linearized depth visualization
float linear_depth = linearize_depth(depth);
o_color = vec4(vec3(linear_depth), 1.0);
```

## Hybrid Rendering

Combine deferred and forward for optimal results:

| Use Case | Renderer | Reason |
|----------|----------|--------|
| Opaque geometry | Deferred | Efficient with many lights |
| Transparent objects | Forward | Blending support |
| Skyboxes | Forward | No lighting needed |
| Particles | Forward | Alpha blending |

## Examples

```bash
# Deferred rendering demo
cargo run --example deferred_demo

# Advanced lighting
cargo run --example advanced_lighting_demo
```

## See Also

- [Forward Rendering](forward-rendering.md) - Alternative rendering approach
- [HDR and Tone Mapping](hdr-tonemapping.md) - High dynamic range
- [Shadows](shadows.md) - Shadow mapping
- [Post-Processing](post-processing.md) - Screen-space effects
- [Environment Probes](environment-probes.md) - Image-based lighting

## References

- **Real-Time Rendering, 4th Edition** - Chapter 20: Efficient Shading
- **GPU Gems 2** - Chapter 9: Deferred Shading in S.T.A.L.K.E.R.
- [GPU Gems 2 - Chapter 9: Deferred Shading](https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-9-deferred-shading-stalker)
- [Learn OpenGL - Deferred Shading](https://learnopengl.com/Advanced-Lighting/Deferred-Shading)
- [OurMachinery - High-Performance Deferred Shading](https://ourmachinery.com/post/high-performance-deferred-shading/)
