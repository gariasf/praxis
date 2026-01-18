# Praxis Graphics

Vulkan-based rendering system for the Praxis game engine.

## Overview

Modern GPU rendering with forward/deferred pipelines, PBR materials, skeletal animation, particles, and advanced post-processing.

**Key Features:**
- Forward and deferred rendering pipelines
- Physically-based rendering (PBR) with material instancing
- Bindless texture/material system (4096 textures)
- GPU-accelerated skeletal animation (256 bones)
- Particle systems with collision detection
- Post-processing (HDR, bloom, SSAO, motion blur)
- Spatial optimization (frustum/GPU culling, LOD)
- Mesh streaming with background loading

## Quick Start

```rust
use praxis_graphics::RenderContext;

// Initialize rendering
let render_context = RenderContext::new(window).await?;

// Render a frame
render_context.render(&RenderCommands {
    view: camera.view,
    proj: camera.projection,
    draw_commands: &objects,
    lighting: Some(&lights),
})?;
```

## Architecture

### Descriptor Set Layout (3 sets)

- **Set 0**: Per-frame/per-draw (camera, model matrix, textures, lights, shadows, bones)
- **Set 1**: Per-material properties (base color, metallic, roughness, emissive)
- **Set 2**: Bindless rendering (texture arrays, material buffers)

See [`DESCRIPTOR_SETS_REFERENCE.md`](DESCRIPTOR_SETS_REFERENCE.md) for complete shader reference.

### Rendering Pipelines

**Forward Rendering:**
- Single-pass lighting calculation
- Per-pixel shading
- Suitable for simple scenes (<1000 objects)

**Deferred Rendering:**
- G-buffer pass (albedo, normal, metallic-roughness)
- Lighting pass (accumulate lights)
- Post-processing effects
- Suitable for complex scenes with many lights

## Documentation

### Core Systems

- [Descriptor Sets Reference](DESCRIPTOR_SETS_REFERENCE.md) - Shader development guide
- [Material System](MATERIAL_SYSTEM.md) - PBR materials and instancing
- [Bindless Rendering](BINDLESS_RENDERING.md) - High-performance material system
- [Descriptor Set Caching](DESCRIPTOR_SET_CACHING.md) - Automatic optimization

### Performance Systems

- [GPU Culling](GPU_CULLING.md) - Frustum culling on GPU
- [LOD System](LOD_SYSTEM.md) - CPU and GPU-driven LOD
- [GPU LOD Integration](GPU_LOD_INTEGRATION.md) - Integration guide
- [Mesh Streaming](MESH_STREAMING.md) - Background mesh loading

### Rendering Features

- [HDR Rendering](HDR_RENDERING.md) - HDR pipeline and tone mapping
- [Post-Processing](POST_PROCESSING.md) - Post-processing framework
- [Particles](PARTICLES.md) - GPU particle systems
- [Material Instancing](MATERIAL_INSTANCING.md) - Efficient material variants
- [Line Renderer](line_renderer_README.md) - Debug visualization

### Comprehensive Guides

- [Rendering Overview](../../docs/guides/rendering.md)
- [PBR Materials Guide](../../docs/guides/rendering/advanced-materials.md)
- [HDR & Tone Mapping](../../docs/guides/rendering/hdr-tonemapping.md)
- [Shadows Guide](../../docs/guides/rendering/shadows.md)

## Examples

```bash
# Basic rendering
cargo run --example hello_triangle
cargo run --example material_demo

# Advanced features
cargo run --example advanced_lighting_demo
cargo run --example particles_demo
cargo run --example gpu_culling_demo
cargo run --example lod_gpu_demo

# Post-processing
cargo run --example hdr_demo
cargo run --example bloom_demo

# Editor integration
cargo run --example editor_demo
cargo run --example selection_demo
```

## Dependencies

- `vulkano` 0.35.1 - Safe Vulkan wrapper
- `glam` 0.30.4 - Fast math library
- `bytemuck` 1.14 - Zero-copy GPU data structures
- `image` 0.24 - Texture loading
- `gltf` 0.16 - glTF model loading
