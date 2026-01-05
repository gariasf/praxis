# Deferred Rendering

This document describes the deferred rendering system in Praxis Graphics, covering G-buffer layout, geometry and lighting pass architecture, and light accumulation techniques.

## Overview

Deferred rendering is a rendering technique that separates geometry processing from lighting calculations by storing surface properties in intermediate buffers (G-buffer). This enables efficient rendering of scenes with many lights.

### Traditional Forward Rendering

In forward rendering:
```
For each object:
    For each light:
        Calculate lighting contribution
```

**Cost**: O(objects × triangles × lights)

### Deferred Rendering

In deferred rendering:
```
Pass 1 (Geometry): For each object:
    Write surface properties to G-buffer

Pass 2 (Lighting): For each pixel:
    For each light:
        Calculate lighting from G-buffer data
```

**Cost**: O(objects × triangles) + O(pixels × lights)

## Architecture

### Two-Pass System

#### Pass 1: Geometry Pass
Renders scene geometry to multiple render targets (G-buffer), storing per-pixel surface properties.

#### Pass 2: Lighting Pass
Full-screen post-process pass that reads G-buffer and accumulates lighting contributions.

## G-Buffer Layout

The G-buffer in Praxis consists of four render targets:

### 1. Albedo Buffer
- **Format**: `R8G8B8A8_UNORM`
- **Size**: 4 bytes per pixel
- **Content**: Base color (RGB) + unused (A)
- **Range**: [0.0, 1.0] normalized values

```glsl
layout(location = 0) out vec4 o_albedo;
o_albedo = vec4(base_color, 1.0);
```

**Purpose**: Stores the surface's intrinsic color (diffuse color) without lighting.

**Notes**:
- Alpha channel is unused in the current implementation
- Could be used for material flags, transparency, or other per-pixel data
- 8-bit precision is sufficient for color storage

### 2. Normal Buffer
- **Format**: `R16G16B16A16_SFLOAT`
- **Size**: 8 bytes per pixel
- **Content**: World-space normal (RGB) + unused (A)
- **Range**: [-1.0, 1.0] floating-point values

```glsl
layout(location = 1) out vec4 o_normal;
o_normal = vec4(normalize(world_normal), 0.0);
```

**Purpose**: Stores the surface normal for lighting calculations.

**Notes**:
- World-space normals (not view-space) for flexibility
- 16-bit float provides sufficient precision for normal vectors
- Normals must be normalized before writing
- Alpha channel unused

### 3. Metallic-Roughness Buffer
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

**Purpose**: Stores PBR material properties for physically-based shading.

**Channel Breakdown**:
- **R (Metallic)**: 0.0 = dielectric (non-metal), 1.0 = metal
- **G (Roughness)**: 0.0 = smooth/glossy, 1.0 = rough/matte
- **B (Emissive)**: Emissive intensity for self-illuminated surfaces
- **A (Unused)**: Reserved for future use

### 4. Depth Buffer
- **Format**: `D32_SFLOAT`
- **Size**: 4 bytes per pixel
- **Content**: Standard depth values
- **Range**: [0.0, 1.0] normalized depth

```glsl
// Depth is written automatically by the GPU
gl_FragDepth = /* computed depth */;
```

**Purpose**: 
- Depth testing during geometry pass
- Position reconstruction in lighting pass
- Compatibility with other depth-based effects (SSAO, shadows)

**Notes**:
- 32-bit float provides high precision
- Can be sampled as a texture in the lighting pass
- Used for reconstructing world-space position from screen coordinates

## Memory Usage

### Per-Pixel Breakdown

For each pixel in the G-buffer:
- Albedo: 4 bytes
- Normal: 8 bytes
- Metallic-Roughness: 4 bytes
- Depth: 4 bytes
- **Total**: 20 bytes per pixel

### Resolution Examples

| Resolution | Total Memory | Albedo | Normal | Metal-Rough | Depth |
|------------|-------------|--------|--------|-------------|-------|
| 1280×720   | ~18.4 MB    | 3.7 MB | 7.4 MB | 3.7 MB      | 3.7 MB |
| 1920×1080  | ~41.0 MB    | 8.3 MB | 16.6 MB| 8.3 MB      | 8.3 MB |
| 2560×1440  | ~72.9 MB    | 14.7 MB| 29.5 MB| 14.7 MB     | 14.7 MB |
| 3840×2160  | ~164.0 MB   | 33 MB  | 66 MB  | 33 MB       | 33 MB |

