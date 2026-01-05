# Rendering Guides

Comprehensive guides for the Praxis rendering system, covering rendering pipelines, HDR, shadows, post-processing, and image-based lighting.

## Overview

Praxis provides a flexible rendering system built on Vulkan via `vulkano`. The system supports both forward and deferred rendering pipelines with full PBR (Physically-Based Rendering) support.

## Guides

### Core Rendering

- **[Forward Rendering](forward-rendering.md)** - Traditional single-pass rendering
  - Best for: Scenes with few lights, transparency, MSAA
  - Complexity: O(objects × triangles × lights)
  - Simple pipeline with native transparency support

- **[Deferred Rendering](deferred-rendering.md)** - Multi-pass rendering with G-buffer
  - Best for: Scenes with many lights (5+)
  - Complexity: O(objects × triangles) + O(pixels × lights)
  - Efficient light accumulation, decoupled geometry and lighting

### Lighting and Color

- **[HDR and Tone Mapping](hdr-tonemapping.md)** - High dynamic range rendering
  - Floating-point render targets for realistic lighting
  - Multiple tone mapping operators (Reinhard, ACES, Uncharted 2)
  - Automatic and manual exposure control
  - Essential for bloom and realistic lighting

- **[Shadows](shadows.md)** - Cascaded shadow maps with PCF
  - Multi-cascade shadow mapping for optimal quality
  - Percentage Closer Filtering for soft shadows
  - Configurable quality levels
  - Integration with forward and deferred rendering

- **[Environment Probes](environment-probes.md)** - Image-based lighting (IBL)
  - Realistic reflections and ambient lighting
  - Precomputed irradiance and specular maps
  - Multiple update modes (once, periodic, manual, continuous)
  - Spatial blending between probes

### Post-Processing

- **[Post-Processing](post-processing.md)** - Screen-space effects
  - **Bloom**: Glowing halos around bright areas
  - **Depth-of-Field**: Camera focus simulation
  - **Motion Blur**: Velocity-based blur
  - **Chromatic Aberration**: Lens color fringing
  - **Vignette**: Edge darkening
  - **Film Grain**: Procedural noise
  - Effect chaining and compositing

## Quick Reference

### Choosing a Rendering Pipeline

| Scenario | Pipeline | Reason |
|----------|----------|--------|
| < 5 lights | Forward | Simple, efficient |
| 5+ lights | Deferred | Scales better with lights |
| Transparency | Forward | Native blending |
| MSAA required | Forward | Hardware MSAA support |
| Screen-space effects | Deferred | Rich G-buffer data |

### Typical Rendering Setup

```rust
use praxis_graphics::{RenderContext, DeferredRenderer, ToneMapper};
use praxis_graphics::{ShadowMapManager, EnvironmentProbeManager};

// 1. Setup renderers
let mut render_context = RenderContext::new(...)?;
let mut deferred_renderer = DeferredRenderer::new(...)?;
let mut tone_mapper = ToneMapper::new(...)?;
let mut shadow_manager = ShadowMapManager::new(...)?;
let mut probe_manager = EnvironmentProbeManager::new(...)?;

// 2. Per-frame rendering
// Update shadows
shadow_manager.update(light_direction, view, proj)?;

// Update probes
probe_manager.update_probes()?;

// Render scene (deferred)
deferred_renderer.render(
    builder,
    hdr_target,
    viewport,
    draw_commands,
    view_proj_buffer,
    lighting_buffer,
)?;

// Apply post-processing
post_process_chain.process(&hdr_target, &processed_target)?;

// Tone map to LDR
tone_mapper.apply(&processed_target, &output)?;
```

## Rendering Pipeline Flow

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
              │ HDR Target    │
              └──────┬────────┘
                     │
              ┌──────▼────────┐
              │ Post-Process  │
              │ (Bloom, DOF,  │
              │  etc.)        │
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

## Performance Considerations

### Memory Usage

| System | Resolution | Memory |
|--------|------------|--------|
| Forward | 1920×1080 | ~8 MB (single target) |
| Deferred G-buffer | 1920×1080 | ~41 MB (4 targets) |
| HDR Target | 1920×1080 | ~16 MB (FP16 RGBA) |
| Shadow Maps (3×1024) | - | ~12 MB |
| Environment Probe (512) | - | ~14 MB per probe |

### Typical Frame Budget (1080p, 60fps)

| System | Cost | Optimization |
|--------|------|--------------|
| Geometry | 5-8ms | LOD, culling |
| Shadows | 2-4ms | Resolution, cascades |
| Lighting | 1-3ms | Light culling |
| Post-Processing | 3-6ms | Resolution scaling |
| Tone Mapping | <1ms | Negligible |

## Best Practices

### General

1. **Use deferred for many lights**: Switch at ~5 lights
2. **Enable HDR**: Essential for bloom and realistic lighting
3. **Apply post-processing selectively**: Not every effect is needed
4. **Profile regularly**: Measure actual performance on target hardware

### Quality vs Performance

**High Quality**:
- Deferred rendering
- 2048×2048 shadow maps, 4 cascades
- HDR with ACES tone mapping
- Full post-processing chain
- Multiple environment probes

**Balanced**:
- Deferred or forward (scene-dependent)
- 1024×1024 shadow maps, 3 cascades
- HDR with ACES tone mapping
- Bloom + vignette
- 1-2 environment probes

**Performance**:
- Forward rendering
- 512×512 shadow maps, 2 cascades
- LDR or simple HDR
- Minimal post-processing
- No environment probes or static only

## Examples

Each guide includes examples. Run them with:

```bash
# Forward rendering
cargo run --example comprehensive_scene_demo

# Deferred rendering
cargo run --example deferred_demo

# HDR and tone mapping
cargo run --example hdr_demo

# Shadows
cargo run --example shadow_demo

# Environment probes
cargo run --example environment_probe_demo

# Post-processing
cargo run --example bloom_demo
cargo run --example cinematic_post_processing_demo
```

## Additional Resources

- **[CLAUDE.md](../../CLAUDE.md)** - Architecture overview
- **[docs/concepts/vulkan-rendering.md](../../concepts/vulkan-rendering.md)** - Vulkan fundamentals
- **[crates/praxis_graphics/](../../crates/praxis_graphics/)** - Implementation source

## See Also

- **Material System**: PBR metallic-roughness workflow
- **Camera System**: Projection and view matrices
- **Asset System**: Mesh and texture loading
- **Scene System**: Transform hierarchy and animation
