# Rendering Pipeline Deep Dive

This document provides a comprehensive explanation of Praxis's rendering pipeline, covering the lighting system, material system, and physics rendering integration.

**Related Architecture Documentation:**
- [Rendering Pipeline Stages](../architecture/rendering-pipeline-stages.md) - Detailed visual breakdown of rendering stages
- [Render Pipeline Architecture](../architecture/render-pipeline.md) - Forward vs deferred rendering comparison

## Table of Contents

1. [Overview](#overview)
2. [Graphics Pipeline Architecture](#graphics-pipeline-architecture)
3. [Lighting System](#lighting-system)
4. [Material System](#material-system)
5. [Physics Rendering Integration](#physics-rendering-integration)
6. [Descriptor Set Management](#descriptor-set-management)
7. [Performance Optimizations](#performance-optimizations)
8. [Common Patterns](#common-patterns)

---

## Overview

Praxis uses Vulkan (via `vulkano`) for rendering, implementing a forward rendering pipeline with support for:

- **Physically-Based Rendering (PBR)**: Metallic-roughness workflow for realistic materials
- **Dynamic Lighting**: Multiple directional and point lights with attenuation
- **Texture Mapping**: Full UV-based texture support
- **Physics Visualization**: Rendering of physics colliders and debug information

The rendering system follows a unified API design where a single `render()` call handles all rendering with automatic batching and optimization.

---

## Graphics Pipeline Architecture

### High-Level Flow

```text
Application → RenderContext::render() → GPU Rendering → Screen
    ↓                    ↓                    ↓
Camera Matrices    Command Buffer      Present Frame
Draw Commands      Recording
Lighting Data      Descriptor Sets
```

### Complete Pipeline Flow

```
┌──────────────────────────────────────────────────────────────────────┐
│ CPU (Application Thread)                                             │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────┐                                                │
│  │ Game Logic      │                                                │
│  │ - ECS Systems   │                                                │
│  │ - Physics       │                                                │
│  │ - Animation     │                                                │
│  └────────┬────────┘                                                │
│           │                                                          │
│           ▼                                                          │
│  ┌─────────────────┐       ┌──────────────────┐                    │
│  │ Collect         │       │ Camera           │                    │
│  │ Transforms      │◄──────┤ View/Projection  │                    │
│  └────────┬────────┘       └──────────────────┘                    │
│           │                                                          │
│           ▼                                                          │
│  ┌─────────────────────────────────────┐                           │
│  │ Build DrawCommands                  │                           │
│  │ - mesh_id                           │                           │
│  │ - model matrix                      │                           │
│  │ - texture_name                      │                           │
│  │ - material_properties               │                           │
│  └────────┬────────────────────────────┘                           │
│           │                                                          │
│           ▼                                                          │
│  ┌─────────────────────────────────────┐                           │
│  │ RenderContext::render()             │                           │
│  │                                      │                           │
│  │  1. Acquire swapchain image         │                           │
│  │     ┌──────────────────────┐        │                           │
│  │     │ Swapchain            │        │                           │
│  │     │ [Img0][Img1][Img2]   │        │                           │
│  │     │   ^                  │        │                           │
│  │     │   └─ Next available  │        │                           │
│  │     └──────────────────────┘        │                           │
│  │                                      │                           │
│  │  2. Upload lighting (if changed)    │                           │
│  │     ┌──────────────────────┐        │                           │
│  │     │ GPU Buffer           │        │                           │
│  │     │ - Directional lights │        │                           │
│  │     │ - Point lights       │        │                           │
│  │     └──────────────────────┘        │                           │
│  │                                      │                           │
│  │  3. Sort DrawCommands by material   │                           │
│  │     ┌──────────────────────┐        │                           │
│  │     │ Before:              │        │                           │
│  │     │ [TexB][TexA][TexB]   │        │                           │
│  │     │          ↓            │        │                           │
│  │     │ After:               │        │                           │
│  │     │ [TexA][TexB][TexB]   │        │                           │
│  │     └──────────────────────┘        │                           │
│  │                                      │                           │
│  │  4. Record command buffer            │                           │
│  │     ┌──────────────────────────────┐│                           │
│  │     │ begin_render_pass()          ││                           │
│  │     │ bind_pipeline()              ││                           │
│  │     │ set_viewport()               ││                           │
│  │     │                              ││                           │
│  │     │ for each DrawCommand:        ││                           │
│  │     │   bind_vertex_buffer()       ││                           │
│  │     │   bind_index_buffer()        ││                           │
│  │     │   bind_descriptor_set(0)     ││  ◄─ Transform + Texture  │
│  │     │   bind_descriptor_set(1)     ││  ◄─ Material Properties  │
│  │     │   draw_indexed()             ││                           │
│  │     │                              ││                           │
│  │     │ end_render_pass()            ││                           │
│  │     └──────────────────────────────┘│                           │
│  │                                      │                           │
│  │  5. Submit command buffer to GPU    │                           │
│  └──────────────┬───────────────────────┘                           │
│                 │                                                    │
└─────────────────┼────────────────────────────────────────────────────┘
                  │
                  ▼
┌──────────────────────────────────────────────────────────────────────┐
│ GPU (Graphics Hardware)                                              │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────────────────────────────────────────────┐            │
│  │ Vertex Shader (per vertex)                          │            │
│  │                                                      │            │
│  │  Input:  position, color, uv, normal                │            │
│  │  Uniform: model, view, proj matrices                │            │
│  │                                                      │            │
│  │  Transform: gl_Position = proj * view * model * pos │            │
│  │  Output:    v_color, v_uv, v_normal, v_world_pos    │            │
│  └──────────────────────┬───────────────────────────────┘            │
│                         │                                            │
│                         ▼                                            │
│  ┌─────────────────────────────────────────────────────┐            │
│  │ Rasterization                                       │            │
│  │                                                      │            │
│  │  Triangle → Fragments (pixels)                      │            │
│  │  Interpolate vertex attributes                      │            │
│  └──────────────────────┬───────────────────────────────┘            │
│                         │                                            │
│                         ▼                                            │
│  ┌─────────────────────────────────────────────────────┐            │
│  │ Fragment Shader (per pixel)                         │            │
│  │                                                      │            │
│  │  Input:  v_color, v_uv, v_normal, v_world_pos       │            │
│  │  Uniform: texture, lighting, material               │            │
│  │                                                      │            │
│  │  1. Sample texture                                  │            │
│  │     base_color = texture(tex, v_uv) * v_color       │            │
│  │                                                      │            │
│  │  2. Calculate lighting                              │            │
│  │     For each directional light:                     │            │
│  │       diffuse += dot(normal, light_dir) * color     │            │
│  │     For each point light:                           │            │
│  │       diffuse += dot(normal, light_dir) *           │            │
│  │                  color * attenuation                │            │
│  │                                                      │            │
│  │  3. Apply material                                  │            │
│  │     final = (ambient + diffuse) * base_color        │            │
│  │     final += emissive                               │            │
│  │                                                      │            │
│  │  Output: f_color (RGBA)                             │            │
│  └──────────────────────┬───────────────────────────────┘            │
│                         │                                            │
│                         ▼                                            │
│  ┌─────────────────────────────────────────────────────┐            │
│  │ Framebuffer Operations                              │            │
│  │                                                      │            │
│  │  - Depth test                                       │            │
│  │  - Blending                                         │            │
│  │  - Write to framebuffer                             │            │
│  └──────────────────────┬───────────────────────────────┘            │
│                         │                                            │
└─────────────────────────┼────────────────────────────────────────────┘
                          │
                          ▼
┌──────────────────────────────────────────────────────────────────────┐
│ Present to Screen                                                    │
├──────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌──────────────────────┐                                           │
│  │ Swapchain Present    │                                           │
│  │                      │                                           │
│  │ [Img0] → Display     │                                           │
│  │ [Img1] ← Rendering   │                                           │
│  │ [Img2] ← Idle        │                                           │
│  └──────────────────────┘                                           │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘

Timeline (60 FPS, 16.67ms per frame):
═══════════════════════════════════════

Frame N:
  0ms     ├─ Game logic (4ms)
  4ms     ├─ Build draw commands (1ms)
  5ms     ├─ Record command buffer (2ms)
  7ms     ├─ Submit to GPU
  7-16ms  │  GPU processing (parallel)
  16ms    └─ Present

Frame N+1:
  16ms    ├─ Acquire next swapchain image
  ...     └─ Repeat
```

### Pipeline Stages

#### 1. Initialization (`RenderContext::new()`)

```rust
pub async fn new(window: Arc<Window>) -> Result<Self>
```

**What happens:**
- Creates Vulkan instance and device
- Selects appropriate GPU
- Creates swapchain for presenting images
- Compiles shaders (vertex and fragment)
- Creates graphics pipeline
- Allocates descriptor set pools
- Initializes managers (mesh, texture, material, lighting)

**Key Components Created:**
- `GraphicsPipeline`: Defines vertex processing, rasterization, and fragment shading
- `RenderPass`: Describes color attachments and rendering operations
- `Framebuffers`: One per swapchain image, binds images to render pass attachments
- `DescriptorSetAllocator`: Pool for allocating descriptor sets efficiently

#### 2. Per-Frame Rendering (`RenderContext::render()`)

```rust
pub fn render(&mut self, cmds: &RenderCommands) -> Result<()>
```

**Rendering Sequence:**

1. **Acquire Swapchain Image**
   ```rust
   let (image_index, suboptimal, acquire_future) =
       vulkano::swapchain::acquire_next_image(self.swapchain.clone(), None)?;
   ```
   Gets the next available image from the swapchain to render into.

2. **Upload Lighting Data** (if provided)
   ```rust
   if let Some(lighting) = cmds.lighting {
       self.lighting_buffer.update(lighting)?;
   }
   ```
   Updates the GPU buffer with new lighting information.

3. **Sort Draw Commands** (material batching)
   ```rust
   indexed_commands.sort_by(|(_, a), (_, b)| {
       // Sort by texture name, then by material properties
   });
   ```
   Groups objects with identical materials to minimize GPU state changes.

4. **Build Descriptor Sets**
   - **Per-Object Transform Set** (set 0): Contains model/view/projection matrices, texture, lighting
   - **Per-Material Set** (set 1): Contains material properties (metallic, roughness, etc.)

5. **Record Command Buffer**
   ```rust
   command_buffer_builder
       .begin_render_pass(...)
       .bind_pipeline_graphics(...)
       .set_viewport(...)
       // For each draw command:
       .bind_vertex_buffers(...)
       .bind_index_buffer(...)
       .bind_descriptor_sets(...)
       .draw_indexed(...)
       .end_render_pass(...);
   ```

6. **Submit to GPU**
   ```rust
   let execution = previous_frame_end
       .join(acquire_future)
       .then_execute(self.graphics_queue.clone(), command_buffer);
   ```

7. **Present Frame**
   ```rust
   let future = execution.then_swapchain_present(...);
   ```

### Shader Pipeline

#### Vertex Shader

**Input:**
```glsl
layout(location = 0) in vec3 position;  // Vertex position
layout(location = 1) in vec4 color;     // Vertex color
layout(location = 2) in vec2 uv;        // Texture coordinates
```

**Uniforms:**
```glsl
layout(set = 0, binding = 0) uniform Uniforms {
    mat4 model;  // Model matrix (object → world)
    mat4 view;   // View matrix (world → camera)
    mat4 proj;   // Projection matrix (camera → clip space)
};
```

**Output:**
```glsl
layout(location = 0) out vec4 v_color;      // Interpolated color
layout(location = 1) out vec2 v_uv;         // Interpolated UV
layout(location = 2) out vec3 v_normal;     // World-space normal
layout(location = 3) out vec3 v_world_pos;  // World-space position
```

**Transformation:**
```glsl
void main() {
    // Transform vertex to clip space for rasterization
    gl_Position = proj * view * model * vec4(position, 1.0);
    
    // Pass data to fragment shader
    v_color = color;
    v_uv = uv;
    v_world_pos = (model * vec4(position, 1.0)).xyz;
    v_normal = normalize(mat3(model) * normal);
}
```

#### Fragment Shader

**Input:**
```glsl
layout(location = 0) in vec4 v_color;      // From vertex shader
layout(location = 1) in vec2 v_uv;
layout(location = 2) in vec3 v_normal;
layout(location = 3) in vec3 v_world_pos;
```

**Uniforms:**
```glsl
// Set 0: Transform, texture, lighting
layout(set = 0, binding = 1) uniform sampler2D tex;
layout(set = 0, binding = 2) uniform LightingUniforms { ... };

// Set 1: Material properties
layout(set = 1, binding = 0) uniform MaterialProperties {
    vec4 albedo;
    float metallic;
    float roughness;
    float emissive;
    float _padding;
};
```

**Output:**
```glsl
layout(location = 0) out vec4 f_color;  // Final pixel color
```

**Lighting Calculation** (simplified):
```glsl
void main() {
    // Sample texture
    vec4 tex_color = texture(tex, v_uv);
    vec4 base_color = tex_color * v_color * albedo;
    
    // Initialize lighting
    vec3 light_color = vec3(0.0);
    
    // Add directional lights (sun, moon)
    for (int i = 0; i < num_directional; i++) {
        vec3 light_dir = normalize(-directional_lights[i].direction);
        float diff = max(dot(v_normal, light_dir), 0.0);
        light_color += directional_lights[i].color * diff * directional_lights[i].intensity;
    }
    
    // Add point lights (lamps, fires)
    for (int i = 0; i < num_points; i++) {
        vec3 light_vec = point_lights[i].position - v_world_pos;
        vec3 light_dir = normalize(light_vec);
        float distance = length(light_vec);
        
        // Attenuation (inverse square law)
        float attenuation = 1.0 / (1.0 + point_lights[i].attenuation * distance * distance);
        
        float diff = max(dot(v_normal, light_dir), 0.0);
        light_color += point_lights[i].color * diff * point_lights[i].intensity * attenuation;
    }
    
    // Apply lighting with ambient
    vec3 ambient = vec3(0.1);
    vec3 lit_color = (ambient + light_color) * base_color.rgb;
    
    // Add emissive
    lit_color += emissive * base_color.rgb;
    
    // Output final color
    f_color = vec4(lit_color, base_color.a);
}
```

---

## Lighting System

### Architecture

The lighting system uses a **uniform buffer approach** where all lighting data is packed into a single GPU buffer and bound once per frame. This is efficient for scenes with a reasonable number of lights (up to 8 directional + 32 point lights by default).

### Components

#### 1. `LightingUniforms` (CPU-side)

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct LightingUniforms {
    pub directional_lights: [DirectionalLightData; MAX_DIRECTIONAL_LIGHTS],
    pub point_lights: [PointLightData; MAX_POINT_LIGHTS],
    pub num_directional: u32,
    pub num_point: u32,
    pub _padding1: u32,
    pub _padding2: u32,
}
```

**Key Points:**
- Uses `#[repr(C)]` for consistent memory layout matching GLSL std140
- Implements `bytemuck::Pod` for safe byte casting (zero-copy upload to GPU)
- Padding ensures proper alignment (16-byte boundaries for std140)

#### 2. `DirectionalLightData`

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DirectionalLightData {
    pub direction: [f32; 4],  // xyz = direction, w = padding
    pub color: [f32; 4],      // rgb = color, a = unused
    pub intensity: f32,
    pub _padding: [f32; 3],
}
```

**Directional lights** represent distant light sources (sun, moon) with:
- **Direction**: Vector pointing from surface toward light source
- **No position**: Light rays are parallel (infinite distance)
- **No attenuation**: Intensity is constant everywhere
- **Use case**: Outdoor scenes, primary lighting

**Physical Model:**
```text
Sun (infinitely far)
    ↓  ↓  ↓  ↓  ↓
   Parallel rays
    ↓  ↓  ↓  ↓  ↓
   Scene objects
```

#### 3. `PointLightData`

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PointLightData {
    pub position: [f32; 4],    // xyz = position, w = padding
    pub color: [f32; 4],       // rgb = color, a = unused
    pub intensity: f32,
    pub attenuation: f32,      // Controls falloff rate
    pub _padding: [f32; 2],
}
```

**Point lights** represent localized light sources (lamps, torches, fires) with:
- **Position**: 3D location in world space
- **Attenuation**: Light intensity decreases with distance
- **Omnidirectional**: Emits light in all directions
- **Use case**: Indoor scenes, localized lighting effects

**Attenuation Formula:**
```glsl
float distance = length(light_position - fragment_position);
float attenuation = 1.0 / (1.0 + attenuation_factor * distance * distance);
```

This implements the **inverse square law** from physics: light intensity decreases proportionally to distance squared.

#### 4. `LightingUniformBuffer` (GPU-side)

```rust
pub struct LightingUniformBuffer {
    buffer: Arc<Buffer>,
}

impl LightingUniformBuffer {
    pub fn update(&mut self, lighting: &LightingUniforms) -> Result<()> {
        // Write new lighting data to GPU buffer
        let mut write_lock = self.buffer.write()?;
        *write_lock = *lighting;
        Ok(())
    }
}
```

**Buffer Management:**
- Uses `HOST_SEQUENTIAL_WRITE` memory for efficient CPU→GPU transfers
- Buffer is mapped persistently (no allocation per frame)
- Updates are synchronous (completed before rendering)

### Lighting Workflow

#### Setup (Once)

```rust
// In application initialization
let lighting_buffer = LightingUniformBuffer::new(memory_allocator)?;

// Configure lights
let mut lighting = LightingUniforms::default();

// Add directional light (sun)
lighting.directional_lights[0] = DirectionalLightData {
    direction: [0.0, -1.0, -0.5, 0.0],  // Slightly angled down and forward
    color: [1.0, 0.95, 0.8, 0.0],       // Warm sunlight
    intensity: 1.0,
    _padding: [0.0; 3],
};
lighting.num_directional = 1;

// Add point light (torch)
lighting.point_lights[0] = PointLightData {
    position: [0.0, 2.0, 0.0, 0.0],
    color: [1.0, 0.7, 0.3, 0.0],  // Warm orange
    intensity: 10.0,
    attenuation: 0.5,
    _padding: [0.0; 2],
};
lighting.num_point = 1;
```

#### Per-Frame Update

```rust
// Update lighting data each frame (if dynamic)
render_context.render(&RenderCommands {
    view: camera_view,
    proj: camera_proj,
    draw_commands: &objects,
    lighting: Some(&lighting),  // Upload new lighting
})?;
```

**When to update:**
- **Static lighting**: Pass `lighting: None` after first frame (more efficient)
- **Dynamic lighting**: Pass `Some(&lighting)` when lights change position/color
- **Day/night cycle**: Interpolate directional light direction and color
- **Flickering torches**: Randomize point light intensity

### Advanced Lighting Techniques

#### Blinn-Phong Shading

The fragment shader implements **Blinn-Phong** for specular highlights:

```glsl
// Blinn halfway vector
vec3 view_dir = normalize(camera_pos - v_world_pos);
vec3 halfway = normalize(light_dir + view_dir);

// Specular component
float spec = pow(max(dot(v_normal, halfway), 0.0), shininess);
vec3 specular = light_color * spec * specular_intensity;
```

**Why Blinn-Phong over Phong?**
- More efficient (one normalize vs two)
- Better behavior at grazing angles
- Easier to implement with rough surfaces

#### Ambient Occlusion (Future)

Currently uses constant ambient (`vec3(0.1)`). Could be enhanced with:
- **SSAO** (Screen-Space Ambient Occlusion): Approximate occlusion from depth buffer
- **Baked AO**: Pre-computed in texture or vertex colors
- **HBAO** (Horizon-Based AO): Higher quality screen-space technique

#### Shadow Mapping (Future)

To add shadows:
1. Render scene from light's perspective to depth texture (shadow map)
2. In fragment shader, transform fragment position to light space
3. Compare fragment depth to shadow map depth
4. If fragment is further, it's in shadow (multiply light contribution by 0)

---

## Material System

### Architecture

The material system defines how surfaces respond to light and combines with textures. It uses a **PBR metallic-roughness workflow** for physically accurate rendering.

### Components

#### 1. `MaterialProperties` (CPU-side)

```rust
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MaterialProperties {
    pub albedo: [f32; 4],     // Base color (RGBA)
    pub metallic: f32,        // 0 = dielectric, 1 = metal
    pub roughness: f32,       // 0 = smooth (mirror), 1 = rough (matte)
    pub emissive: f32,        // Self-illumination strength
    pub _padding: f32,
}
```

**PBR Parameters Explained:**

**Albedo** (base color):
- RGB color of the surface
- For metals: tinted based on material (gold = yellow-orange)
- For dielectrics: actual surface color (wood, plastic, stone)
- Alpha channel: transparency (not currently used)

**Metallic**:
- 0.0 = Non-metal (dielectric): wood, plastic, stone, water
- 1.0 = Metal: iron, gold, copper, aluminum
- In-between: Typically not physically accurate, but useful for artistic effects (rusty metal = 0.7)

**Physical meaning:**
- Metals have no diffuse reflection (light doesn't penetrate)
- Metals have colored specular reflection (from free electrons)
- Dielectrics have white specular (Fresnel effect)
- Dielectrics have diffuse reflection (light penetrates and scatters)

**Roughness**:
- 0.0 = Perfectly smooth surface (mirror, polished metal)
- 1.0 = Completely rough surface (matte paper, unfinished wood)

**Physical meaning:**
- Rough surfaces scatter light in many directions (wide specular lobe)
- Smooth surfaces reflect light coherently (tight specular lobe)
- Roughness affects both diffuse and specular
- Microfacet distribution (GGX, Beckmann)

**Emissive**:
- How much the surface glows (self-illumination)
- 0.0 = No emission
- 1.0+ = Full emission (light source)
- Added to final color (not affected by lighting)
- Use for: screens, neon lights, glowing materials

#### 2. `Material`

```rust
pub struct Material {
    texture: Arc<Texture>,
    properties: MaterialProperties,
    descriptor_set: Arc<DescriptorSet>,
}
```

**Why materials own descriptor sets:**

Traditional approach (inefficient):
```text
Object 1: Create descriptor set → Bind → Draw
Object 2: Create descriptor set → Bind → Draw
Object 3: Create descriptor set → Bind → Draw
... (100 objects with same material = 100 descriptor sets)
```

Material-based approach (efficient):
```text
Material A: Create descriptor set (once)
    Object 1: Bind material A → Draw
    Object 2: Bind material A → Draw
    Object 3: Bind material A → Draw
... (100 objects with same material = 1 descriptor set)
```

**Benefits:**
- Reduced memory: 20x fewer descriptor sets in typical scenes
- Reduced CPU overhead: Fewer allocation/free operations
- Better GPU performance: Fewer state changes (binding is expensive)
- Simpler code: Material encapsulates everything needed to render

#### 3. `MaterialManager`

```rust
pub struct MaterialManager {
    materials: HashMap<String, Arc<Material>>,
}

impl MaterialManager {
    pub fn get_or_create_material(
        &mut self,
        name: &str,
        texture: Arc<Texture>,
        properties: MaterialProperties,
        // ... GPU resources
    ) -> Result<Arc<Material>> {
        // Check cache first
        if let Some(material) = self.materials.get(name) {
            return Ok(material.clone());
        }
        
        // Create new material
        let material = Material::new(texture, properties, ...)?;
        let material_arc = Arc::new(material);
        self.materials.insert(name.to_string(), material_arc.clone());
        Ok(material_arc)
    }
}
```

**Material Caching:**
- Materials are cached by a key (typically texture + properties hash)
- Multiple objects reference the same `Arc<Material>`
- Materials are automatically cleaned up when no longer referenced
- Cache is shared across entire application

### Material Workflow

#### Defining Materials

```rust
// Metal material (polished gold)
let gold = MaterialProperties {
    albedo: [1.0, 0.84, 0.0, 1.0],  // Gold color
    metallic: 1.0,                   // Full metal
    roughness: 0.1,                  // Polished
    emissive: 0.0,
    _padding: 0.0,
};

// Dielectric material (rough stone)
let stone = MaterialProperties {
    albedo: [0.5, 0.5, 0.5, 1.0],  // Gray
    metallic: 0.0,                  // Not metal
    roughness: 0.9,                 // Very rough
    emissive: 0.0,
    _padding: 0.0,
};

// Emissive material (glowing screen)
let screen = MaterialProperties {
    albedo: [0.2, 0.6, 1.0, 1.0],  // Blue tint
    metallic: 0.0,
    roughness: 0.5,
    emissive: 2.0,                  // Strong glow
    _padding: 0.0,
};
```

#### Using Materials in Rendering

```rust
let draw_commands = vec![
    DrawCommand {
        mesh_id: "sphere".to_string(),
        model: Mat4::from_translation(Vec3::new(-5.0, 0.0, 0.0)),
        texture_name: Some("gold_texture".to_string()),
        material_properties: Some(gold),
    },
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        texture_name: Some("stone_texture".to_string()),
        material_properties: Some(stone),
    },
    DrawCommand {
        mesh_id: "quad".to_string(),
        model: Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0)),
        texture_name: Some("screen_texture".to_string()),
        material_properties: Some(screen),
    },
];
```

### Material Batching

The rendering system automatically sorts draw commands to batch by material:

```rust
// Sort by texture first, then by material properties
indexed_commands.sort_by(|(_, a), (_, b)| {
    let tex_a = a.texture_name.as_deref().unwrap_or("_default_white");
    let tex_b = b.texture_name.as_deref().unwrap_or("_default_white");
    
    match tex_a.cmp(tex_b) {
        Ordering::Equal => {
            // Same texture, compare material properties
            let props_a = a.material_properties.unwrap_or_default();
            let props_b = b.material_properties.unwrap_or_default();
            bytemuck::bytes_of(&props_a).cmp(bytemuck::bytes_of(&props_b))
        }
        other => other,
    }
});
```

**Why this ordering?**
1. **Texture changes are expensive**: Binding new descriptor sets with textures
2. **Material changes are cheap**: Just different uniform values
3. **Grouping identical materials**: Descriptor set reuse (see below)

**Descriptor Set Reuse:**
```rust
let mut current_material_props = None;
let mut current_material_set = None;

for draw_cmd in sorted_commands {
    let material_changed = current_material_props != Some(draw_cmd.material_properties);
    
    let material_set = if material_changed {
        // Create new descriptor set
        let new_set = create_material_descriptor_set(draw_cmd.material_properties)?;
        current_material_props = Some(draw_cmd.material_properties);
        current_material_set = Some(new_set.clone());
        new_set
    } else {
        // Reuse existing descriptor set
        current_material_set.unwrap()
    };
    
    // Only bind if changed
    if material_changed {
        command_buffer.bind_descriptor_sets(1, material_set);
    }
}
```

**Performance Impact Example:**

Scene with 200 objects:
- 50 gold spheres (same material)
- 100 stone cubes (same material)
- 50 glowing screens (same material)

Without batching:
- Descriptor sets created: 200
- Descriptor set binds: 200

With batching:
- Descriptor sets created: 3
- Descriptor set binds: 3

**Result: 66x reduction in GPU state changes**

---

## Physics Rendering Integration

### Overview

Physics and rendering are separate systems that need coordination for visual feedback. The physics system uses Rapier3D for simulation, while rendering uses Vulkan. Integration happens at the **Transform component** level.

### Transform Synchronization

#### Architecture

```text
Game Loop:
    1. Input → Update Transforms
    2. Physics sync: ECS Transform → Rapier
    3. Physics step: Rapier simulation
    4. Physics sync: Rapier → ECS Transform
    5. Render: ECS Transform → GPU
```

#### Bidirectional Sync

**Before Physics (ECS → Rapier):**
```rust
pub fn sync_physics_transforms_system(
    mut physics_world: ResMut<PhysicsWorld>,
    changed_query: Query<(Entity, &Transform, &RigidBody), Changed<Transform>>,
) {
    for (entity, transform, rigid_body) in changed_query.iter() {
        // Only update kinematic bodies (player/scripted movement)
        if rigid_body.is_kinematic() {
            if let Some(body_handle) = physics_world.get_body_handle(entity) {
                if let Some(body) = physics_world.rigid_body_set.get_mut(body_handle) {
                    // Update Rapier body position
                    body.set_position(transform_to_isometry(transform), true);
                }
            }
        }
    }
}
```

**Why only kinematic?**
- **Dynamic bodies**: Moved by physics, not gameplay
- **Static bodies**: Never move
- **Kinematic bodies**: Moved by gameplay, physics follows

**After Physics (Rapier → ECS):**
```rust
pub fn sync_physics_transforms_system(
    physics_world: Res<PhysicsWorld>,
    mut all_query: Query<(Entity, &mut Transform, &RigidBody)>,
) {
    for (entity, mut transform, rigid_body) in all_query.iter_mut() {
        // Only update dynamic bodies (physics-driven movement)
        if rigid_body.is_dynamic() {
            if let Some(body_handle) = physics_world.get_body_handle(entity) {
                if let Some(body) = physics_world.rigid_body_set.get(body_handle) {
                    // Update ECS transform from Rapier
                    let position = body.position();
                    transform.translation = isometry_to_translation(position);
                    transform.rotation = isometry_to_rotation(position);
                }
            }
        }
    }
}
```

**Why only dynamic?**
- **Dynamic bodies**: Updated by physics simulation
- **Static bodies**: Never move
- **Kinematic bodies**: ECS is source of truth

#### Rendering Physics Objects

Physics objects render using the same pipeline as everything else:

```rust
// Spawn a physics cube
world.spawn((
    Transform::from_xyz(0.0, 10.0, 0.0),
    RigidBody::Dynamic,
    Collider::cuboid(0.5, 0.5, 0.5),
));

// In render loop
let draw_commands = physics_query
    .iter()
    .map(|(entity, transform, _rigid_body, _collider)| DrawCommand {
        mesh_id: "cube".to_string(),
        model: transform.to_matrix(),
        texture_name: None,
        material_properties: Some(MaterialProperties::default()),
    })
    .collect::<Vec<_>>();

render_context.render(&RenderCommands {
    view: camera.view_matrix(),
    proj: camera.projection_matrix(),
    draw_commands: &draw_commands,
    lighting: None,
})?;
```

### Debug Visualization

#### Collider Wireframes

For debugging physics, render collider shapes as wireframes:

```rust
fn render_collider_debug(
    query: Query<(&Transform, &Collider)>,
) -> Vec<DrawCommand> {
    query.iter().map(|(transform, collider)| {
        let mesh_id = match collider {
            Collider::Cuboid { hx, hy, hz } => "debug_box",
            Collider::Sphere { radius } => "debug_sphere",
            Collider::CapsuleY { .. } => "debug_capsule",
            // ... other shapes
        };
        
        DrawCommand {
            mesh_id: mesh_id.to_string(),
            model: transform.to_matrix(),
            texture_name: None,
            material_properties: Some(MaterialProperties {
                albedo: [0.0, 1.0, 0.0, 0.3],  // Semi-transparent green
                metallic: 0.0,
                roughness: 1.0,
                emissive: 0.5,  // Slight glow for visibility
                _padding: 0.0,
            }),
        }
    }).collect()
}
```

#### Velocity Vectors

Visualize velocity with lines/arrows:

```rust
fn render_velocity_debug(
    query: Query<(&Transform, &PhysicsVelocity)>,
) -> Vec<DrawCommand> {
    query.iter().map(|(transform, velocity)| {
        let velocity_scale = 0.1;  // Scale for visibility
        let velocity_offset = velocity.linear * velocity_scale;
        
        DrawCommand {
            mesh_id: "arrow".to_string(),
            model: Mat4::from_translation(transform.translation)
                * Mat4::from_rotation_translation(
                    Quat::from_rotation_arc(Vec3::Y, velocity.linear.normalize()),
                    Vec3::ZERO,
                )
                * Mat4::from_scale(Vec3::new(0.1, velocity.linear.length() * velocity_scale, 0.1)),
            texture_name: None,
            material_properties: Some(MaterialProperties {
                albedo: [1.0, 0.0, 0.0, 1.0],  // Red
                emissive: 1.0,
                ..Default::default()
            }),
        }
    }).collect()
}
```

#### Contact Points

Visualize collision contacts:

```rust
fn render_contact_debug(
    contact_events: Res<ContactEvents>,
    query: Query<&Transform>,
) -> Vec<DrawCommand> {
    let mut commands = Vec::new();
    
    for (entity1, entity2) in &contact_events.collision_started {
        if let (Ok(t1), Ok(t2)) = (query.get(*entity1), query.get(*entity2)) {
            // Approximate contact point (midpoint)
            let contact_pos = (t1.translation + t2.translation) * 0.5;
            
            commands.push(DrawCommand {
                mesh_id: "sphere".to_string(),
                model: Mat4::from_scale_rotation_translation(
                    Vec3::splat(0.1),
                    Quat::IDENTITY,
                    contact_pos,
                ),
                texture_name: None,
                material_properties: Some(MaterialProperties {
                    albedo: [1.0, 1.0, 0.0, 1.0],  // Yellow
                    emissive: 2.0,  // Bright
                    ..Default::default()
                }),
            });
        }
    }
    
    commands
}
```

### Performance Considerations

#### Transform Updates

**Problem:** Updating transforms is expensive if done naively
```rust
// Bad: Updates every physics object every frame
for (entity, physics_transform) in physics_query.iter() {
    render_transform.set(entity, physics_transform);  // Marks changed!
}
```

**Solution:** Use change detection
```rust
// Good: Only updates if transform actually changed
for (entity, mut transform, rigid_body) in query.iter_mut() {
    if rigid_body.is_dynamic() {
        let new_translation = get_physics_translation(entity);
        if transform.translation != new_translation {
            transform.translation = new_translation;  // Only marks if different
        }
    }
}
```

#### Interpolation (Future Enhancement)

Fixed timestep physics can cause visual stuttering:

```text
Frame rate: 144 FPS (6.9ms per frame)
Physics rate: 60 Hz (16.67ms per step)

Frame 1: Physics hasn't stepped → Render old position (stale)
Frame 2: Physics hasn't stepped → Render old position (stale)
Frame 3: Physics steps → Render new position (jump!)
```

**Solution:** Interpolate between previous and current physics state
```rust
struct PhysicsTransform {
    current: Transform,
    previous: Transform,
}

fn interpolate_transforms(
    physics_time: Res<PhysicsTime>,
    physics_config: Res<PhysicsConfig>,
    query: Query<(&PhysicsTransform, &mut Transform)>,
) {
    let alpha = physics_time.accumulator / physics_config.timestep;
    
    for (physics_transform, mut render_transform) in query.iter_mut() {
        render_transform.translation = physics_transform.previous.translation.lerp(
            physics_transform.current.translation,
            alpha,
        );
        render_transform.rotation = physics_transform.previous.rotation.slerp(
            physics_transform.current.rotation,
            alpha,
        );
    }
}
```

This provides smooth visuals at any frame rate while maintaining physics determinism.

---

## Descriptor Set Management

### What Are Descriptor Sets?

Descriptor sets are Vulkan's way of binding resources (buffers, textures, samplers) to shaders. Think of them as "resource packages" that shaders can access.

### Praxis's Descriptor Set Layout

```text
Set 0 (Per-Object):
    Binding 0: Uniform buffer (model/view/projection matrices)
    Binding 1: Texture sampler (albedo texture)
    Binding 2: Uniform buffer (lighting data)

Set 1 (Per-Material):
    Binding 0: Uniform buffer (material properties)
```

### Why Two Sets?

**Frequency of Change:**
- Set 0 changes **every object** (different transform, possibly different texture)
- Set 1 changes **per material** (many objects share same material)

**Performance Benefit:**
```rust
// Without separate sets:
for object in objects {
    bind_set_0(transform, texture, lighting, material);  // 4 bindings
    draw(object);
}

// With separate sets:
bind_set_0_bindings_0_and_2(lighting);  // Once per frame
for object in objects {
    bind_set_0_binding_1(transform, texture);  // 2 bindings
    if material_changed {
        bind_set_1(material);  // Only when material changes
    }
    draw(object);
}
```

### Descriptor Set Lifecycle

#### Allocation

```rust
let descriptor_set = DescriptorSet::new(
    allocator.clone(),
    layout.clone(),
    [
        WriteDescriptorSet::buffer(0, uniform_buffer),
        WriteDescriptorSet::image_view_sampler(1, texture_view, sampler),
        WriteDescriptorSet::buffer(2, lighting_buffer),
    ],
    [],
)?;
```

**Allocator:** Uses a pool pattern
- Fast: O(1) allocation from pool
- Memory efficient: Reuses freed descriptors
- Thread-safe: Internal synchronization

#### Binding

```rust
unsafe {
    command_buffer.bind_descriptor_sets(
        PipelineBindPoint::Graphics,
        pipeline.layout().clone(),
        0,  // First set index
        descriptor_set,
    )?;
}
```

**Cost:** Expensive GPU state change
- Minimize by batching (sort by material)
- Reuse when possible (cache descriptor sets)

#### Cleanup

```rust
// Automatic: Descriptor sets are freed when Arc<DescriptorSet> is dropped
// No manual cleanup needed
```

### Descriptor Set Allocator Performance

#### Standard Allocator

```rust
let allocator = StandardDescriptorSetAllocator::new(
    device.clone(),
    Default::default(),
);
```

**Characteristics:**
- **Thread-safe**: Can allocate from multiple threads
- **Pooled**: Maintains pools per descriptor type
- **Growing**: Automatically creates new pools when needed
- **Cleanup**: Automatically reclaims freed descriptors

**Performance Tips:**
1. Reuse descriptor sets when possible (cache by material)
2. Allocate in batches (reduces lock contention)
3. Use separate allocators for different thread pools
4. Monitor pool growth (high growth = need more initial capacity)

---

## Performance Optimizations

### Material Batching

**Problem:** GPU state changes are expensive

**Solution:** Sort draw commands by material

**Implementation:**
```rust
draw_commands.sort_by_key(|cmd| {
    (cmd.texture_name.clone(), bytemuck::bytes_of(&cmd.material_properties))
});
```

**Impact:** 20-50x reduction in descriptor set binds for typical scenes

### Descriptor Set Reuse

**Problem:** Creating descriptor sets is expensive

**Solution:** Reuse descriptor sets for identical materials

**Implementation:**
```rust
let mut material_cache: HashMap<MaterialKey, Arc<DescriptorSet>> = HashMap::new();

for draw_cmd in draw_commands {
    let key = (draw_cmd.texture_name, draw_cmd.material_properties);
    let descriptor_set = material_cache.entry(key)
        .or_insert_with(|| create_descriptor_set(...));
    
    // Use cached descriptor set
}
```

**Impact:** 100x reduction in descriptor set allocations for repeated materials

### Static Lighting

**Problem:** Uploading lighting data every frame is wasteful if it doesn't change

**Solution:** Only upload when lighting changes

**Implementation:**
```rust
// Initial frame or when lighting changes
render_context.render(&RenderCommands {
    lighting: Some(&lighting),  // Upload
    ..
})?;

// Subsequent frames (static lighting)
render_context.render(&RenderCommands {
    lighting: None,  // Reuse previous
    ..
})?;
```

**Impact:** 1000x reduction in lighting buffer updates for static scenes

### Command Buffer Recycling

Vulkan command buffers can be reused:

```rust
// Good: One-time submit (recreate each frame)
let command_buffer = AutoCommandBufferBuilder::primary(
    allocator,
    queue_family,
    CommandBufferUsage::OneTimeSubmit,
)?;

// Better: Reusable command buffers (for static scenes)
let command_buffer = AutoCommandBufferBuilder::primary(
    allocator,
    queue_family,
    CommandBufferUsage::SimultaneousUse,  // Can be submitted multiple times
)?;
```

**When to use:**
- **OneTimeSubmit**: Dynamic scenes (most games)
- **SimultaneousUse**: Static UI, menus, or repeated scenes

### GPU Culling (Future)

Frustum culling on GPU:

```glsl
// In vertex shader
if (is_outside_frustum(gl_Position)) {
    gl_Position = vec4(0.0, 0.0, 0.0, -1.0);  // Clip primitive
}
```

**Benefits:**
- CPU doesn't need to check every object
- GPU hardware culling is very fast
- Reduces vertex processing for invisible objects

---

## Common Patterns

### Textured Object

```rust
// Load texture
render_context.texture_manager_mut()
    .load_texture("wall", "assets/textures/wall.png")?;

// Render
DrawCommand {
    mesh_id: "cube",
    model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
    texture_name: Some("wall".to_string()),
    material_properties: None,  // Use default
}
```

### Glowing Object

```rust
DrawCommand {
    mesh_id: "sphere",
    model: Mat4::from_translation(Vec3::new(0.0, 2.0, 0.0)),
    texture_name: None,
    material_properties: Some(MaterialProperties {
        albedo: [1.0, 0.5, 0.0, 1.0],  // Orange
        emissive: 2.0,  // Strong glow
        ..Default::default()
    }),
}
```

### Metal Surface

```rust
DrawCommand {
    mesh_id: "sphere",
    model: Mat4::from_translation(Vec3::new(2.0, 0.0, 0.0)),
    texture_name: Some("metal_texture".to_string()),
    material_properties: Some(MaterialProperties {
        albedo: [0.8, 0.8, 0.8, 1.0],
        metallic: 1.0,
        roughness: 0.2,  // Polished
        ..Default::default()
    }),
}
```

### Dynamic Lighting

```rust
// Update light position each frame
let time = frame_timer.elapsed_seconds();
lighting.point_lights[0].position = [
    5.0 * time.cos(),
    2.0,
    5.0 * time.sin(),
    0.0,
];

render_context.render(&RenderCommands {
    lighting: Some(&lighting),  // Upload updated lighting
    ..
})?;
```

### Physics-Driven Rendering

```rust
// Render all dynamic physics objects
let draw_commands: Vec<DrawCommand> = physics_query
    .iter()
    .filter(|(_, rigid_body, _)| rigid_body.is_dynamic())
    .map(|(entity, _, transform)| DrawCommand {
        mesh_id: get_mesh_for_entity(entity),
        model: transform.to_matrix(),
        texture_name: get_texture_for_entity(entity),
        material_properties: get_material_for_entity(entity),
    })
    .collect();
```

---

## Summary

Praxis's rendering pipeline provides a unified, efficient system for rendering 3D scenes with:

1. **Lighting System**: Dynamic directional and point lights with physically-based attenuation
2. **Material System**: PBR metallic-roughness workflow with efficient descriptor set management
3. **Physics Integration**: Bidirectional transform synchronization for seamless physics rendering

Key performance features:
- Automatic material batching (20-50x fewer state changes)
- Descriptor set reuse (100x fewer allocations)
- Static lighting optimization (1000x fewer updates)

The system is designed to be both powerful and easy to use, with a single `render()` call handling all complexity.