## Geometry Pass

### Overview

The geometry pass renders all opaque scene geometry to the G-buffer, writing surface properties to multiple render targets simultaneously.

### Shader Interface

#### Vertex Shader
```glsl
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

#### Fragment Shader
```glsl
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

### Render Pass Setup

```rust
vulkano::ordered_passes_renderpass!(
    device.clone(),
    attachments: {
        albedo: {
            format: Format::R8G8B8A8_UNORM,
            samples: 1,
            load_op: Clear,
            store_op: Store,
        },
        normal: {
            format: Format::R16G16B16A16_SFLOAT,
            samples: 1,
            load_op: Clear,
            store_op: Store,
        },
        metallic_roughness: {
            format: Format::R8G8B8A8_UNORM,
            samples: 1,
            load_op: Clear,
            store_op: Store,
        },
        depth: {
            format: Format::D32_SFLOAT,
            samples: 1,
            load_op: Clear,
            store_op: Store,
        }
    },
    passes: [
        {
            color: [albedo, normal, metallic_roughness],
            depth_stencil: {depth},
            input: []
        }
    ]
)
```

## Lighting Pass

### Overview

The lighting pass is a full-screen post-process effect that reads the G-buffer textures and computes lighting for each visible pixel. This decouples lighting from geometry complexity.

### Full-Screen Quad

The lighting pass renders a full-screen quad covering the entire viewport:

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

### Shader Interface

#### Vertex Shader
```glsl
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 v_uv;

void main() {
    v_uv = uv;
    gl_Position = vec4(position, 0.0, 1.0);
}
```

#### Fragment Shader
```glsl
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

// SSAO texture (optional)
layout(set = 0, binding = 6) uniform sampler2D u_ssao;

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
    
    // Sample SSAO (1.0 = no occlusion, 0.0 = full occlusion)
    float ao = texture(u_ssao, v_uv).r;
    
    // Accumulate lighting
    vec3 final_color = vec3(0.0);
    
    // Ambient lighting with AO
    vec3 ambient = lighting.ambient_color.rgb * albedo * ao;
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

Reconstructing world-space position from depth is crucial for lighting calculations:

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

## Light Accumulation

### PBR Lighting Model

Praxis uses physically-based rendering (PBR) with the Cook-Torrance BRDF:

```
BRDF = k_d * f_lambert + k_s * f_cook_torrance

where:
  k_d = diffuse factor (1 - metallic)
  k_s = specular factor (Fresnel)
  f_lambert = albedo / π
  f_cook_torrance = (D * F * G) / (4 * (n·l) * (n·v))
```

### Directional Light Calculation

```glsl
vec3 calculate_directional_light(
    DirectionalLight light,
    vec3 normal,
    vec3 view_dir,
    vec3 albedo,
    float metallic,
    float roughness
) {
    vec3 light_dir = normalize(-light.direction);
    vec3 halfway = normalize(light_dir + view_dir);
    
    // Lambert diffuse
    float n_dot_l = max(dot(normal, light_dir), 0.0);
    
    // Cook-Torrance specular
    float D = distribution_ggx(normal, halfway, roughness);
    float G = geometry_smith(normal, view_dir, light_dir, roughness);
    vec3 F = fresnel_schlick(max(dot(halfway, view_dir), 0.0), albedo, metallic);
    
    // Combine diffuse and specular
    vec3 k_d = (vec3(1.0) - F) * (1.0 - metallic);
    vec3 diffuse = k_d * albedo / PI;
    
    vec3 numerator = D * G * F;
    float denominator = 4.0 * max(dot(normal, view_dir), 0.0) * n_dot_l + 0.0001;
    vec3 specular = numerator / denominator;
    
    vec3 radiance = light.color * light.intensity;
    return (diffuse + specular) * radiance * n_dot_l;
}
```

### Point Light Calculation

```glsl
vec3 calculate_point_light(
    PointLight light,
    vec3 world_pos,
    vec3 normal,
    vec3 view_dir,
    vec3 albedo,
    float metallic,
    float roughness
) {
    vec3 light_dir = normalize(light.position - world_pos);
    float distance = length(light.position - world_pos);
    
    // Attenuation
    float attenuation = 1.0 / (distance * distance);
    attenuation *= smoothstep(light.range, light.range * 0.5, distance);
    
    // Same PBR calculation as directional light
    vec3 halfway = normalize(light_dir + view_dir);
    float n_dot_l = max(dot(normal, light_dir), 0.0);
    
    float D = distribution_ggx(normal, halfway, roughness);
    float G = geometry_smith(normal, view_dir, light_dir, roughness);
    vec3 F = fresnel_schlick(max(dot(halfway, view_dir), 0.0), albedo, metallic);
    
    vec3 k_d = (vec3(1.0) - F) * (1.0 - metallic);
    vec3 diffuse = k_d * albedo / PI;
    
    vec3 numerator = D * G * F;
    float denominator = 4.0 * max(dot(normal, view_dir), 0.0) * n_dot_l + 0.0001;
    vec3 specular = numerator / denominator;
    
    vec3 radiance = light.color * light.intensity * attenuation;
    return (diffuse + specular) * radiance * n_dot_l;
}
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

