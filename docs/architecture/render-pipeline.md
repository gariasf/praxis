# Render Pipeline Architecture

This document provides a deep dive into Praxis's rendering architecture, comparing forward and deferred rendering approaches, and explaining when to use each technique. Understanding these rendering paths is essential for optimizing visual quality and performance.

## Overview

Praxis supports two primary rendering pipelines:

1. **Forward Rendering**: Traditional single-pass approach with immediate shading
2. **Deferred Rendering**: Multi-pass approach with G-buffer composition

Both pipelines support the same feature set (PBR materials, lighting, shadows), but differ in their execution strategy and performance characteristics.

## Forward Rendering

### Architecture

Forward rendering processes geometry in a single pass, computing lighting for each fragment during geometry rendering:

```
Scene Geometry → Vertex Shader → Fragment Shader → Framebuffer
                                     ↓
                               Lighting Calculation
```

### Pipeline Stages

#### 1. Geometry Processing (Vertex Shader)

```glsl
// vertex shader (simplified)
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;
layout(location = 4) in vec4 tangent;

layout(location = 0) out vec3 frag_world_pos;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_color;
layout(location = 3) out vec2 frag_uv;
layout(location = 4) out mat3 frag_tbn;

layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
} vp;

layout(set = 0, binding = 1) uniform Model {
    mat4 model;
} m;

void main() {
    // Transform to world space
    vec4 world_pos = m.model * vec4(position, 1.0);
    frag_world_pos = world_pos.xyz;
    
    // Transform to clip space
    gl_Position = vp.proj * vp.view * world_pos;
    
    // Normal transformation
    mat3 normal_matrix = transpose(inverse(mat3(m.model)));
    frag_normal = normalize(normal_matrix * normal);
    
    // Calculate TBN matrix for normal mapping
    vec3 T = normalize(normal_matrix * tangent.xyz);
    vec3 N = frag_normal;
    vec3 B = cross(N, T) * tangent.w;
    frag_tbn = mat3(T, B, N);
    
    // Pass through color and UV
    frag_color = color;
    frag_uv = uv;
}
```

**Key Operations**:
- Model → World → View → Clip space transformations
- Normal matrix computation for non-uniform scaling
- TBN matrix calculation for normal mapping
- UV coordinate pass-through for texture sampling

#### 2. Rasterization

Hardware rasterizer converts triangles to fragments:
- **Viewport Transform**: Clip space → Screen space
- **Primitive Assembly**: Connect vertices into triangles
- **Triangle Rasterization**: Generate fragments for pixels covered by triangle
- **Depth Testing**: Early Z-test to reject occluded fragments
- **Attribute Interpolation**: Interpolate vertex attributes across triangle

#### 3. Fragment Shading

