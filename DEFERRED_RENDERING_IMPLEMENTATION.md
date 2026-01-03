# Deferred Rendering Implementation

This document describes the implementation of deferred rendering in the Praxis engine.

## Overview

Deferred rendering is a rendering technique that separates geometry processing from lighting calculations. This approach is particularly efficient for scenes with many lights, as lighting cost becomes O(lights × pixels) instead of O(lights × triangles).

## Architecture

### G-Buffer (Geometry Buffer)

The G-buffer consists of multiple render targets storing per-pixel geometry data:

1. **Albedo** (R8G8B8A8_UNORM)
   - RGB: Base color (texture × vertex color × material tint)
   - A: Unused

2. **Normal** (R16G16B16A16_SFLOAT)
   - RGB: World-space normal vector
   - A: Unused

3. **Metallic-Roughness** (R8G8B8A8_UNORM)
   - R: Metallic factor [0, 1]
   - G: Roughness factor [0, 1]
   - B: Emissive strength [0, ∞]
   - A: Unused

4. **Depth** (D32_SFLOAT)
   - Standard depth buffer for depth testing and world position reconstruction

### Two-Pass Rendering

#### Pass 1: Geometry Pass

Renders scene geometry to the G-buffer:

**Vertex Shader** (`deferred_geometry.vert`):
- Transforms vertices to clip space
- Passes world position, normal, color, and UV to fragment shader

**Fragment Shader** (`deferred_geometry.frag`):
- Samples albedo texture
- Combines texture × vertex color × material tint
- Writes albedo to G-buffer attachment 0
- Normalizes and writes world-space normal to attachment 1
- Packs material properties to attachment 2

**Pipeline Configuration**:
- Multiple render targets (3 color + 1 depth)
- Depth testing enabled (Less)
- Back-face culling enabled
- No color blending (opaque geometry only)

#### Pass 2: Lighting Pass

Full-screen pass that accumulates lighting from G-buffer data:

**Vertex Shader** (`deferred_lighting.vert`):
- Simple pass-through for full-screen quad
- Outputs clip-space position and UV

**Fragment Shader** (`deferred_lighting.frag`):
- Samples all G-buffer textures
- Reconstructs world position from depth using inverse projection
- Calculates view direction from camera position
- Computes lighting from directional and point lights
- Applies Blinn-Phong lighting model with PBR parameters
- Outputs final lit color

**Pipeline Configuration**:
- Single color attachment (swapchain)
- No depth testing (full-screen quad)
- No face culling
- No color blending

## Key Components

### `DeferredRenderer`

Main struct managing the deferred rendering pipeline:

```rust
pub struct DeferredRenderer {
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    
    geometry_pass: Arc<RenderPass>,
    lighting_pass: Arc<RenderPass>,
    
    geometry_pipeline: Arc<GraphicsPipeline>,
    lighting_pipeline: Arc<GraphicsPipeline>,
    
    gbuffer: Option<GBuffer>,
    
    fullscreen_quad_vertices: Subbuffer<[FullscreenVertex]>,
    fullscreen_quad_indices: Subbuffer<[u32]>,
    
    width: u32,
    height: u32,
}
```

**Methods**:
- `new()`: Creates renderer with specified dimensions
- `resize()`: Recreates G-buffer for new dimensions
- `render()`: Executes both rendering passes
- `geometry_pass_render()`: Renders meshes to G-buffer
- `lighting_pass_render()`: Accumulates lighting from G-buffer

### `GBuffer`

Container for G-buffer render targets:

```rust
pub struct GBuffer {
    pub albedo: Arc<ImageView>,
    pub normal: Arc<ImageView>,
    pub metallic_roughness: Arc<ImageView>,
    pub depth: Arc<ImageView>,
    pub framebuffer: Arc<Framebuffer>,
}
```

## Lighting Calculations

The lighting pass implements Blinn-Phong lighting with PBR material properties:

### Material Properties

- **Metallic**: Controls diffuse vs. specular behavior
  - 0.0: Dielectric (strong diffuse, white specular)
  - 1.0: Metal (no diffuse, colored specular)

- **Roughness**: Controls specular highlight sharpness
  - 0.0: Smooth/glossy (tight highlights, high shininess)
  - 1.0: Rough/matte (wide highlights, low shininess)

