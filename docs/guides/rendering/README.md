# Rendering Guides

Comprehensive guides for the Praxis rendering system, covering rendering pipelines, HDR, shadows, post-processing, and image-based lighting.

## Overview

Praxis provides a flexible rendering system built on Vulkan via `vulkano`. The system supports both forward and deferred rendering pipelines with full PBR (Physically-Based Rendering) support.

## Core Guides

### Rendering Pipelines

- **[Forward Rendering](forward-rendering.md)** - Traditional single-pass rendering
  - Best for: Scenes with few lights, transparency, MSAA
  - Complexity: O(objects × triangles × lights)
  - Simple pipeline with native transparency support

- **[Deferred Rendering](deferred-rendering.md)** - Multi-pass rendering with G-buffer
  - Best for: Scenes with many lights (5+)
  - Complexity: O(objects × triangles) + O(pixels × lights)
  - Efficient light accumulation, decoupled geometry and lighting

- **[Advanced Rendering](advanced-rendering.md)** - Advanced rendering techniques and optimizations

### Lighting and Color

- **[HDR and Tone Mapping](hdr-tonemapping.md)** - High dynamic range rendering
  - Floating-point render targets for realistic lighting
  - Multiple tone mapping operators (Reinhard, ACES, Uncharted 2)
  - Automatic and manual exposure control

- **[Advanced Lighting](advanced-lighting.md)** - Light probes, volumetric effects, area lights
  - Light probe grids for indirect lighting
  - Volumetric fog and god rays
  - Area light approximations

- **[Shadows](shadows.md)** - Cascaded shadow maps with PCF
  - Multi-cascade shadow mapping for optimal quality
  - Percentage Closer Filtering for soft shadows
  - Configurable quality levels

- **[Environment Probes](environment-probes.md)** - Image-based lighting (IBL)
  - Realistic reflections and ambient lighting
  - Precomputed irradiance and specular maps
  - Multiple update modes

- **[Skybox](skybox.md)** - Sky rendering and environment backgrounds

### Materials

- **[Advanced Materials](advanced-materials.md)** - PBR and advanced material techniques
- **[Material Instancing](material-instancing.md)** - Efficient material management

### Post-Processing

- **[Post-Processing](post-processing.md)** - Screen-space effects pipeline
  - Bloom, Depth-of-Field, Motion Blur
  - Chromatic Aberration, Vignette, Film Grain
  
- **[Bloom](bloom.md)** - Detailed bloom implementation
- **[Cinematic Effects](cinematic-effects.md)** - Film-like visual effects

### Optimization

- **[LOD System](lod.md)** - Level of detail management
- **[GPU Culling](gpu-culling.md)** - GPU-driven visibility culling
- **[Frustum Culling](frustum-culling.md)** - CPU-side visibility culling

### Special Effects

- **[Particles](particles.md)** - Particle system effects
- **[Line Rendering](line-rendering.md)** - Debug and gizmo line rendering
  - [Quick Reference](line-rendering-quick-ref.md)

## Quick Reference

### Choosing a Rendering Pipeline

| Scenario | Pipeline | Reason |
|----------|----------|--------|
| < 5 lights | Forward | Simple, efficient |
| 5+ lights | Deferred | Scales better with lights |
| Transparency | Forward | Native blending |
| MSAA required | Forward | Hardware MSAA support |
| Screen-space effects | Deferred | Rich G-buffer data |

### Rendering Pipeline Flow

```
┌──────────────┐
│ Scene Setup  │
└──────┬───────┘
       │
       ├─────────────────────────────┐
       │                             │
┌──────▼────────┐          ┌─────────▼────────┐
│ Shadow Pass   │          │ Probe Capture    │
│ (Depth only)  │          │ (6 cubemap faces)│
└──────┬────────┘          └─────────┬────────┘
       │                             │
       └─────────────┬───────────────┘
                     │
              ┌──────▼────────┐
              │ Main Render   │
              │ (Forward or   │
              │  Deferred)    │
              └──────┬────────┘
                     │
              ┌──────▼────────┐
              │ Post-Process  │
              │ (Bloom, DOF)  │
              └──────┬────────┘
                     │
              ┌──────▼────────┐
              │ Tone Mapping  │
              │ (HDR → LDR)   │
              └──────┬────────┘
                     │
              ┌──────▼────────┐
              │ Final Output  │
              └───────────────┘
```

## Examples

```bash
# Forward rendering
cargo run --example comprehensive_scene_demo

# Advanced lighting
cargo run --example advanced_lighting_demo

# Environment probes
cargo run --example environment_probe_demo

# Materials
cargo run --example material_demo

# Particles
cargo run --example particles_demo
```

## See Also

- [Rendering Concepts](../../concepts/vulkan-rendering.md) - Vulkan fundamentals
- [Rendering Pipeline Explained](../../concepts/rendering-pipeline.md) - Theory and architecture
- [Spatial Optimization](../spatial-optimization.md) - Culling and LOD strategies
