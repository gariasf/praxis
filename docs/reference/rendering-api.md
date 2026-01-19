# Rendering API Reference

API reference for Vulkan-based rendering system.

## Core Types

### RenderContext

Main rendering interface managing Vulkan resources.

```rust
pub struct RenderContext { /* ... */ }
```

**Methods:**
- `new(window, device, queue, allocator) -> Result<Self>`
- `render(commands: &RenderCommands) -> Result<()>`
- `mesh_manager() -> &MeshAssetManager`
- `mesh_manager_mut() -> &mut MeshAssetManager`
- `texture_manager() -> &TextureManager`
- `texture_manager_mut() -> &mut TextureManager`
- `create_render_pass(...) -> Result<Arc<RenderPass>>`
- `begin_frame() -> Result<FrameContext>`
- `end_frame(context: FrameContext) -> Result<()>`

### RenderCommands

Rendering command set for a frame.

```rust
pub struct RenderCommands<'a> {
    pub view: Mat4,
    pub proj: Mat4,
    pub draw_commands: &'a [DrawCommand],
    pub lighting: Option<&'a LightingData>,
}
```

### DrawCommand

Individual object draw command.

```rust
pub struct DrawCommand {
    pub mesh_id: String,
    pub model: Mat4,
    pub texture_name: Option<String>,
    pub material_properties: Option<MaterialProperties>,
}
```

## Meshes

### MeshData

CPU-side mesh representation.

```rust
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub colors: Option<Vec<[f32; 4]>>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub indices: Vec<u16>,
}
```

**Methods:**
- `new(positions, indices) -> Self`
- `with_colors(positions, colors, indices) -> Self`
- `with_normals(positions, normals, indices) -> Self`
- `set_colors(&mut self, colors)`
- `set_normals(&mut self, normals)`
- `set_uvs(&mut self, uvs)`
- `vertex_count() -> usize`
- `index_count() -> usize`
- `triangle_count() -> usize`

### GpuMesh

GPU-side mesh with Vulkan buffers.

```rust
pub struct GpuMesh {
    pub vertex_buffer: Subbuffer<[Vertex3D]>,
    pub index_buffer: Subbuffer<[u16]>,
    pub vertex_count: u32,
    pub index_count: u32,
}
```

**Methods:**
- `new(device, allocator, vertices, indices) -> Result<Self>`

### MeshAssetManager

Central mesh asset cache.

```rust
pub struct MeshAssetManager { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `load_mesh(id: &str, mesh: MeshData) -> Result<()>`
- `get_mesh(id: &str) -> Option<&GpuMesh>`
- `contains_mesh(id: &str) -> bool`
- `remove_mesh(id: &str) -> Option<GpuMesh>`
- `mesh_count() -> usize`
- `clear()`

## Materials & Textures

### MaterialProperties

PBR material properties.

```rust
pub struct MaterialProperties {
    pub base_color: [f32; 4],
    pub metallic: f32,
    pub roughness: f32,
    pub emissive_strength: f32,
}
```

**Methods:**
- `default()` - White diffuse material
- `metal(roughness: f32)` - Metallic material
- `dielectric(roughness: f32)` - Non-metallic material
- `emissive(color: [f32; 3], strength: f32)` - Self-illuminating

### Texture

Texture resource.

```rust
pub struct Texture {
    pub image: Arc<ImageView>,
    pub sampler: Arc<Sampler>,
}
```

**Methods:**
- `from_image_path(path: &str, device, allocator, queue) -> Result<Self>`
- `from_bytes(bytes: &[u8], width, height, device, allocator, queue) -> Result<Self>`
- `solid_color(color: [u8; 4], device, allocator, queue) -> Result<Self>`

### TextureManager

Texture asset cache.

```rust
pub struct TextureManager { /* ... */ }
```

**Methods:**
- `new() -> Self`
- `load_texture(name: &str, path: &str, ...) -> Result<()>`
- `get_texture(name: &str) -> Option<&Texture>`
- `contains_texture(name: &str) -> bool`
- `remove_texture(name: &str)`
- `clear()`

## Lighting

### LightingData

Scene lighting configuration.

```rust
pub struct LightingData {
    pub directional_lights: Vec<DirectionalLight>,
    pub point_lights: Vec<PointLight>,
    pub ambient: [f32; 3],
}
```

### DirectionalLight

Infinite directional light (sun).

```rust
pub struct DirectionalLight {
    pub direction: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
}
```

**Methods:**
- `new(direction: Vec3, color: [f32; 3], intensity: f32) -> Self`
- `sun(direction: Vec3) -> Self` - Preset sun light

### PointLight

Omnidirectional point light.

```rust
pub struct PointLight {
    pub position: Vec3,
    pub color: [f32; 3],
    pub intensity: f32,
    pub attenuation: f32,
}
```

**Methods:**
- `new(position: Vec3, color: [f32; 3], intensity: f32) -> Self`
- `with_attenuation(self, attenuation: f32) -> Self`

## Deferred Rendering

### DeferredRenderer

G-buffer based deferred rendering pipeline.

```rust
pub struct DeferredRenderer { /* ... */ }
```

**Methods:**
- `new(device, queue, allocator, swapchain_format) -> Result<Self>`
- `begin_geometry_pass() -> GeometryPass`
- `end_geometry_pass(pass: GeometryPass)`
- `lighting_pass(lighting: &LightingData) -> Result<()>`
- `get_gbuffer() -> &GBuffer`

### GBuffer

Geometry buffer for deferred rendering.

```rust
pub struct GBuffer {
    pub albedo: Arc<ImageView>,      // RGB: color, A: unused
    pub normal: Arc<ImageView>,      // RGB: normal, A: unused
    pub position: Arc<ImageView>,    // RGB: position, A: unused
    pub material: Arc<ImageView>,    // R: metallic, G: roughness, B: emissive
    pub depth: Arc<ImageView>,
}
```

## Post-Processing

### ToneMapper

HDR to LDR tone mapping.

```rust
pub struct ToneMapper { /* ... */ }
```

**Methods:**
- `new(device, allocator) -> Result<Self>`
- `set_operator(operator: ToneMappingOperator)`
- `set_exposure(exposure: f32)`
- `apply(input: Arc<ImageView>, output: Arc<ImageView>) -> Result<()>`

### ToneMappingOperator

Tone mapping algorithms.

```rust
pub enum ToneMappingOperator {
    ACES,         // ACES filmic (default, cinematic)
    Reinhard,     // Reinhard (simple, fast)
    Uncharted2,   // Uncharted 2 (game-like)
    Linear,       // No tone mapping
}
```

## Primitive Meshes

Built-in mesh generators.

```rust
// Colored cube (different color per vertex)
pub fn colored_cube_mesh() -> MeshData;

