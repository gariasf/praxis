# Descriptor Sets Reference

Quick reference for descriptor set layouts used throughout Praxis graphics shaders.

## Standard Layout Convention

**Set 0**: Per-frame/per-draw resources  
**Set 1**: Per-material properties  
**Set 2**: Bindless rendering (optional)

## Set 0: Per-Frame/Per-Draw Resources

```glsl
// Binding 0: Camera and view data
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
} view_proj;

// Binding 1: Model matrix (DYNAMIC - use dynamic offset)
layout(set = 0, binding = 1, std140) uniform Model {
    mat4 model;
} model_ubo;

// Binding 2: Albedo texture
layout(set = 0, binding = 2) uniform sampler2D albedo_texture;

// Binding 3: Lighting data
layout(set = 0, binding = 3, std140) uniform LightingData {
    DirectionalLight directional_lights[8];
    PointLight point_lights[16];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
} lighting;

// Binding 4: Shadow mapping data
layout(set = 0, binding = 4, std140) uniform ShadowData {
    mat4 light_space_matrices[4];
    vec4 cascade_distances;
    uint cascade_count;
    uint shadow_map_size;
    uint pcf_samples;
    float bias;
} shadow;

// Bindings 5-8: Shadow cascade samplers
layout(set = 0, binding = 5) uniform sampler2DShadow shadow_map_0;
layout(set = 0, binding = 6) uniform sampler2DShadow shadow_map_1;
layout(set = 0, binding = 7) uniform sampler2DShadow shadow_map_2;
layout(set = 0, binding = 8) uniform sampler2DShadow shadow_map_3;

// Binding 9: Normal map
layout(set = 0, binding = 9) uniform sampler2D normal_map;

// Binding 10: Skeletal animation bones
layout(set = 0, binding = 10, std140) uniform BoneMatrices {
    mat4 bone_matrices[256];
} bone_matrices_ubo;
```

## Set 1: Per-Material Properties

```glsl
layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;         // RGBA color tint
    float metallic;          // 0.0 = dielectric, 1.0 = metal
    float roughness;         // 0.0 = smooth, 1.0 = rough
    float emissive_strength; // Self-illumination intensity
} material;
```

## Set 2: Bindless Rendering (Optional)

```glsl
// Binding 0: Texture array (up to 4096 textures)
layout(set = 2, binding = 0) uniform sampler2D bindless_textures[];

// Binding 1: Material data buffer
layout(set = 2, binding = 1, std140) uniform BindlessMaterialData {
    BindlessMaterial materials[4096];
} bindless_materials;

// Push constants for material index
layout(push_constant) uniform PushConstants {
    uint material_index;
} push;
```

## Shader Examples

### Standard Forward Rendering

**Vertex Shader:**
```glsl
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 v_world_pos;
layout(location = 1) out vec3 v_normal;
layout(location = 2) out vec2 v_uv;

layout(set = 0, binding = 0) uniform ViewProjection { ... } view_proj;
layout(set = 0, binding = 1) uniform Model { ... } model_ubo;

void main() {
    vec4 world_pos = model_ubo.model * vec4(position, 1.0);
    gl_Position = view_proj.proj * view_proj.view * world_pos;
    v_world_pos = world_pos.xyz;
    v_normal = mat3(model_ubo.model) * normal;
    v_uv = uv;
}
```

**Fragment Shader:**
```glsl
#version 450

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_normal;
layout(location = 2) in vec2 v_uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform ViewProjection { ... } view_proj;
layout(set = 0, binding = 2) uniform sampler2D albedo_texture;
layout(set = 0, binding = 3) uniform LightingData { ... } lighting;
layout(set = 1, binding = 0) uniform MaterialProperties { ... } material;

void main() {
    vec4 tex_color = texture(albedo_texture, v_uv);
    vec3 albedo = tex_color.rgb * material.base_color.rgb;
    // Calculate lighting...
    f_color = vec4(final_color, tex_color.a);
}
```

### Post-Process Shader

```glsl
#version 450

// Vertex
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 0) out vec2 v_uv;
void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_uv = uv;
}

// Fragment
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 0) uniform sampler2D input_texture;
void main() {
    f_color = texture(input_texture, v_uv);
}
```

### Compute Shader

```glsl
#version 450
layout(local_size_x = 256) in;

layout(set = 0, binding = 0, std430) readonly buffer Input {
    vec4 data[];
} input_buf;

layout(set = 0, binding = 1, std430) writeonly buffer Output {
    vec4 results[];
} output_buf;

layout(set = 0, binding = 2, std140) uniform Params {
    uint count;
    float multiplier;
} params;

void main() {
    uint id = gl_GlobalInvocationID.x;
    if (id >= params.count) return;
    output_buf.results[id] = input_buf.data[id] * params.multiplier;
}
```

## Special Cases

### Shadow Rendering
Uses a simplified Set 0 layout optimized for depth-only rendering:
- Binding 0: Model matrix
- Binding 1: Light-space matrix
- Binding 10: Bone matrices (if animated)

### Deferred Rendering
- **G-buffer pass**: Standard Set 0 layout
- **Lighting pass**: Repurposes Set 0 bindings for G-buffer textures

### Particles
Uses Set 1 for particle-specific textures instead of material properties:
- Binding 0: Particle texture
- Binding 1: Depth texture (for soft particles)

## Memory Layouts

- **std140**: Uniform buffers (16-byte alignment)
- **std430**: Storage buffers (tighter packing, compute only)

## Dynamic Offsets

Set 0, Binding 1 (Model matrix) uses dynamic offsets for per-object transforms:

```rust
command_buffer.bind_descriptor_sets(
    PipelineBindPoint::Graphics,
    pipeline.layout().clone(),
    0,
    descriptor_set,
    [object_index * 64], // 64 bytes per mat4
);
```

## Bindless Rendering

When using bindless mode, check material index in shader:

```glsl
bool use_bindless = (push.material_index != 0xFFFFFFFF);
if (use_bindless) {
    BindlessMaterial mat = bindless_materials.materials[push.material_index];
    tex_color = texture(bindless_textures[nonuniformEXT(mat.albedo_texture_index)], v_uv);
} else {
    tex_color = texture(albedo_texture, v_uv);
}
```

## See Also

- [Bindless Rendering](BINDLESS_RENDERING.md)
- [Material System](MATERIAL_SYSTEM.md)
- [Shader Source](src/shaders/)