## Benefits of Deferred Rendering

### 1. Many Lights Performance

**Forward rendering**: Cost increases with every light for every triangle drawn.
- Rendering 100 objects with 20 lights = 2000 lighting calculations per vertex/fragment

**Deferred rendering**: Cost is constant per light based on visible pixels.
- Rendering 100 objects with 20 lights = 100 geometry passes + 20 lighting passes over visible pixels

**When beneficial**: Scenes with 5+ dynamic lights show significant performance gains.

### 2. Efficient Light Culling

Since lighting is calculated per-pixel, occluded geometry contributes no cost:
- Forward: Must shade all triangles even if they're occluded later
- Deferred: Only shades visible pixels (Z-test already passed)

### 3. Decoupled Shading

Material properties and lighting are completely separate:
- Change lighting without re-rendering geometry
- Apply post-processing effects easily
- Add light types without modifying material shaders

### 4. Consistent Material System

All materials go through the same G-buffer:
- Unified PBR lighting model
- Easy to add new material properties (just add a G-buffer channel)
- Consistent shading across all objects

## Limitations and Trade-offs

### 1. Memory Bandwidth

**Problem**: Writing and reading 20 bytes per pixel multiple times

**Impact**:
- 1080p = ~41 MB G-buffer
- 4K = ~164 MB G-buffer
- Read 4 textures per pixel in lighting pass

**Mitigation**:
- Use appropriate formats (8-bit for color, 16-bit for normals)
- Consider resolution reduction for G-buffer
- Tile-based deferred rendering for mobile

### 2. Transparency

**Problem**: G-buffer stores only one value per pixel (no blending)

**Solutions**:
1. **Forward pass**: Render transparent objects in a separate forward pass after deferred lighting
2. **Weighted blended OIT**: Order-independent transparency techniques
3. **Depth peeling**: Multiple G-buffer layers (expensive)

**Common approach**: Hybrid deferred (opaque) + forward (transparent)

### 3. MSAA Compatibility

**Problem**: MSAA with multiple render targets is expensive
- Each G-buffer would need per-sample storage
- Memory usage multiplies by sample count
- Resolve operations for each buffer

**Solutions**:
1. **Post-process AA**: FXAA, SMAA, TAA instead of MSAA
2. **Forward pass for edges**: Deferred for most geometry, forward with MSAA for edges
3. **Compute-based MSAA**: Custom resolve in compute shaders

### 4. Material Variety

**Problem**: All materials must write to the same G-buffer layout

**Limitations**:
- Limited material properties (must fit in G-buffer)
- Cannot easily support exotic shading models
- Subsurface scattering requires special handling

**Solutions**:
1. **Material ID buffer**: Different lighting paths per material type
2. **Hybrid approach**: Special materials use forward rendering
3. **Packed data**: Clever packing of multiple properties in G-buffer channels

### 5. Bandwidth Concerns

**Reading 4 textures per pixel**:
```
1920×1080 × 60 fps × 20 bytes × 2 (read + write) = ~4.7 GB/s
```

**Mitigation**:
- Tile-based rendering (mobile GPUs)
- Lower G-buffer resolution for some targets
- Compute-based deferred with LDS (Local Data Share)

## Performance Optimization

### 1. Light Culling

Only process lights that affect the current pixel:

```glsl
// Simple radius check
float distance = length(light.position - world_pos);
if (distance > light.range) {
    continue;  // Skip this light
}
```

**Advanced**: Tile-based light culling (split screen into tiles, cull lights per-tile in compute shader).

