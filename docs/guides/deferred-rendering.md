# Deferred Rendering Guide

## Overview

Deferred rendering is a rendering technique that separates geometry processing from lighting calculations, enabling efficient rendering of scenes with many lights. This guide explains the architecture, implementation, and usage of deferred rendering in Praxis.

## Why Deferred Rendering?

In traditional forward rendering, lighting is computed for every fragment of every triangle for every light. This results in O(lights × triangles) complexity. Deferred rendering decouples geometry from lighting, making lighting cost O(lights × pixels) instead—a significant improvement for scenes with many lights.

### Benefits

- **Many Lights**: Efficiently handles dozens or hundreds of dynamic lights
- **Efficient Culling**: Occluded geometry doesn't consume lighting computation
- **Decoupled Shading**: Geometry complexity is independent of lighting complexity
- **Post-Processing Foundation**: G-buffer data enables advanced effects like SSAO and SSR

### Trade-offs

- **Memory Overhead**: Multiple full-screen render targets required
  - 1920×1080: ~24 MB G-buffer (albedo 8MB + normal 16MB + material 8MB + depth 8MB)
- **Bandwidth**: Multiple render target writes and reads
- **Transparency**: Requires separate forward pass or order-independent transparency
- **MSAA**: Expensive with multiple render targets (consider TAA instead)

## Architecture

### G-Buffer (Geometry Buffer)

The G-buffer stores per-pixel geometry data across multiple render targets:

| Attachment | Format | Contents |
|------------|--------|----------|
| **Albedo** | R8G8B8A8_UNORM | RGB: Base color (texture × vertex color × material tint) |
| **Normal** | R16G16B16A16_SFLOAT | RGB: World-space normal vector |
| **Metallic-Roughness** | R8G8B8A8_UNORM | R: Metallic, G: Roughness, B: Emissive strength |
| **Depth** | D32_SFLOAT | Standard depth buffer for depth testing |

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
- Outputs clip-space position and UV coordinates

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

## Implementation Components

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

**Key Methods**:
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

## Material Properties and Lighting

### Material Properties

The deferred renderer uses physically-based material properties:

- **Metallic** [0.0, 1.0]: Controls diffuse vs. specular behavior
  - 0.0: Dielectric (strong diffuse, white specular)
  - 1.0: Metal (no diffuse, colored specular)

- **Roughness** [0.0, 1.0]: Controls specular highlight sharpness
  - 0.0: Smooth/glossy (tight highlights, high shininess)
  - 1.0: Rough/matte (wide highlights, low shininess)

- **Emissive** [0.0, ∞]: Self-illumination independent of lights
  - 0.0: No emission
  - >0.0: Glowing surface (multiplied by albedo)

### Light Types

**Directional Lights**:
- Uniform direction across scene
- No distance attenuation
- Ideal for sun/moon lighting

**Point Lights**:
- Position-based omnidirectional lights
- Distance attenuation: `1 / (1 + distance²)`
- Range cutoff for performance optimization

### World Position Reconstruction

The lighting pass reconstructs world-space position from depth to save G-buffer memory:

1. Sample depth from G-buffer
2. Convert UV and depth to NDC [-1, 1]
3. Apply inverse projection to get view space
4. Apply inverse view to get world space

This technique avoids explicitly storing world position, reducing G-buffer memory consumption.

## Usage Example

```rust
use praxis_graphics::{DeferredRenderer, DrawCommand, MaterialProperties};
use praxis_math::Mat4;

// Create deferred renderer
let deferred_renderer = DeferredRenderer::new(
    device.clone(),
    memory_allocator.clone(),
    descriptor_set_allocator.clone(),
    1920,
    1080,
)?;

// Define draw commands
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

## Hybrid Rendering

The deferred renderer coexists with the forward renderer, enabling hybrid approaches:

| Use Case | Renderer | Reason |
|----------|----------|--------|
| Opaque geometry | Deferred | Efficient with many lights |
| Transparent objects | Forward | Blending support |
| Skyboxes | Forward | No lighting needed |
| Particles | Forward | Alpha blending |

Applications can switch between renderers or use both in a single frame for optimal performance.

## Performance Characteristics

### Typical Performance (1920×1080, 1000 objects, 50 lights)

- **Geometry Pass**: ~8-12ms
- **Lighting Pass**: ~2-4ms
- **Total Frame Time**: ~10-16ms (60-100 FPS)

### Memory Usage

- **G-buffer**: ~24 MB for 1920×1080
- Scales linearly with resolution
- Consider dynamic resolution for lower-end hardware

### Optimization Tips

1. **Minimize G-buffer size**: Pack data efficiently
2. **Use tile-based lighting**: Reduce per-pixel light calculations
3. **Implement light culling**: Skip lights outside view frustum
4. **Batch draw calls**: Reduce render pass overhead
5. **Use compute shaders**: Offload lighting to compute queue

## Future Enhancements

Planned improvements to the deferred renderer:

1. **Light Culling**: Tile-based or cluster-based light culling for better performance
2. **Decal Support**: Render decals using G-buffer data
3. **SSAO**: Screen-space ambient occlusion using depth/normal
4. **SSR**: Screen-space reflections using G-buffer
5. **Thin G-Buffer**: Pack data more efficiently (e.g., octahedral normals)
6. **HDR Pipeline**: Use floating-point formats for high dynamic range

## See Also

- [Rendering Guide](rendering.md)
- [HDR and Tonemapping](hdr-and-tonemapping.md)
- [Shadows Guide](shadows.md)
- [Post-Processing Guide](post-processing.md)

## References

- Shishkovtsov, Oles (2005). "Deferred Shading in S.T.A.L.K.E.R."
- Valient, Michal (2007). "Deferred Rendering in Killzone 2"
- Kaplanyan, Anton (2010). "Light Pre-Pass Renderer Mark III"