```glsl
// fragment shader (simplified)
layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec3 frag_color;
layout(location = 3) in vec2 frag_uv;
layout(location = 4) in mat3 frag_tbn;

layout(location = 0) out vec4 out_color;

layout(set = 0, binding = 2) uniform sampler2D albedo_texture;
layout(set = 0, binding = 3) uniform Lighting {
    DirectionalLight directional_lights[4];
    PointLight point_lights[16];
    vec3 ambient_color;
    int num_directional_lights;
    int num_point_lights;
} lighting;

layout(set = 1, binding = 0) uniform Material {
    float metallic;
    float roughness;
    float ao;
    vec3 emissive;
} material;

void main() {
    // Sample textures
    vec4 albedo = texture(albedo_texture, frag_uv) * vec4(frag_color, 1.0);
    
    // PBR lighting calculation
    vec3 N = normalize(frag_normal);
    vec3 V = normalize(camera_position - frag_world_pos);
    
    vec3 total_light = lighting.ambient_color * albedo.rgb * material.ao;
    
    // Directional lights
    for (int i = 0; i < lighting.num_directional_lights; i++) {
        vec3 L = normalize(-lighting.directional_lights[i].direction);
        vec3 H = normalize(V + L);
        
        float NdotL = max(dot(N, L), 0.0);
        
        // Cook-Torrance BRDF
        vec3 F0 = mix(vec3(0.04), albedo.rgb, material.metallic);
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);
        float D = distributionGGX(N, H, material.roughness);
        float G = geometrySmith(N, V, L, material.roughness);
        
        vec3 specular = (D * F * G) / max(4.0 * max(dot(N, V), 0.0) * NdotL, 0.001);
        vec3 kD = (vec3(1.0) - F) * (1.0 - material.metallic);
        
        vec3 radiance = lighting.directional_lights[i].color * lighting.directional_lights[i].intensity;
        total_light += (kD * albedo.rgb / PI + specular) * radiance * NdotL;
    }
    
    // Point lights
    for (int i = 0; i < lighting.num_point_lights; i++) {
        vec3 L = lighting.point_lights[i].position - frag_world_pos;
        float distance = length(L);
        L = normalize(L);
        
        // Attenuation
        float attenuation = 1.0 / (distance * distance);
        attenuation *= max(0.0, 1.0 - (distance / lighting.point_lights[i].range));
        
        // Similar BRDF calculation as directional
        // ... (omitted for brevity)
        
        total_light += calculated_light * attenuation;
    }
    
    // Add emissive
    total_light += material.emissive;
    
    out_color = vec4(total_light, albedo.a);
}
```

**Key Features**:
- **PBR Shading**: Cook-Torrance microfacet BRDF
- **Multiple Light Types**: Directional and point lights
- **Material Properties**: Metallic-roughness workflow
- **Texture Sampling**: Albedo, normal, metallic-roughness maps

### Performance Characteristics

**Time Complexity**: O(lights × triangles_drawn)

For each visible triangle (after culling):
- Vertex shader: 3 invocations per triangle
- Fragment shader: N invocations (N = pixels covered)
- Each fragment processes ALL lights

**Example Calculation**:
```
Scene: 100,000 triangles, 10 lights, 1920×1080 resolution
Average overdraw: 2x (each pixel covered by 2 triangles on average)

Fragments processed: 1920 × 1080 × 2 = 4,147,200
Light calculations: 4,147,200 × 10 = 41,472,000 operations

If triangle count doubles: 82,944,000 operations (linear scaling with geometry)
If light count doubles: 82,944,000 operations (linear scaling with lights)
```

### Advantages

1. **Simple Pipeline**: Single render pass, straightforward implementation
2. **Transparency Support**: Natural handling of alpha blending
3. **MSAA Friendly**: Hardware MSAA works efficiently
4. **Low Memory Usage**: No intermediate G-buffer storage
5. **Good for Few Lights**: Efficient when light count is low (< 5-10)

### Disadvantages

1. **Poor Light Scaling**: Cost increases linearly with light count
2. **Overdraw Waste**: Occluded fragments still compute lighting
3. **Shader Complexity**: All lighting logic in one massive fragment shader
4. **No Lighting Culling**: All lights evaluated for all fragments

### Use Cases

Forward rendering is optimal for:

- **Few Lights**: Scenes with 1-5 lights (outdoor day scenes, simple indoor)
- **Transparency Heavy**: Games with lots of alpha-blended effects
- **Simple Lighting**: Non-PBR or simplified lighting models
- **Mobile/Low-End**: Devices with limited memory bandwidth
- **VR**: Low latency requirements favor simpler pipelines

**Example Scenarios**:
- Outdoor racing game with sun as primary light
- Stylized art style with simple shading
- UI rendering with minimal lighting
- Particle-heavy effects

## Deferred Rendering

### Architecture

Deferred rendering separates geometry and lighting into distinct passes:

```
Pass 1 (Geometry):
Scene Geometry → Vertex Shader → Fragment Shader → G-Buffer
                                     ↓
                               Material Properties

Pass 2 (Lighting):
G-Buffer → Full-Screen Quad → Fragment Shader → Framebuffer
                                  ↓
                           Lighting Calculation
```

### G-Buffer Layout

The G-buffer stores per-pixel geometry information:

```rust
pub struct GBuffer {
    // Attachment 0: Albedo (RGB) + unused (A)
    pub albedo: Arc<ImageView>,              // Format: R8G8B8A8_UNORM
    
    // Attachment 1: Normal (RGB) + unused (A)
    pub normal: Arc<ImageView>,              // Format: R16G16B16A16_SFLOAT
    
    // Attachment 2: Metallic (R), Roughness (G), Emissive (B), unused (A)
    pub metallic_roughness: Arc<ImageView>,  // Format: R8G8B8A8_UNORM
    
    // Depth attachment
    pub depth: Arc<ImageView>,               // Format: D32_SFLOAT
}
```

**Memory Requirements** (1920×1080):
```
Albedo:             1920 × 1080 × 4 bytes = 8.29 MB
Normal:             1920 × 1080 × 8 bytes = 16.59 MB (16-bit floats for precision)
Metallic-Roughness: 1920 × 1080 × 4 bytes = 8.29 MB
Depth:              1920 × 1080 × 4 bytes = 8.29 MB
Total:              41.47 MB per frame
```

### Pipeline Stages

#### Pass 1: Geometry Pass

Renders scene geometry to G-buffer:

```glsl
// geometry vertex shader
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

layout(location = 0) out vec3 frag_world_pos;
layout(location = 1) out vec3 frag_normal;
layout(location = 2) out vec3 frag_color;
layout(location = 3) out vec2 frag_uv;

void main() {
    // Similar to forward rendering vertex shader
    // Transform vertices, compute normals, pass data
}
```

```glsl
// geometry fragment shader
layout(location = 0) in vec3 frag_world_pos;
layout(location = 1) in vec3 frag_normal;
layout(location = 2) in vec3 frag_color;
layout(location = 3) in vec2 frag_uv;

layout(location = 0) out vec4 out_albedo;
layout(location = 1) out vec4 out_normal;
layout(location = 2) out vec4 out_metallic_roughness;

layout(set = 0, binding = 2) uniform sampler2D albedo_texture;
layout(set = 1, binding = 0) uniform Material {
    float metallic;
    float roughness;
    float ao;
    vec3 emissive;
} material;

void main() {
    // Sample and output to G-buffer
    vec4 albedo = texture(albedo_texture, frag_uv) * vec4(frag_color, 1.0);
    
    out_albedo = albedo;
    out_normal = vec4(normalize(frag_normal) * 0.5 + 0.5, 1.0);  // Encode [-1,1] → [0,1]
    out_metallic_roughness = vec4(material.metallic, material.roughness, material.emissive.r, 1.0);
}
```

**Operations**:
- Render all geometry once
- Write material properties to G-buffer
- No lighting calculations
- Depth testing for visibility

#### Pass 2: Lighting Pass

Full-screen pass that accumulates lighting:

```glsl
// lighting vertex shader (full-screen quad)
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;

layout(location = 0) out vec2 frag_uv;

void main() {
    frag_uv = uv;
    gl_Position = vec4(position, 0.0, 1.0);
}
```