- **Emissive**: Self-illumination independent of lights
  - 0.0: No emission
  - \>0.0: Glowing surface (multiplied by albedo)

### Light Types

**Directional Lights**:
- Uniform direction across scene
- No distance attenuation
- Used for sun/moon lighting

**Point Lights**:
- Position-based omnidirectional lights
- Distance attenuation: `1 / (1 + d²)`
- Range cutoff for performance

### World Position Reconstruction

The lighting pass reconstructs world-space position from depth:

1. Sample depth from G-buffer
2. Convert UV and depth to NDC [-1, 1]
3. Apply inverse projection to get view space
4. Apply inverse view to get world space

This avoids storing world position explicitly, saving G-buffer memory.

## Performance Characteristics

### Benefits

- **Many Lights**: Lighting cost is O(lights × pixels) instead of O(lights × triangles)
- **Efficient Culling**: Occluded geometry doesn't consume lighting computation
- **Decoupled Shading**: Geometry complexity independent of lighting complexity
- **Flexible Post-Processing**: G-buffer data enables advanced effects

### Trade-offs

- **Memory**: Multiple full-screen render targets
  - 1920×1080: ~24 MB G-buffer (albedo 8MB + normal 16MB + material 8MB + depth 8MB)
- **Bandwidth**: Multiple render target writes and reads
- **Transparency**: Requires separate forward pass or order-independent transparency
- **MSAA**: Expensive with multiple render targets (consider TAA instead)

## Usage Example

```rust
use praxis_graphics::{DeferredRenderer, DrawCommand, RenderContext};

// Create deferred renderer
let deferred_renderer = DeferredRenderer::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
    1920,
    1080,
)?;

// In render loop
let draw_commands = vec![
    DrawCommand {
        mesh_id: "cube".to_string(),
        model: Mat4::IDENTITY,
        texture_name: Some("brick".to_string()),
        material_properties: Some(MaterialProperties::new()
            .with_metallic(0.0)
            .with_roughness(0.6)),
    },
];

// Render using deferred pipeline
deferred_renderer.render(
    &mut command_buffer_builder,
    output_framebuffer,
    viewport,
    &draw_commands,
    view_proj_buffer,
    &dynamic_uniform_buffer,
    mesh_manager,
    texture_manager,
    lighting_buffer,
)?;
```

## Integration with Forward Renderer

The deferred renderer coexists with the forward renderer:

- **Forward**: Good for transparent objects, skyboxes, particles
- **Deferred**: Good for opaque geometry with many lights
- **Hybrid**: Use deferred for opaque, forward for transparent

Applications can switch between renderers or use both in a single frame.

## Future Enhancements

Potential improvements to the deferred renderer:

1. **Light Culling**: Tile-based or cluster-based light culling
2. **Decal Support**: Render decals using G-buffer data
3. **SSAO**: Screen-space ambient occlusion using depth/normal
4. **SSR**: Screen-space reflections using G-buffer
5. **Thin G-Buffer**: Pack data more efficiently (e.g., octahedral normals)
6. **HDR**: Use floating-point formats for high dynamic range

## References

- Shishkovtsov, Oles (2005). "Deferred Shading in S.T.A.L.K.E.R."
- Valient, Michal (2007). "Deferred Rendering in Killzone 2"
- Kaplanyan, Anton (2010). "Light Pre-Pass Renderer Mark III"

## Files

**Rust Code**:
- `crates/praxis_graphics/src/deferred.rs` - Main implementation
- `crates/praxis_graphics/src/lib.rs` - Module exports and documentation

**Shaders**:
- `crates/praxis_graphics/src/shaders/deferred_geometry.vert` - Geometry pass vertex shader
- `crates/praxis_graphics/src/shaders/deferred_geometry.frag` - Geometry pass fragment shader
- `crates/praxis_graphics/src/shaders/deferred_lighting.vert` - Lighting pass vertex shader
- `crates/praxis_graphics/src/shaders/deferred_lighting.frag` - Lighting pass fragment shader

**Examples**:
- `examples/deferred_demo.rs` - Deferred rendering demonstration with many lights