// Solid color cube
pub fn solid_cube_mesh(color: [f32; 3]) -> MeshData;

// Flat quad facing up
pub fn quad_mesh(size: f32, color: [f32; 3]) -> MeshData;

// 4-sided pyramid
pub fn pyramid_mesh(base_color: [f32; 3], tip_color: [f32; 3]) -> MeshData;

// UV sphere
pub fn sphere_mesh(radius: f32, segments: u32, rings: u32) -> MeshData;
```

## Common Patterns

### Basic Rendering Setup

```rust
use praxis_graphics::{RenderContext, RenderCommands, DrawCommand};

// Create context
let render_context = RenderContext::new(window, device, queue, allocator)?;

// Load mesh
render_context.mesh_manager_mut().load_mesh(
    "cube",
    colored_cube_mesh(),
)?;

// Per frame
let commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        texture_name: None,
        material_properties: None,
    },
];

let render_commands = RenderCommands {
    view: camera_view,
    proj: camera_proj,
    draw_commands: &commands,
    lighting: None,
};

render_context.render(&render_commands)?;
```

### PBR Materials

```rust
use praxis_graphics::MaterialProperties;

// Metal
let metal = MaterialProperties::metal(0.1);  // Polished

// Plastic
let plastic = MaterialProperties::dielectric(0.8);

// Custom
let custom = MaterialProperties {
    base_color: [0.8, 0.2, 0.2, 1.0],
    metallic: 0.0,
    roughness: 0.4,
    emissive_strength: 0.0,
};

// Emissive
let emissive = MaterialProperties::emissive([1.0, 0.8, 0.0], 2.0);
```

### Lighting Setup

```rust
use praxis_graphics::{LightingData, DirectionalLight, PointLight};

let lighting = LightingData {
    directional_lights: vec![
        DirectionalLight::sun(Vec3::new(-0.3, -1.0, -0.5)),
    ],
    point_lights: vec![
        PointLight::new(
            Vec3::new(0.0, 2.0, 0.0),
            [1.0, 0.8, 0.6],
            10.0,
        ).with_attenuation(1.0),
    ],
    ambient: [0.1, 0.1, 0.15],
};
```

### Texture Usage

```rust
// Load texture
render_context.texture_manager_mut().load_texture(
    "brick",
    "assets/textures/brick.png",
    device,
    allocator,
    queue,
)?;

// Use in draw command
DrawCommand {
    mesh_id: "cube".to_string(),
    model: transform,
    texture_name: Some("brick".to_string()),
    material_properties: Some(MaterialProperties::default()),
}
```

### Deferred Rendering

```rust
use praxis_graphics::DeferredRenderer;

let mut deferred = DeferredRenderer::new(
    device,
    queue,
    allocator,
    swapchain_format,
)?;

// Geometry pass
let geom_pass = deferred.begin_geometry_pass();
for command in draw_commands {
    // Render to G-buffer
}
deferred.end_geometry_pass(geom_pass);

// Lighting pass
deferred.lighting_pass(&lighting)?;

// Access results
let gbuffer = deferred.get_gbuffer();
```

### HDR & Tone Mapping

```rust
use praxis_graphics::{ToneMapper, ToneMappingOperator};

let mut tone_mapper = ToneMapper::new(device, allocator)?;
tone_mapper.set_operator(ToneMappingOperator::ACES);
tone_mapper.set_exposure(1.0);

// Apply to HDR image
tone_mapper.apply(hdr_image, ldr_output)?;
```

## See Also

- [Rendering Guide](../guides/rendering.md) - Rendering overview
- [Rendering Guides](../guides/rendering/README.md) - Comprehensive rendering documentation
- [HDR/Tone Mapping Guide](../guides/rendering/hdr-tonemapping.md) - HDR workflow
- [Deferred Rendering Guide](../guides/rendering/deferred-rendering.md) - G-buffer details
- [praxis_graphics Crate](../../crates/praxis_graphics/README.md) - Crate documentation
- [Shaders Reference](shaders.md) - Shader bindings and conventions
