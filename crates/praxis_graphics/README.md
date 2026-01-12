# Praxis Graphics

Graphics system for the Praxis game engine, providing modern GPU rendering using Vulkan.

## Features

- **Forward and Deferred Rendering**: Choose the best approach for your scene
- **Physically-Based Rendering (PBR)**: Metallic-roughness workflow with proper material properties
- **Advanced Lighting**: Directional and point lights with shadow mapping
- **Bindless Rendering**: High-performance material system with texture arrays
- **Post-Processing**: HDR, tone mapping, bloom, SSAO, motion blur, and more
- **Skeletal Animation**: GPU-accelerated skeletal animation with up to 256 bones
- **Particle Systems**: GPU-based particle simulation and rendering
- **Spatial Optimization**: Frustum culling, occlusion culling, GPU-driven rendering

## Documentation

### Getting Started

- [`src/lib.rs`](src/lib.rs) - Comprehensive overview of the rendering architecture
- [`DESCRIPTOR_SETS_REFERENCE.md`](DESCRIPTOR_SETS_REFERENCE.md) - Quick reference for shader development

### Advanced Topics

- [`DESCRIPTOR_SET_AUDIT.md`](DESCRIPTOR_SET_AUDIT.md) - Complete audit of descriptor set layouts
- [`src/shaders/README.md`](src/shaders/README.md) - Shader documentation and conventions
- [`src/bindless.rs`](src/bindless.rs) - Bindless rendering system documentation

### Key Modules

- `pipeline.rs` - Graphics pipeline creation and management
- `render_context.rs` - Main rendering context and frame management
- `deferred.rs` - Deferred rendering implementation
- `lighting.rs` - Lighting system (directional, point, ambient)
- `shadow.rs` - Shadow mapping with cascaded shadow maps
- `particles.rs` - GPU-accelerated particle system
- `post_process/` - Post-processing effects

## Descriptor Set Layout

The Praxis graphics system uses a standardized three-set layout:

### Set 0: Per-Frame/Per-Draw Resources
- Camera matrices and position
- Model matrix (dynamic offset for per-object updates)
- Textures (albedo, normal maps)
- Lighting data (directional/point lights, ambient)
- Shadow maps and shadow data
- Bone matrices for skeletal animation

### Set 1: Per-Material Properties
- Material properties (base color, metallic, roughness, emissive)

### Set 2: Bindless Rendering (Optional)
- Texture array (up to 4096 textures)
- Material data buffer (up to 4096 materials)

See [`DESCRIPTOR_SETS_REFERENCE.md`](DESCRIPTOR_SETS_REFERENCE.md) for complete details and code examples.

## Usage Example

```rust
use praxis_graphics::{RenderContext, pipeline::create_simple_pipeline_3d};

// Create render context
let mut render_context = RenderContext::new(
    device,
    swapchain,
    memory_allocator,
    command_buffer_allocator,
    descriptor_set_allocator,
)?;

// Create pipeline
let pipeline = create_simple_pipeline_3d(
    &device,
    &render_pass,
    [width, height],
)?;

// Render frame
render_context.render(|ctx| {
    // Add meshes, lights, etc.
    ctx.add_mesh(mesh_id, &transform);
    ctx.add_directional_light(direction, color, intensity);
    
    Ok(())
})?;
```

## Performance Tips

1. **Use bindless rendering** for scenes with many materials to reduce descriptor set binds
2. **Enable GPU culling** for scenes with many objects to reduce draw calls
3. **Use deferred rendering** for scenes with many lights
4. **Batch similar materials** to minimize state changes
5. **Use dynamic offsets** for per-object data (already done by default for model matrices)

## Architecture

The graphics system is built on several key principles:

- **Explicit resource management**: Direct control over GPU memory and synchronization
- **Multi-threaded friendly**: Command buffer recording can be parallelized
- **Modern API usage**: Leverages Vulkan 1.2+ features like descriptor indexing
- **Educational code**: Extensive comments explain GPU concepts and techniques

## Shader Development

When creating new shaders:

1. Follow the standard descriptor set layout (see reference guide)
2. Use appropriate layout qualifiers (`std140` for uniforms, `std430` for storage buffers)
3. Document deviations from standard layout in shader comments
4. Update shader README with new shader descriptions
5. Test with validation layers enabled

## Dependencies

- **vulkano**: Rust wrapper for Vulkan API
- **glam**: Mathematics library for vectors and matrices
- **bytemuck**: Safe transmutation for GPU data structures
- **bevy_ecs**: Entity component system (optional, for integration)

## Contributing

When making changes to shaders or pipeline code:

1. Verify all descriptor set layouts remain consistent
2. Update `DESCRIPTOR_SET_AUDIT.md` if layouts change
3. Run with validation layers to catch Vulkan errors
4. Test with different hardware if possible
5. Document any new patterns or conventions

## License

See the main Praxis repository for license information.
