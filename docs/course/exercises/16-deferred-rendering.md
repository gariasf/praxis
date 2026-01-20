# Exercise 16: Deferred Rendering Pipeline

**Difficulty**: 🔴 Advanced | **Estimated Time**: 6-8h | **Subsystem**: Graphics

## Overview

Implement a deferred rendering pipeline that separates geometry rendering from lighting calculations. Essential technique for scenes with many lights.

## Learning Objectives

- Understand deferred vs forward rendering trade-offs
- Learn G-buffer layout and management
- Implement multi-pass rendering
- Optimize lighting performance with many lights

## Requirements

### Functional Requirements

1. **G-Buffer Creation**
   - Position buffer (RGB: world position)
   - Normal buffer (RGB: world normal)
   - Albedo buffer (RGB: base color)
   - Metallic-Roughness buffer (RG: metallic/roughness)

2. **Geometry Pass**
   - Render scene geometry to G-buffer
   - Store material properties per pixel
   - Efficient packing of data

3. **Lighting Pass**
   - Full-screen quad
   - Sample G-buffer
   - Apply all lights in single pass
   - Output final color

4. **Transparency Handling**
   - Forward pass for transparent objects
   - Composite with deferred result

### Non-Functional Requirements

- **Performance**: Handle 100+ lights at 60 FPS
- **Memory**: Reasonable G-buffer size (< 200MB at 1080p)
- **Quality**: No visible artifacts from precision issues

## API Design

```rust
pub struct DeferredRenderer {
    gbuffer: GBuffer,
    lighting_pipeline: Arc<GraphicsPipeline>,
    geometry_pipeline: Arc<GraphicsPipeline>,
}

pub struct GBuffer {
    position: Arc<ImageView>,
    normal: Arc<ImageView>,
    albedo: Arc<ImageView>,
    metallic_roughness: Arc<ImageView>,
    depth: Arc<ImageView>,
}

impl DeferredRenderer {
    pub fn new(device: Arc<Device>, extent: [u32; 2]) -> Result<Self>;
    
    pub fn render_geometry_pass(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder,
        scene: &Scene,
        camera: &Camera,
    );
    
    pub fn render_lighting_pass(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder,
        lights: &[Light],
    );
}
```

## Validation Criteria

### Correctness
- [ ] G-buffer contains correct data
- [ ] Lighting matches forward rendering
- [ ] Multiple lights accumulate correctly
- [ ] Transparent objects render properly

### Performance
- [ ] 100 point lights @ 60 FPS
- [ ] G-buffer bandwidth acceptable
- [ ] Lighting pass scales with light count

## Shaders

### Geometry Pass Vertex Shader
```glsl
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 uv;

layout(location = 0) out vec3 fragPos;
layout(location = 1) out vec3 fragNormal;
layout(location = 2) out vec2 fragUV;

layout(set = 0, binding = 0) uniform Camera {
    mat4 view;
    mat4 proj;
};

layout(push_constant) uniform Model {
    mat4 model;
};

void main() {
    vec4 worldPos = model * vec4(position, 1.0);
    fragPos = worldPos.xyz;
    fragNormal = mat3(model) * normal;
    fragUV = uv;
    
    gl_Position = proj * view * worldPos;
}
```

### Geometry Pass Fragment Shader
```glsl
#version 450

layout(location = 0) in vec3 fragPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec2 fragUV;

layout(location = 0) out vec4 gPosition;
layout(location = 1) out vec4 gNormal;
layout(location = 2) out vec4 gAlbedo;
layout(location = 3) out vec4 gMetallicRoughness;

layout(set = 1, binding = 0) uniform sampler2D albedoMap;
layout(set = 1, binding = 1) uniform sampler2D normalMap;

void main() {
    gPosition = vec4(fragPos, 1.0);
    gNormal = vec4(normalize(fragNormal), 1.0);
    gAlbedo = texture(albedoMap, fragUV);
    gMetallicRoughness = vec4(0.0, 0.5, 0.0, 1.0); // metallic, roughness
}
```