```glsl
// lighting fragment shader
layout(location = 0) in vec2 frag_uv;
layout(location = 0) out vec4 out_color;

// G-buffer inputs
layout(set = 0, binding = 0) uniform sampler2D gbuffer_albedo;
layout(set = 0, binding = 1) uniform sampler2D gbuffer_normal;
layout(set = 0, binding = 2) uniform sampler2D gbuffer_metallic_roughness;
layout(set = 0, binding = 3) uniform sampler2D gbuffer_depth;

layout(set = 0, binding = 4) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
} vp;

layout(set = 0, binding = 5) uniform Lighting {
    DirectionalLight directional_lights[4];
    PointLight point_lights[16];
    vec3 ambient_color;
    int num_directional_lights;
    int num_point_lights;
} lighting;

void main() {
    // Sample G-buffer
    vec4 albedo = texture(gbuffer_albedo, frag_uv);
    vec4 normal_encoded = texture(gbuffer_normal, frag_uv);
    vec3 normal = normalize(normal_encoded.rgb * 2.0 - 1.0);  // Decode [0,1] → [-1,1]
    vec4 metallic_roughness = texture(gbuffer_metallic_roughness, frag_uv);
    float depth = texture(gbuffer_depth, frag_uv).r;
    
    // Reconstruct world position from depth
    vec4 clip_pos = vec4(frag_uv * 2.0 - 1.0, depth, 1.0);
    vec4 view_pos = inverse(vp.proj) * clip_pos;
    view_pos /= view_pos.w;
    vec4 world_pos = inverse(vp.view) * view_pos;
    
    // Extract material properties
    float metallic = metallic_roughness.r;
    float roughness = metallic_roughness.g;
    float emissive = metallic_roughness.b;
    
    // Accumulate lighting (same PBR calculation as forward rendering)
    vec3 N = normal;
    vec3 V = normalize(vp.camera_position - world_pos.xyz);
    
    vec3 total_light = lighting.ambient_color * albedo.rgb;
    
    // Process all lights...
    for (int i = 0; i < lighting.num_directional_lights; i++) {
        // Directional light calculation
    }
    
    for (int i = 0; i < lighting.num_point_lights; i++) {
        // Point light calculation
    }
    
    out_color = vec4(total_light + vec3(emissive), 1.0);
}
```

**Operations**:
- Single full-screen quad (2 triangles, 6 vertices)
- Sample G-buffer textures
- Reconstruct world position from depth
- Calculate lighting for ALL lights
- Output final color

### Performance Characteristics

**Time Complexity**: O(triangles + lights × pixels)

Geometry pass: O(triangles)
- Each triangle rendered once
- Writes to G-buffer

Lighting pass: O(lights × pixels)
- Full-screen pass (every pixel)
- Each pixel processes all lights

**Example Calculation**:
```
Scene: 100,000 triangles, 50 lights, 1920×1080 resolution
Average overdraw: 2x

Geometry Pass:
  Triangles processed: 100,000
  Fragments written: 1920 × 1080 = 2,073,600 (only visible pixels)

Lighting Pass:
  Fragments processed: 1920 × 1080 = 2,073,600
  Light calculations: 2,073,600 × 50 = 103,680,000

Total operations: ~104 million

If triangle count doubles: ~104 million (geometry pass impact minimal)
If light count doubles: ~207 million (linear scaling with visible pixels only)
```

**Comparison with Forward**:
```
Forward (100k triangles, 50 lights, 2x overdraw):
  Fragments: 1920 × 1080 × 2 = 4,147,200
  Light calculations: 4,147,200 × 50 = 207,360,000

Deferred (same scene):
  Geometry pass: 2,073,600 fragments
  Lighting pass: 2,073,600 × 50 = 103,680,000
  Total: ~104 million (50% reduction!)

Savings: ~103 million operations
```

### Advantages

1. **Excellent Light Scaling**: Cost is O(lights × visible_pixels), not O(lights × all_fragments)
2. **No Overdraw Waste**: Lighting only computed for visible pixels
3. **Decoupled Shading**: Geometry and lighting are independent
4. **Shader Simplicity**: Separate simple shaders vs. one complex shader
5. **Easy to Optimize**: Light culling per-tile, per-cluster, etc.

### Disadvantages

1. **High Memory Usage**: G-buffer requires significant VRAM (40-80 MB at 1080p)
2. **High Bandwidth**: Multiple render target writes and reads
3. **Transparency Issues**: Cannot handle alpha blending naturally
4. **MSAA Expensive**: MSAA on G-buffer multiplies memory and bandwidth
5. **Limited Material Variety**: G-buffer format constrains material properties

### Use Cases

Deferred rendering is optimal for:

- **Many Lights**: Scenes with 10+ lights (complex indoor, night scenes)
- **Complex Lighting**: Advanced lighting techniques (SSAO, light volumes)
- **High Triangle Count**: Dense geometry with significant overdraw
- **Opaque Geometry**: Primarily solid objects with minimal transparency
- **PC/Console**: Platforms with ample VRAM and bandwidth

