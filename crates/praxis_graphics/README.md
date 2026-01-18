# Praxis Graphics

Vulkan-based rendering system for the Praxis game engine.

## Overview

Modern GPU rendering with forward/deferred pipelines, PBR materials, skeletal animation, particles, and post-processing.

**Key Features:**
- Forward and deferred rendering
- Physically-based rendering (PBR)
- Bindless texture/material system
- GPU-accelerated skeletal animation (256 bones)
- Particle systems
- Post-processing (HDR, bloom, SSAO, motion blur)
- Spatial optimization (frustum/GPU culling)

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

**Descriptor Set Layout (3 sets):**
- Set 0: Per-frame/per-draw (camera, model matrix, textures, lights, shadows, bones)
- Set 1: Per-material properties
- Set 2: Bindless rendering (texture arrays, material buffers)

See [`DESCRIPTOR_SETS_REFERENCE.md`](DESCRIPTOR_SETS_REFERENCE.md) for shader development.

## Documentation

**Comprehensive Guides:**
- [Rendering Overview](../../docs/guides/rendering.md)
- [Forward Rendering](../../docs/guides/rendering/forward-rendering.md)
- [Deferred Rendering](../../docs/guides/rendering/deferred-rendering.md)
- [PBR Materials](../../docs/guides/rendering/advanced-materials.md)
- [HDR & Tonemapping](../../docs/guides/rendering/hdr-tonemapping.md)
- [Shadows](../../docs/guides/rendering/shadows.md)
- [Particles](../../docs/guides/rendering/particles.md)

**Reference:**
- [Shaders Documentation](src/shaders/README.md)
- [Descriptor Set Audit](DESCRIPTOR_SET_AUDIT.md)
- [Mesh API Reference](../../docs/reference/mesh-api.md)

## Examples

```bash
cargo run --example hello_triangle
cargo run --example material_demo
cargo run --example advanced_lighting_demo
cargo run --example particles_demo
```

## Dependencies

- `vulkano` 0.35.1: Vulkan wrapper
- `glam` 0.30.4: Math library
- `bytemuck` 1.14: GPU data structures