### Lighting Pass Vertex Shader
```glsl
#version 450

layout(location = 0) out vec2 fragUV;

// Full-screen triangle
vec2 positions[3] = vec2[](
    vec2(-1.0, -1.0),
    vec2(3.0, -1.0),
    vec2(-1.0, 3.0)
);

vec2 uvs[3] = vec2[](
    vec2(0.0, 0.0),
    vec2(2.0, 0.0),
    vec2(0.0, 2.0)
);

void main() {
    fragUV = uvs[gl_VertexIndex];
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}
```

### Lighting Pass Fragment Shader
```glsl
#version 450

layout(location = 0) in vec2 fragUV;
layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform sampler2D gPosition;
layout(set = 0, binding = 1) uniform sampler2D gNormal;
layout(set = 0, binding = 2) uniform sampler2D gAlbedo;
layout(set = 0, binding = 3) uniform sampler2D gMetallicRoughness;

struct Light {
    vec4 position; // w: radius
    vec4 color;    // rgb: color, a: intensity
};

layout(set = 0, binding = 4) uniform Lights {
    Light lights[100];
    int lightCount;
};

layout(set = 0, binding = 5) uniform Camera {
    vec3 viewPos;
};

vec3 calculateLighting(vec3 fragPos, vec3 normal, vec3 albedo, 
                      float metallic, float roughness, vec3 viewPos) {
    vec3 result = vec3(0.0);
    vec3 V = normalize(viewPos - fragPos);
    
    // Ambient
    result += albedo * 0.1;
    
    // Point lights
    for(int i = 0; i < lightCount; ++i) {
        vec3 L = lights[i].position.xyz - fragPos;
        float distance = length(L);
        L = normalize(L);
        
        if(distance > lights[i].position.w) continue;
        
        // Attenuation
        float attenuation = 1.0 / (1.0 + 0.09 * distance + 0.032 * distance * distance);
        
        // Diffuse
        float diff = max(dot(normal, L), 0.0);
        vec3 diffuse = diff * albedo * lights[i].color.rgb * lights[i].color.a;
        
        result += diffuse * attenuation;
    }
    
    return result;
}

void main() {
    vec3 fragPos = texture(gPosition, fragUV).rgb;
    vec3 normal = texture(gNormal, fragUV).rgb;
    vec3 albedo = texture(gAlbedo, fragUV).rgb;
    vec2 metallicRoughness = texture(gMetallicRoughness, fragUV).rg;
    
    vec3 lighting = calculateLighting(
        fragPos, normal, albedo,
        metallicRoughness.r, metallicRoughness.g,
        viewPos
    );
    
    outColor = vec4(lighting, 1.0);
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Geometry pass | < 8ms |
| Lighting pass (100 lights) | < 6ms |
| Total frame time | < 16.67ms (60 FPS) |
| G-buffer memory (1080p) | < 100MB |

## Hints & Guidance

### G-Buffer Format Selection
```rust
// Position: RGBA32F (high precision)
// Normal: RGBA16F (sufficient precision)
// Albedo: RGBA8 (color texture)
// Metallic/Roughness: RG8 (2 channels sufficient)
```

### Light Culling
For many lights, use tile-based or clustered deferred rendering:
1. Divide screen into tiles (16x16 pixels)
2. Compute which lights affect each tile
3. Only evaluate relevant lights per pixel

### Memory Optimization
- Pack normal into 2 channels, reconstruct Z
- Use view-space positions (smaller range)
- Share depth buffer with forward pass

## Reference Implementation

See `praxis_graphics` deferred renderer implementation and modern engine examples.

## Related Resources

- [Learn OpenGL - Deferred Shading](https://learnopengl.com/Advanced-Lighting/Deferred-Shading)
- [GPU Gems 2 - Deferred Shading](https://developer.nvidia.com/gpugems/gpugems2/part-ii-shading-lighting-and-shadows/chapter-9-deferred-shading-tabula-rasa)
- [Praxis Rendering Guide](../../guides/rendering.md)

## Next Steps

- Implement tile-based deferred rendering
- Add SSAO (screen-space ambient occlusion)
- Explore clustered forward rendering hybrid