### 2. Batching Lights

Group lights by type and process in batches:
- All directional lights in one loop
- All point lights in another loop
- Reduce branching and improve cache coherency

### 3. Quality Tiers

```rust
pub enum DeferredQuality {
    Low,      // 32-bit normals, simpler lighting
    Medium,   // 64-bit normals, standard lighting
    High,     // 64-bit normals, advanced effects
}
```

Adjust G-buffer precision and lighting complexity based on performance target.

### 4. Resolution Scaling

Render G-buffer at lower resolution:
- Geometry pass: 1440p (75% scale)
- Lighting pass: 1440p → upscale to 2160p
- Saves bandwidth and fillrate

### 5. Compute-Based Lighting

Use compute shaders for lighting pass:
- Better occupancy on modern GPUs
- Shared memory for G-buffer data
- Flexible work group sizes

## Integration with Other Systems

### SSAO Integration

Screen-space ambient occlusion fits naturally with deferred rendering:

```rust
// After geometry pass:
let ssao_texture = ssao_renderer.render(builder, &gbuffer, projection, view)?;

// In lighting pass:
// Multiply ambient lighting by SSAO factor
vec3 ambient = lighting.ambient_color.rgb * albedo * ssao_factor;
```

See `praxis_graphics::ssao` for details.

### Shadow Mapping

Shadows can be computed before or during the lighting pass:

```glsl
// In lighting pass, for each light:
float shadow_factor = calculate_shadow(world_pos, light);
vec3 light_contribution = /* ... */ * shadow_factor;
```

See `docs/shadow_mapping.md` for details.

### HDR and Tone Mapping

Deferred rendering output can be HDR:

```rust
// Lighting pass outputs to HDR framebuffer
let hdr_framebuffer = HdrRenderTarget::new(...)?;

// Apply tone mapping after lighting
tone_mapper.apply(builder, &hdr_framebuffer, swapchain_framebuffer, ...)?;
```

See `docs/advanced_rendering.md` for HDR details.

## Usage Example

```rust
use praxis_graphics::{DeferredRenderer, DrawCommand};
use praxis_utils::Result;

fn render_frame(
    deferred_renderer: &DeferredRenderer,
    builder: &mut AutoCommandBufferBuilder,
    output_framebuffer: Arc<Framebuffer>,
    viewport: Viewport,
    draw_commands: &[DrawCommand],
    view_proj_buffer: Subbuffer<ViewProjectionUniforms>,
    dynamic_uniform_buffer: &DynamicUniformBuffer,
    mesh_manager: &MeshAssetManager,
    texture_manager: &TextureManager,
    lighting_buffer: Subbuffer<LightingUniforms>,
) -> Result<()> {
    deferred_renderer.render(
        builder,
        output_framebuffer,
        viewport,
        draw_commands,
        view_proj_buffer,
        dynamic_uniform_buffer,
        mesh_manager,
        texture_manager,
        lighting_buffer,
    )?;
    
    Ok(())
}
```

### With SSAO

```rust
// After geometry pass, render SSAO
let ssao_texture = ssao_renderer.render(
    builder,
    &deferred_renderer.gbuffer.as_ref().unwrap(),
    projection_matrix,
    view_matrix,
)?;

// Render with SSAO applied to ambient lighting
deferred_renderer.render_with_ssao(
    builder,
    output_framebuffer,
    viewport,
    draw_commands,
    view_proj_buffer,
    dynamic_uniform_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
    ssao_texture,
)?;
```

## Debugging G-Buffer

Visualize individual G-buffer components:

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

## Further Reading

- **Real-Time Rendering, 4th Edition** - Chapter 20: Efficient Shading
- **GPU Gems 2** - Chapter 9: Deferred Shading in S.T.A.L.K.E.R.
- [Learn OpenGL - Deferred Shading](https://learnopengl.com/Advanced-Lighting/Deferred-Shading)
- [OurMachinery - High-Performance Deferred Shading](https://ourmachinery.com/post/high-performance-deferred-shading/)
- **[Praxis Documentation]** - `docs/advanced_rendering.md` for SSAO, HDR, and IBL
- **[Praxis Documentation]** - `docs/shadow_mapping.md` for shadow integration

## References

- `crates/praxis_graphics/src/deferred.rs` - Implementation
- `examples/deferred_demo.rs` - Complete example
- `crates/praxis_graphics/src/ssao.rs` - SSAO integration
