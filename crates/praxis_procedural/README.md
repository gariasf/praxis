# Praxis Procedural

GPU-accelerated procedural texture generation for the Praxis game engine.

## Overview

Runtime texture synthesis with noise functions, programmable graphs, GPU compute shaders, and automatic caching.

**Key Features:**
- Noise functions: Perlin, Simplex, Worley
- Node-based texture graphs (noise, blend, transform, color ramp, filters)
- GPU compute shader generation (GLSL → SPIR-V at runtime)
- Automatic LRU caching
- ~5-10ms generation time for 512x512 textures

## Quick Start

```rust
use praxis_procedural::{TextureGraph, TextureNode, NoiseType, TextureGenerationParams};

// Create graph
let mut graph = TextureGraph::new();
let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});
graph.set_output(noise_id);

// Generate texture (requires ProceduralTextureManager)
let params = TextureGenerationParams { width: 512, height: 512, seed: 42 };
let texture = manager.generate_texture(&graph, params)?;
```

## Node Types

- **Noise**: Perlin, Simplex, Worley with octave control
- **Transform**: Scale, rotate, translate coordinates
- **Blend**: Add, Multiply, Mix, Screen, Overlay, Subtract
- **ColorRamp**: Grayscale → color gradient mapping
- **Filters**: Invert, Clamp, Power, Threshold, Contrast, Brightness

## Documentation

**Comprehensive Guide:**
- [Procedural Textures Guide](../../docs/guides/assets/procedural-textures.md) - Complete usage and patterns

## Examples

```bash
cargo run --example procedural_texture_demo
```

## Performance

- GPU generation: 5-10ms for 512×512
- Cache hit rate: >90% typical
- Shader compilation: One-time cost per graph structure

## Dependencies

- `vulkano` 0.35.1: Vulkan compute
- `shaderc` 0.8: GLSL → SPIR-V compilation
- `seahash` 4.1: Fast hashing for cache keys
