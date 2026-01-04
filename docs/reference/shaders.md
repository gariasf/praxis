# Shaders Reference

Shader bindings and conventions in Praxis.

## Descriptor Set Layout

Praxis uses a consistent descriptor set organization:

### Set 0: Per-Frame / Global
Bound once per frame, shared across all objects.

| Binding | Type | Content |
|---------|------|---------|
| 0 | Uniform Buffer | View/Projection matrices |
| 2 | Uniform Buffer | Lighting data |
| 4-8 | (Shadow maps) | Shadow cascade textures |

### Set 1: Per-Material
Bound when material changes.

| Binding | Type | Content |
|---------|------|---------|
| 0 | Uniform Buffer | Material properties |
| 1 | Combined Sampler | Albedo texture |
| 2 | Combined Sampler | Normal map (optional) |

### Set 2: Per-Object
Bound for each draw call.

| Binding | Type | Content |
|---------|------|---------|
| 0 | Uniform Buffer | Model matrix |

## Uniform Structures

### ViewProjection
```glsl
layout(set = 0, binding = 0) uniform ViewProjection {
    mat4 view;
    mat4 proj;
    vec3 camera_pos;
};
```

### LightingUniforms
```glsl
struct DirectionalLight {
    vec4 direction;  // xyz = dir, w = padding
    vec4 color;      // rgb = color, a = unused
    float intensity;
    float _pad[3];
};

struct PointLight {
    vec4 position;   // xyz = pos, w = padding
    vec4 color;      // rgb = color, a = unused
    float intensity;
    float attenuation;
    float _pad[2];
};

layout(set = 0, binding = 2) uniform LightingUniforms {
    DirectionalLight directional_lights[8];
    PointLight point_lights[32];
    uint num_directional;
    uint num_point;
};
```

### MaterialProperties
```glsl
layout(set = 1, binding = 0) uniform MaterialProperties {
    vec4 albedo;
    float metallic;
    float roughness;
    float emissive;
    float _padding;
};
```

### ModelMatrix
```glsl
layout(set = 2, binding = 0) uniform Model {
    mat4 model;
};
```

## Vertex Formats

### Standard Vertex
```glsl
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 normal;
```

### Skinned Vertex (Animation)
```glsl
layout(location = 0) in vec3 position;
layout(location = 1) in vec4 color;
layout(location = 2) in vec2 uv;
layout(location = 3) in vec3 normal;
layout(location = 4) in uvec4 joint_indices;
layout(location = 5) in vec4 joint_weights;
```

## Shader Compilation

Shaders are compiled at build time via `vulkano-shaders`:

```rust
mod vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/main.vert",
    }
}

mod fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/main.frag",
    }
}
```

## Shader Locations

```
crates/praxis_graphics/src/shaders/
├── main.vert           # Standard vertex shader
├── main.frag           # PBR fragment shader
├── shadow.vert         # Shadow pass vertex
├── shadow.frag         # Shadow pass fragment
├── deferred_geom.vert  # Deferred geometry pass
├── deferred_geom.frag  # G-buffer output
├── deferred_light.vert # Full-screen quad
├── deferred_light.frag # Lighting accumulation
├── tonemap.vert        # Post-process vertex
├── tonemap.frag        # Tone mapping
└── README.md           # Shader documentation
```

## Common Patterns

### Transform to World Space
```glsl
vec4 world_pos = model * vec4(position, 1.0);
vec3 world_normal = mat3(model) * normal;
```

### Transform to Clip Space
```glsl
gl_Position = proj * view * model * vec4(position, 1.0);
```

### Sample Texture
```glsl
vec4 color = texture(albedo_tex, uv);
```

### PBR Lighting
```glsl
// Diffuse (Lambert)
float NdotL = max(dot(normal, light_dir), 0.0);
vec3 diffuse = albedo * NdotL;

// Specular (Blinn-Phong or GGX)
vec3 H = normalize(light_dir + view_dir);
float NdotH = max(dot(normal, H), 0.0);
vec3 specular = /* ... */;
```

## See Also

- [Rendering Guide](../guides/rendering.md)
- [Vulkan Concepts](../concepts/vulkan-rendering.md)
- [crates/praxis_graphics/src/shaders/README.md](../../crates/praxis_graphics/src/shaders/README.md)
