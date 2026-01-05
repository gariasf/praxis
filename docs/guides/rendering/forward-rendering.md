# Forward Rendering

Forward rendering is the traditional rendering approach where each object is processed through the full lighting pipeline. This guide covers forward rendering implementation, usage, and best practices in Praxis.

## Overview

In forward rendering, lighting calculations are performed for each fragment of each triangle during the geometry pass:

```text
For each object:
    For each light:
        Calculate lighting contribution
```

**Complexity**: O(objects × triangles × lights)

**Best for**: Scenes with few lights, transparency, MSAA requirements.

## Architecture

Forward rendering in Praxis uses a single-pass approach:

1. **Vertex Shader**: Transform vertices to clip space
2. **Fragment Shader**: Calculate lighting for each fragment
3. **Output**: Final lit color directly to framebuffer

### Advantages

- **Simple Pipeline**: Single render pass, no intermediate buffers
- **Transparency Support**: Native alpha blending support
- **MSAA Compatible**: Hardware MSAA works naturally
- **Low Memory**: No G-buffer overhead
- **Flexible Materials**: Can use complex, varied material shaders

### Trade-offs

- **Light Scalability**: Performance degrades with many lights
- **Overdraw Cost**: Hidden fragments still compute lighting
- **Shader Complexity**: All lighting must be in fragment shader
- **Limited to ~8-16 lights**: Practical light count constraint

## Material System

Praxis uses a PBR (Physically-Based Rendering) metallic-roughness workflow:

### Material Properties

```rust
use praxis_graphics::MaterialProperties;

let material = MaterialProperties {
    albedo: [1.0, 0.84, 0.0, 1.0],  // Gold color (RGBA)
    metallic: 1.0,                   // Full metal (0=dielectric, 1=metal)
    roughness: 0.1,                  // Polished (0=smooth, 1=rough)
    emissive: 0.0,                   // No self-emission
    _padding: 0.0,
};
```

**Albedo (Base Color)**:
- RGB: Surface color in linear space
- For metals: This is the specular/reflection color
- For non-metals: This is the diffuse color
- Alpha: Currently unused (reserved for transparency)

**Metallic** [0.0, 1.0]:
- 0.0: Dielectric (plastic, wood, stone) - strong diffuse, white specular
- 1.0: Metal (gold, iron, copper) - no diffuse, colored specular
- Values between: Rarely used, but supported for special effects

**Roughness** [0.0, 1.0]:
- 0.0: Smooth/glossy - sharp, mirror-like reflections
- 0.5: Medium - common for most materials
- 1.0: Rough/matte - diffuse-like appearance
- Maps to specular power: `shininess = (1 - roughness)²`

**Emissive** [0.0, ∞]:
- 0.0: No emission
- >0.0: Self-illuminated surface (multiplied by albedo)
- Common for lights, screens, magic effects
- In HDR: Can exceed 1.0 for bloom effects

### Applying Materials

```rust
use praxis_graphics::DrawCommand;

let draw_commands = vec![
    DrawCommand {
        mesh_id: "sphere".to_string(),
        model: transform_matrix,
        texture_name: Some("gold_texture".to_string()),
        material_properties: Some(MaterialProperties::new()
            .with_metallic(1.0)
            .with_roughness(0.2)),
    },
];
```

## Lighting

Forward rendering supports two light types:

### Directional Lights

Uniform light direction across the scene (sun, moon):

```rust
use praxis_graphics::{DirectionalLightData, LightingUniforms};

let mut lighting = LightingUniforms::default();

lighting.directional_lights[0] = DirectionalLightData {
    direction: [0.0, -1.0, -0.5, 0.0],  // Normalized direction
    color: [1.0, 0.95, 0.8, 0.0],        // Warm white
    intensity: 1.0,
    _padding: [0.0; 3],
};
lighting.directional_light_count = 1;
```

**Characteristics**:
- No distance attenuation
- Uniform direction everywhere
- Cheapest light type (no distance calculations)
- Best for primary scene illumination

