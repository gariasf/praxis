# praxis_procedural

Procedural texture generation for Praxis engine.

## Overview

Node-based procedural texture generation with GPU acceleration.

## Features

### Texture Graph

- Node-based composition system
- DAG (Directed Acyclic Graph) structure
- Visual programming for textures

### Noise Functions

- **Perlin Noise**: Smooth, natural patterns
- **Simplex Noise**: Better gradients than Perlin
- **Worley Noise**: Cellular patterns
- **Fractional Brownian Motion (FBM)**: Multiple octaves

### Operations

- **Blend**: Mix textures with various modes
- **Transform**: Rotate, scale, translate
- **Filter**: Blur, sharpen, edge detect
- **Color**: Remap, gradient, HSV adjust

### GPU Generation

- Runtime GLSL compilation to SPIR-V
- Compute shader dispatch
- Efficient GPU-based generation

### Caching

- LRU cache for generated textures
- Configurable cache size
- Disk serialization (optional)

## Example

```rust
use praxis_procedural::{TextureGraph, TextureNode, NoiseType};

let mut graph = TextureGraph::new();

// Add noise node
let noise = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Add color remap
let remap = graph.add_node(TextureNode::ColorRemap {
    input: noise,
    gradient: vec![
        (0.0, Vec3::new(0.0, 0.0, 0.5)),
        (0.5, Vec3::new(0.0, 0.5, 0.0)),
        (1.0, Vec3::new(1.0, 1.0, 1.0)),
    ],
});

graph.set_output(remap);

// Generate 512x512 texture
let params = TextureGenerationParams {
    width: 512,
    height: 512,
    seed: 42,
};

let texture = manager.generate_texture(&graph, params)?;
```

## Performance

- 5-10ms for 512x512 textures on GPU
- ~50-100ms for CPU generation
- LRU cache for instant retrieval of duplicates

## Dependencies

- `noise`: Noise algorithms
- `vulkano`: GPU compute
- `shaderc`: Shader compilation
- `rustc-hash`: Fast hash maps

## Usage

```toml
praxis_procedural = { path = "../praxis_procedural", version = "0.1.0" }
```