**Example Scenarios**:
- Indoor scene with many light sources
- Night city with hundreds of street lights
- Complex architectural visualization
- Games with destructible environments (many lights from explosions/debris)

## Hybrid Approaches

Many modern engines combine both techniques:

### Deferred + Forward for Transparency

```rust
// Render opaque geometry with deferred
deferred_renderer.render_opaque(cmd_buffer, opaque_objects);

// Render transparent geometry with forward on top
forward_renderer.render_transparent(cmd_buffer, transparent_objects);
```

**Benefits**:
- Efficient many-light rendering for opaque geometry
- Proper alpha blending for transparent effects
- Best of both worlds

### Forward+ (Tiled Forward Rendering)

Alternative to deferred that keeps forward's benefits while improving light scaling:

1. **Depth Pre-pass**: Render depth only
2. **Light Culling**: Divide screen into tiles, cull lights per-tile
3. **Forward Rendering**: Render with per-tile light lists

**Advantages over Deferred**:
- Supports MSAA efficiently
- Handles transparency naturally
- Lower memory usage (no G-buffer)
- Material variety unlimited

**Advantages over Forward**:
- Much better light scaling
- Reduced lighting calculations

## Choosing the Right Pipeline

### Decision Matrix

| Criteria | Forward | Deferred | Forward+ |
|----------|---------|----------|----------|
| **Light Count** | < 10 | > 10 | > 10 |
| **Transparency** | Excellent | Poor | Excellent |
| **MSAA** | Good | Poor | Good |
| **Memory** | Low | High | Medium |
| **Bandwidth** | Medium | High | Medium |
| **Complexity** | Low | Medium | High |

### Performance Guidelines

**Forward is faster when**:
```
lights × triangles_drawn < pixels × lights

Example:
5 lights × 200,000 triangles with 30% visibility = 300,000 light calculations
1,920,000 pixels × 5 lights = 9,600,000 light calculations

Forward wins! (300k < 9.6M)
```

**Deferred is faster when**:
```
lights × pixels < lights × triangles_drawn

Example:
50 lights × 2,073,600 pixels = 103,680,000 light calculations
50 lights × 4,000,000 fragments (2x overdraw) = 200,000,000 light calculations

Deferred wins! (103M < 200M)
```

### Platform Considerations

**Desktop/Console (High-end)**:
- Deferred: Ample VRAM, high bandwidth
- Can afford G-buffer overhead
- Complex scenes with many lights

**Desktop/Console (Mid-range)**:
- Forward+ or Hybrid: Balance memory and performance
- Selective deferred for complex areas

**Mobile**:
- Forward: Limited bandwidth, prefer tile-based rendering
- Simpler scenes with fewer lights
- MSAA support important

**VR**:
- Forward+ preferred: Low latency critical
- Multi-view rendering support
- MSAA for edge quality

## Implementation Details

### Forward Rendering in Praxis

```rust
// From praxis_graphics/src/lib.rs
pub fn render(&mut self, cmds: &RenderCommands) -> Result<()> {
    // ... setup ...
    
    // Sort draw commands by material for batching
    indexed_commands.sort_by(|(_, a), (_, b)| {
        tex_a.cmp(tex_b).then_with(|| props_a.cmp(props_b))
    });
    
    // Update buffers
    self.dynamic_uniform_buffer.write_models(&model_matrices)?;
    
    // Record commands
    for (transform_set, material_set, mesh, object_index) in draw_list {
        // Bind mesh buffers
        builder.bind_vertex_buffers(0, mesh.vertex_buffer.clone())?
               .bind_index_buffer(mesh.index_buffer.clone())?;
        
        // Bind descriptor sets with dynamic offset
        builder.bind_descriptor_sets_unchecked(
            PipelineBindPoint::Graphics,
            pipeline.layout(),
            0,
            DescriptorSetWithOffsets::new(transform_set, [dynamic_offset])
        );
        
        // Only bind material if changed (optimization)
        if material_changed {
            builder.bind_descriptor_sets(..., material_set)?;
        }
        
        // Draw
        builder.draw_indexed(mesh.index_count, 1, 0, 0, 0)?;
    }
    
    // ... present ...
}
```

