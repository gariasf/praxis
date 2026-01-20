# Exercise 15: Shadow Mapping

**Difficulty**: 🔴 Advanced | **Estimated Time**: 5-6h | **Subsystem**: Graphics

## Overview

Implement shadow mapping - rendering shadows cast by directional lights. One of the most important lighting techniques in real-time graphics.

## Learning Objectives

- Understand shadow map generation
- Learn depth buffer rendering
- Implement shadow map sampling
- Handle shadow acne and peter-panning

## Requirements

### Functional Requirements

1. **Shadow Map Generation**
   - Render scene from light's perspective
   - Store depth in texture
   - Configure appropriate shadow map resolution

2. **Shadow Map Sampling**
   - Transform fragment position to light space
   - Sample shadow map
   - Compare depth values

3. **Shadow Quality**
   - PCF (Percentage Closer Filtering)
   - Bias to prevent shadow acne
   - Handle edge cases (out of shadow map bounds)

### Non-Functional Requirements

- **Performance**: Render shadows at 60 FPS
- **Quality**: Smooth shadow edges, minimal artifacts
- **Resolution**: 1024x1024 or 2048x2048 shadow map

## API Design

```rust
pub struct ShadowMapper {
    shadow_map: Arc<ImageView>,
    shadow_framebuffer: Arc<Framebuffer>,
    shadow_render_pass: Arc<RenderPass>,
    light_view_proj: Mat4,
}

impl ShadowMapper {
    pub fn new(device: Arc<Device>, resolution: u32) -> Result<Self>;
    
    pub fn render_shadow_map(
        &mut self,
        command_buffer: &mut AutoCommandBufferBuilder,
        scene: &Scene,
        light: &DirectionalLight,
    );
    
    pub fn get_shadow_map(&self) -> Arc<ImageView>;
    pub fn get_light_matrix(&self) -> Mat4;
}
```

## Validation Criteria

### Correctness
- [ ] Objects cast shadows
- [ ] Shadows positioned correctly
- [ ] Self-shadowing works (no acne)
- [ ] Soft shadow edges (with PCF)

### Performance
- [ ] 60 FPS with shadows enabled
- [ ] Shadow map render < 5ms
- [ ] Sampling overhead < 1ms per frame

## Test Cases

Manual/visual validation required:
- Place cube above plane with light overhead
- Cube should cast shadow on plane
- Shadow moves with light direction
- No flickering or artifacts

## Shaders

### Shadow Map Vertex Shader
```glsl
#version 450

layout(location = 0) in vec3 position;

layout(set = 0, binding = 0) uniform LightMatrix {
    mat4 light_view_proj;
};

layout(push_constant) uniform ModelMatrix {
    mat4 model;
};

void main() {
    gl_Position = light_view_proj * model * vec4(position, 1.0);
}
```

### Shadow Map Fragment Shader
```glsl
#version 450

// No output needed - depth written automatically
void main() {
}
```

### Main Scene Fragment Shader (with shadows)
```glsl
#version 450

layout(location = 0) in vec3 fragPos;
layout(location = 1) in vec3 fragNormal;
layout(location = 2) in vec4 fragPosLightSpace;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform sampler2D shadowMap;

float calculate_shadow(vec4 fragPosLightSpace) {
    // Perspective divide
    vec3 projCoords = fragPosLightSpace.xyz / fragPosLightSpace.w;
    
    // Transform to [0,1] range
    projCoords = projCoords * 0.5 + 0.5;
    
    // Get closest depth from light's perspective
    float closestDepth = texture(shadowMap, projCoords.xy).r;
    
    // Get current depth
    float currentDepth = projCoords.z;
    
    // Bias to prevent shadow acne
    float bias = 0.005;
    
    // PCF for soft shadows
    float shadow = 0.0;
    vec2 texelSize = 1.0 / textureSize(shadowMap, 0);
    for(int x = -1; x <= 1; ++x) {
        for(int y = -1; y <= 1; ++y) {
            float pcfDepth = texture(shadowMap, 
                projCoords.xy + vec2(x, y) * texelSize).r;
            shadow += currentDepth - bias > pcfDepth ? 1.0 : 0.0;
        }
    }
    shadow /= 9.0;
    
    // Keep shadows within shadow map bounds
    if(projCoords.z > 1.0)
        shadow = 0.0;
    
    return shadow;
}

void main() {
    vec3 lightColor = vec3(1.0);
    vec3 objectColor = vec3(0.8);
    
    // Ambient
    vec3 ambient = 0.3 * lightColor;
    
    // Diffuse
    vec3 lightDir = normalize(vec3(0.0, -1.0, 0.0));
    float diff = max(dot(fragNormal, -lightDir), 0.0);
    vec3 diffuse = diff * lightColor;
    
    // Shadow
    float shadow = calculate_shadow(fragPosLightSpace);
    vec3 lighting = (ambient + (1.0 - shadow) * diffuse) * objectColor;
    
    outColor = vec4(lighting, 1.0);
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Shadow map render | < 5ms |
| Scene render with shadows | < 16ms |
| Total frame time | < 16.67ms (60 FPS) |

## Hints & Guidance

### Light View Matrix
For directional light, use orthographic projection:
```rust
let light_view = Mat4::look_at_rh(
    light_position,
    light_position + light_direction,
    Vec3::Y
);

let light_proj = Mat4::orthographic_rh(
    -10.0, 10.0,  // left, right
    -10.0, 10.0,  // bottom, top
    0.1, 50.0     // near, far
);

let light_view_proj = light_proj * light_view;
```

### Shadow Acne
Add small bias when comparing depths:
```glsl
float bias = 0.005;
float shadow = currentDepth - bias > closestDepth ? 1.0 : 0.0;
```

### PCF (Percentage Closer Filtering)
Sample surrounding texels and average:
```glsl
float shadow = 0.0;
for(int x = -1; x <= 1; ++x) {
    for(int y = -1; y <= 1; ++y) {
        // Sample and compare
    }
}
shadow /= 9.0;
```

## Reference Implementation

See Vulkan tutorial shadow mapping and `praxis_graphics` shadow implementation.

## Related Resources

- [Learn OpenGL - Shadow Mapping](https://learnopengl.com/Advanced-Lighting/Shadows/Shadow-Mapping)
- [GPU Gems - Shadow Mapping](https://developer.nvidia.com/gpugems/gpugems/part-ii-lighting-and-shadows/chapter-11-shadow-map-antialiasing)
- [Praxis Lighting Guide](../../guides/rendering/lighting.md)

## Next Steps

- Implement cascaded shadow maps (CSM)
- Add variance shadow maps (VSM)
- Explore PCSS (percentage-closer soft shadows)
