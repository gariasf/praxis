# Descriptor Set Quick Reference

This is a quick reference guide for developers working with shaders and pipelines in Praxis.

## Standard Layout at a Glance

### Set 0: Per-Frame/Per-Draw Resources

```glsl
// Binding 0: Camera and view data
layout(set = 0, binding = 0, std140) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_position;
    float _padding;
} view_proj;

// Binding 1: Model matrix (DYNAMIC - use dynamic offset for per-object)
layout(set = 0, binding = 1, std140) uniform Model {
    mat4 model;
} model_ubo;

// Binding 2: Albedo/diffuse texture
layout(set = 0, binding = 2) uniform sampler2D albedo_texture;

// Binding 3: Lighting data (directional + point lights)
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

// Binding 9: Normal map texture
layout(set = 0, binding = 9) uniform sampler2D normal_map;

// Binding 10: Skeletal animation bone matrices
layout(set = 0, binding = 10, std140) uniform BoneMatrices {
    mat4 bone_matrices[256];
} bone_matrices_ubo;
```

### Set 1: Per-Material Properties

```glsl
layout(set = 1, binding = 0, std140) uniform MaterialProperties {
    vec4 base_color;         // RGBA color tint
    float metallic;          // 0.0 = dielectric, 1.0 = metal
    float roughness;         // 0.0 = smooth, 1.0 = rough
    float emissive_strength; // Self-illumination intensity
    float _padding;          // std140 alignment
} material;
```

### Set 2: Bindless Rendering (Optional)

```glsl
// Binding 0: Texture array (up to 4096 textures)
layout(set = 2, binding = 0) uniform sampler2D bindless_textures[];

// Binding 1: Material data buffer
layout(set = 2, binding = 1, std140) uniform BindlessMaterialData {
    BindlessMaterial materials[4096];
} bindless_materials;

// Push constants for material index
layout(push_constant) uniform PushConstants {
    uint material_index;  // Index into bindless arrays
} push;
```

## Common Patterns

### Standard Forward Rendering Shader

```glsl
#version 450

// Vertex shader
layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec3 color;
layout(location = 3) in vec2 uv;

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

```glsl
// Fragment shader
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

### Post-Process Shader (Full-Screen Quad)

```glsl
#version 450

// Vertex shader
layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 0) out vec2 v_uv;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    v_uv = uv;
}

// Fragment shader
layout(location = 0) in vec2 v_uv;
layout(location = 0) out vec4 f_color;
layout(set = 0, binding = 0) uniform sampler2D input_texture;

void main() {
    vec3 color = texture(input_texture, v_uv).rgb;
    // Apply effect...
    f_color = vec4(color, 1.0);
}
```

### Compute Shader

```glsl
#version 450
layout(local_size_x = 256, local_size_y = 1, local_size_z = 1) in;

// Set 0: Input/output buffers
layout(set = 0, binding = 0, std430) buffer InputBuffer {
    vec4 data[];
} input_buf;

layout(set = 0, binding = 1, std430) buffer OutputBuffer {
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

## Pipeline Creation in Rust

### Basic Pipeline

```rust
use praxis_graphics::pipeline::create_simple_pipeline_3d;

let pipeline = create_simple_pipeline_3d(
    &device,
    &render_pass,
    [width, height],
)?;
```

### Custom Pipeline

```rust
use praxis_graphics::pipeline::{create_graphics_pipeline, PipelineConfig};
use praxis_graphics::vertex::Vertex3D;

let config = PipelineConfig {
    primitive_topology: PrimitiveTopology::TriangleList,
    cull_mode: CullMode::Back,
    front_face: FrontFace::CounterClockwise,
};

let pipeline = create_graphics_pipeline::<Vertex3D>(
    &device,
    &render_pass,
    [width, height],
    config,
)?;
```

## Important Notes

### Memory Layouts

- **std140**: Uniform buffer layout (alignment: vec4 = 16 bytes)
- **std430**: Storage buffer layout (tighter packing, compute shaders only)

### Dynamic Offsets

Set 0, Binding 1 (Model matrix) uses dynamic offsets:

```rust
// Rust side
command_buffer.bind_descriptor_sets(
    PipelineBindPoint::Graphics,
    pipeline.layout().clone(),
    0, // Set 0
    descriptor_set,
    [object_index * 64], // Dynamic offset (64 bytes per model matrix)
);
```

### Bindless Rendering

To use bindless mode:

1. Register textures: `bindless.register_texture(name, view, sampler)?`
2. Register materials: `bindless.register_material(material_data)?`
3. Push material index: `cmd.push_constants(layout, 0, material_index)`

Check `push.material_index != 0xFFFFFFFF` in shader to enable bindless path.

## Troubleshooting

### Common Errors

1. **Descriptor set validation failure**
   - Check binding numbers match between shader and Rust code
   - Verify descriptor types (UniformBuffer vs StorageBuffer)
   - Ensure dynamic offsets are used correctly

2. **Shader compilation errors**
   - Verify layout qualifiers (set, binding, std140/std430)
   - Check structure padding matches between GLSL and Rust
   - Ensure extension enables (e.g., `GL_EXT_nonuniform_qualifier`)

3. **Rendering artifacts**
   - Verify memory barriers between passes
   - Check depth testing configuration
   - Ensure proper synchronization with `cleanup_finished()`

## See Also

- [`DESCRIPTOR_SET_AUDIT.md`](DESCRIPTOR_SET_AUDIT.md) - Comprehensive audit of all shaders
- [`src/shaders/README.md`](src/shaders/README.md) - Shader documentation and conventions
- [`src/pipeline.rs`](src/pipeline.rs) - Pipeline creation implementation
- [`src/bindless.rs`](src/bindless.rs) - Bindless rendering system
