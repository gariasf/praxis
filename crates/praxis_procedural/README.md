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
use praxis_procedural::{
    TextureGraph, TextureNode, NoiseType,
    TextureGenerationParams, ProceduralTextureManager
};
use color_eyre::Result;

fn generate_procedural_texture(
    manager: &mut ProceduralTextureManager
) -> Result<()> {
    // Create a texture graph
    let mut graph = TextureGraph::new();
    
    // Add a Perlin noise node
    let noise_id = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Perlin,
        scale: 8.0,        // Noise frequency
        octaves: 4,        // Detail levels
        persistence: 0.5,  // Amplitude reduction per octave
        lacunarity: 2.0,   // Frequency increase per octave
    });
    
    // Set as output
    graph.set_output(noise_id);
    
    // Generate texture on GPU
    let params = TextureGenerationParams {
        width: 512,
        height: 512,
        seed: 42,
    };
    
    let texture = manager.generate_texture(&graph, params)?;
    
    // Texture is now cached and ready to use
    Ok(())
}
```

## Node Types

```rust
use praxis_procedural::{TextureNode, NoiseType, BlendMode};

// Noise generation
let noise = TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
};

// Coordinate transformation
let transform = TextureNode::Transform {
    scale: 2.0,
    rotate: 45.0,  // degrees
    translate: (0.5, 0.5),
};

// Blend multiple textures
let blend = TextureNode::Blend {
    input_a: node_id_1,
    input_b: node_id_2,
    mode: BlendMode::Multiply,
    factor: 0.5,
};

// Color gradient
let color_ramp = TextureNode::ColorRamp {
    input: node_id,
    gradient: vec![
        (0.0, [0.0, 0.0, 0.0, 1.0]), // Black at 0
        (0.5, [1.0, 0.0, 0.0, 1.0]), // Red at 0.5
        (1.0, [1.0, 1.0, 0.0, 1.0]), // Yellow at 1.0
    ],
};

// Image filters
let filter = TextureNode::Filter {
    input: node_id,
    filter_type: FilterType::Contrast { amount: 1.5 },
};
```

## Complete Example

```rust
use praxis_procedural::{
    TextureGraph, TextureNode, NoiseType, BlendMode,
    TextureGenerationParams, ProceduralTextureManager
};
use color_eyre::Result;

fn create_complex_texture(
    manager: &mut ProceduralTextureManager
) -> Result<()> {
    let mut graph = TextureGraph::new();
    
    // Base layer: Large-scale Perlin noise
    let base_noise = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Perlin,
        scale: 4.0,
        octaves: 3,
        persistence: 0.5,
        lacunarity: 2.0,
    });
    
    // Detail layer: Small-scale Simplex noise
    let detail_noise = graph.add_node(TextureNode::Noise {
        noise_type: NoiseType::Simplex,
        scale: 16.0,
        octaves: 2,
        persistence: 0.4,
        lacunarity: 2.5,
    });
    
    // Blend the two noise layers
    let blended = graph.add_node(TextureNode::Blend {
        input_a: base_noise,
        input_b: detail_noise,
        mode: BlendMode::Add,
        factor: 0.3,
    });
    
    // Apply color gradient
    let colored = graph.add_node(TextureNode::ColorRamp {
        input: blended,
        gradient: vec![
            (0.0, [0.1, 0.1, 0.3, 1.0]),  // Dark blue
            (0.5, [0.3, 0.6, 0.9, 1.0]),  // Light blue
            (1.0, [1.0, 1.0, 1.0, 1.0]),  // White
        ],
    });
    
    // Set as final output
    graph.set_output(colored);
    
    // Generate
    let params = TextureGenerationParams {
        width: 512,
        height: 512,
        seed: 123,
    };
    
    let texture = manager.generate_texture(&graph, params)?;
    
    Ok(())
}
```

## Documentation

**Comprehensive Guide:**
- [Procedural Textures Guide](../../docs/guides/assets/procedural-textures.md) - Complete usage and patterns

## Examples

```bash
# Run procedural texture demo
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