### Point Lights

Omnidirectional lights with distance attenuation:

```rust
lighting.point_lights[0] = PointLightData {
    position: [5.0, 3.0, 2.0, 0.0],
    color: [1.0, 0.8, 0.5, 0.0],  // Orange glow
    intensity: 50.0,               // Brightness
    range: 20.0,                   // Maximum influence distance
};
lighting.point_light_count = 1;
```

**Attenuation**:
```glsl
float distance = length(light.position - fragment_pos);
float attenuation = 1.0 / (distance * distance);
attenuation *= smoothstep(light.range, light.range * 0.5, distance);
```

**Characteristics**:
- Inverse-square falloff (physically accurate)
- Range cutoff for performance
- More expensive than directional (distance calculations)
- Per-fragment position-dependent

### Ambient Lighting

Global ambient term to prevent pure black shadows:

```rust
lighting.ambient_color = [0.2, 0.2, 0.3, 0.0];  // Cool ambient
```

Applied as: `ambient = ambient_color * albedo`

## Shader Pipeline

### Vertex Shader

Transforms geometry and passes data to fragment shader:

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

Computes lighting using PBR model:

```glsl
#version 450

layout(location = 0) in vec3 v_world_pos;
layout(location = 1) in vec3 v_world_normal;
layout(location = 2) in vec2 v_uv;
layout(location = 3) in vec3 v_color;

layout(location = 0) out vec4 o_color;

layout(set = 0, binding = 2) uniform sampler2D u_texture;
layout(set = 0, binding = 3) uniform Material {
    float metallic;
    float roughness;
    float emissive;
} material;

layout(set = 0, binding = 4) uniform Lighting {
    DirectionalLight directional_lights[8];
    PointLight point_lights[16];
    vec4 ambient_color;
    uint directional_light_count;
    uint point_light_count;
} lighting;

void main() {
    // Sample textures
    vec3 albedo = texture(u_texture, v_uv).rgb * v_color;
    vec3 normal = normalize(v_world_normal);
    
    // Start with ambient
    vec3 color = lighting.ambient_color.rgb * albedo;
    
    // Add directional lights
    for (uint i = 0; i < lighting.directional_light_count; i++) {
        color += calculate_directional_light(
            lighting.directional_lights[i],
            normal,
            v_world_pos,
            albedo,
            material.metallic,
            material.roughness
        );
    }
    
    // Add point lights
    for (uint i = 0; i < lighting.point_light_count; i++) {
        color += calculate_point_light(
            lighting.point_lights[i],
            normal,
            v_world_pos,
            albedo,
            material.metallic,
            material.roughness
        );
    }
    
    // Add emissive
    color += albedo * material.emissive;
    
    o_color = vec4(color, 1.0);
}
```

## PBR Lighting Functions

Praxis uses Cook-Torrance microfacet BRDF:

### Distribution Function (GGX)

```glsl
float distribution_ggx(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    float a2 = a * a;
    float n_dot_h = max(dot(N, H), 0.0);
    float n_dot_h2 = n_dot_h * n_dot_h;
    
    float numerator = a2;
    float denominator = (n_dot_h2 * (a2 - 1.0) + 1.0);
    denominator = PI * denominator * denominator;
    
    return numerator / max(denominator, 0.0001);
}
```

### Geometry Function (Smith)

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

### Fresnel Function (Schlick)

```glsl
vec3 fresnel_schlick(float cos_theta, vec3 albedo, float metallic) {
    vec3 F0 = mix(vec3(0.04), albedo, metallic);
    return F0 + (1.0 - F0) * pow(1.0 - cos_theta, 5.0);
}
```

## Usage Example

Complete forward rendering setup:

```rust
use praxis_graphics::{RenderContext, DrawCommand, MaterialProperties, LightingUniforms};
use praxis_math::Mat4;

fn render_scene(render_context: &mut RenderContext) -> Result<()> {
    // Setup camera
    let view = camera.view_matrix();
    let proj = camera.projection_matrix();
    
    // Setup lighting
    let mut lighting = LightingUniforms::default();
    lighting.directional_lights[0] = DirectionalLightData {
        direction: [0.3, -0.8, 0.5, 0.0],
        color: [1.0, 1.0, 1.0, 0.0],
        intensity: 1.0,
        _padding: [0.0; 3],
    };
    lighting.directional_light_count = 1;
    lighting.ambient_color = [0.1, 0.1, 0.15, 0.0];
    
    // Create draw commands
    let draw_commands = vec![
        DrawCommand {
            mesh_id: "sphere".to_string(),
            model: Mat4::from_translation([0.0, 1.0, 0.0]),
            texture_name: Some("metal".to_string()),
            material_properties: Some(MaterialProperties::new()
                .with_metallic(1.0)
                .with_roughness(0.3)),
        },
    ];
    
    // Render
    render_context.render(&RenderCommands {
        view,
        proj,
        draw_commands: &draw_commands,
        lighting: Some(&lighting),
    })?;
    
    Ok(())
}
```

## Performance Optimization

### Light Culling

Only process lights that affect each object:

```rust
fn cull_lights(object: &Object, lights: &[PointLight]) -> Vec<usize> {
    lights.iter()
        .enumerate()
        .filter(|(_, light)| {
            let distance = (light.position - object.position()).length();
            distance < light.range + object.bounding_radius()
        })
        .map(|(idx, _)| idx)
        .collect()
}
```

### Shader Branching

Early exit for zero contribution:

```glsl
float n_dot_l = dot(normal, light_dir);
if (n_dot_l <= 0.0) {
    continue;  // Skip this light
}
```

### LOD for Lighting

Reduce lighting quality for distant objects:

```rust
let light_quality = if distance < 10.0 {
    LightQuality::High  // All lights
} else if distance < 50.0 {
    LightQuality::Medium  // Main lights only
} else {
    LightQuality::Low  // Ambient only
};
```

## When to Use Forward Rendering

| Scenario | Recommended | Reason |
|----------|-------------|--------|
| < 5 dynamic lights | ✓ Forward | Simple, efficient |
| Transparent objects | ✓ Forward | Native blending support |
| MSAA required | ✓ Forward | Hardware MSAA works |
| Many lights (10+) | ✗ Forward | Use deferred instead |
| Limited VRAM | ✓ Forward | No G-buffer overhead |

## Integration with Other Systems

### Shadows

Forward rendering integrates with cascaded shadow maps:

```glsl
float shadow_factor = calculate_shadow(v_world_pos, cascade_index);
vec3 light_contribution = /* ... */ * shadow_factor;
```

See [shadows.md](shadows.md) for details.

### HDR

Forward rendering can output to HDR targets:

```rust
let hdr_target = HdrRenderTarget::new(...)?;
render_context.render_to_target(&hdr_target, ...)?;
```

See [hdr-tonemapping.md](hdr-tonemapping.md) for details.

### Environment Probes

Forward rendering supports IBL (Image-Based Lighting):

```glsl
vec3 ambient = sample_environment_probe(normal, position);
```

See [environment-probes.md](environment-probes.md) for details.

## Examples

```bash
# Basic forward rendering
cargo run --example comprehensive_scene_demo

# Material showcase
cargo run --example material_demo

# Lighting demo
cargo run --example advanced_lighting_demo
```

## See Also

- [Deferred Rendering](deferred-rendering.md) - Alternative rendering approach
- [HDR and Tone Mapping](hdr-tonemapping.md) - High dynamic range
- [Shadows](shadows.md) - Shadow mapping
- [Post-Processing](post-processing.md) - Screen-space effects
- [Environment Probes](environment-probes.md) - Image-based lighting

## References

- [Real-Time Rendering, 4th Edition](http://www.realtimerendering.com/) - Chapter 9: Physically Based Shading
- [Learn OpenGL - PBR Theory](https://learnopengl.com/PBR/Theory)
- [Epic Games - Real Shading in Unreal Engine 4](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)