**Optimizations**:
- Material batching to reduce state changes
- Descriptor set reuse for same materials
- Dynamic uniform buffer for per-object transforms

### Deferred Rendering in Praxis

```rust
// From praxis_graphics/src/deferred.rs
pub fn render(&self, builder: &mut AutoCommandBufferBuilder, ...) -> Result<()> {
    // Pass 1: Geometry
    self.geometry_pass_render(builder, gbuffer, viewport, draw_commands, ...)?;
    
    // Pass 2: Lighting
    self.lighting_pass_render(builder, output_framebuffer, viewport, gbuffer, ...)?;
    
    Ok(())
}

fn geometry_pass_render(&self, ...) -> Result<()> {
    builder.begin_render_pass(
        RenderPassBeginInfo::framebuffer(gbuffer.framebuffer.clone()),
        ...
    )?;
    
    builder.bind_pipeline_graphics(self.geometry_pipeline.clone())?;
    
    for draw_cmd in draw_commands {
        // Render to G-buffer
        builder.draw_indexed(mesh.index_count, 1, 0, 0, 0)?;
    }
    
    builder.end_render_pass(...)?;
    Ok(())
}

fn lighting_pass_render(&self, ...) -> Result<()> {
    builder.begin_render_pass(
        RenderPassBeginInfo::framebuffer(output_framebuffer),
        ...
    )?;
    
    builder.bind_pipeline_graphics(self.lighting_pipeline.clone())?;
    
    // Bind G-buffer textures
    let descriptor_set = DescriptorSet::new(
        allocator,
        layout,
        [
            WriteDescriptorSet::image_view_sampler(0, gbuffer.albedo, sampler),
            WriteDescriptorSet::image_view_sampler(1, gbuffer.normal, sampler),
            WriteDescriptorSet::image_view_sampler(2, gbuffer.metallic_roughness, sampler),
            WriteDescriptorSet::image_view_sampler(3, gbuffer.depth, sampler),
            WriteDescriptorSet::buffer(4, view_proj_buffer),
            WriteDescriptorSet::buffer(5, lighting_buffer),
        ],
        [],
    )?;
    
    builder.bind_descriptor_sets(..., descriptor_set)?;
    
    // Draw full-screen quad
    builder.draw_indexed(6, 1, 0, 0, 0)?;
    
    builder.end_render_pass(...)?;
    Ok(())
}
```

## Advanced Techniques

### Tiled Deferred Rendering

Further optimization of deferred rendering:

1. Divide screen into tiles (e.g., 16×16 pixels)
2. Compute per-tile light lists using depth bounds
3. Process each tile with only affecting lights

**Benefits**:
- Dramatically reduces light calculations
- Better cache coherence
- Scales to hundreds/thousands of lights

### Clustered Rendering

3D extension of tiled rendering:

1. Divide view frustum into 3D clusters
2. Assign lights to clusters based on bounding volumes
3. Each fragment looks up its cluster's light list

**Benefits**:
- Even better culling than 2D tiling
- Handles large depth ranges efficiently
- State-of-the-art for many-light scenarios

## Summary

**Choose Forward Rendering when**:
- Few lights (< 10)
- Heavy transparency usage
- MSAA required
- Mobile/low-end platforms
- Simple lighting requirements

**Choose Deferred Rendering when**:
- Many lights (> 10)
- Mostly opaque geometry
- High-end platforms with VRAM
- Complex post-processing pipeline
- Advanced lighting effects (SSAO, volumetrics)

**Choose Hybrid Approach when**:
- Mix of opaque and transparent geometry
- Variable light counts across scenes
- Maximum flexibility required

Both pipelines in Praxis support the full PBR material system, shadows, and post-processing. The choice depends on your specific game's needs, target platform, and scene complexity. Profile both approaches with representative content to make an informed decision.
