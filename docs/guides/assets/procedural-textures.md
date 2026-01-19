# Procedural Texture System

The procedural texture system provides runtime texture generation using noise functions and programmable texture graphs. Textures are generated on the GPU using compute shaders for optimal performance, with automatic caching to avoid redundant regeneration.

## Overview

The system consists of four main components:

1. **Noise Functions**: Perlin, Simplex, and Worley noise implementations
2. **Texture Graphs**: Node-based system for composing operations
3. **GPU Generator**: Compiles graphs to compute shaders and executes on GPU
4. **Cache System**: Automatic caching with LRU eviction

## Quick Start

### Basic Noise Texture

```rust
use praxis_procedural::{TextureGraph, TextureNode, NoiseType, TextureGenerationParams};
use praxis_graphics::ProceduralTextureManager;

// Create a texture graph
let mut graph = TextureGraph::new();
let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});
graph.set_output(noise_id);

// Generate texture
let params = TextureGenerationParams {
    width: 512,
    height: 512,
    seed: 42,
};

let texture = manager.generate_texture(&graph, params)?;
```

### Using in Rendering

```rust
// Add to texture manager
render_context
    .texture_manager_mut()
    .add_texture("my_procedural", texture);

// Use in draw command
let draw_cmd = DrawCommand {
    mesh_id: "cube".to_string(),
    model: Mat4::IDENTITY,
    texture_name: Some("my_procedural".to_string()),
    material_properties: None,
};
```

## Noise Functions

### Perlin Noise

Classic gradient noise with smooth, organic patterns. Best for:
- Terrain heightmaps
- Cloud textures
- Wood grain
- Marble patterns

```rust
TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,        // Frequency of pattern
    octaves: 4,        // Number of layers
    persistence: 0.5,  // Amplitude decay per octave
    lacunarity: 2.0,   // Frequency multiplier per octave
}
```

### Simplex Noise

Improved Perlin with better isotropy and fewer directional artifacts. Best for:
- Natural-looking terrain
- Flowing patterns
- General-purpose noise

```rust
TextureNode::Noise {
    noise_type: NoiseType::Simplex,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
}
```

### Worley (Cellular) Noise

Distance-based cellular patterns. Best for:
- Stone textures
- Cellular structures
- Cracked earth
- Water caustics

```rust
TextureNode::Noise {
    noise_type: NoiseType::Worley,
    scale: 16.0,
    octaves: 1,
    persistence: 0.5,
    lacunarity: 2.0,
}
```

## Node Types

### Transform Node

Transforms texture coordinates (scale, rotate, translate):

```rust
TextureNode::Transform {
    input: noise_id,
    params: TransformParams {
        offset: Vec2::new(0.1, 0.2),
        rotation: 0.5,  // Radians
        scale: Vec2::new(2.0, 1.0),
    },
}
```

### Blend Node

Combines two textures using various blend modes:

```rust
TextureNode::Blend {
    input_a: noise1_id,
    input_b: noise2_id,
    mode: BlendMode::Add,  // Add, Multiply, Mix, Screen, Overlay, etc.
    factor: 0.5,           // For Mix mode
}
```

Available blend modes:
- `Add`: A + B
- `Multiply`: A × B
- `Min`: min(A, B)
- `Max`: max(A, B)
- `Mix`: lerp(A, B, factor)
- `Screen`: 1 - (1-A) × (1-B)
- `Overlay`: Photoshop-style overlay
- `Subtract`: A - B

### Color Ramp Node

Maps grayscale values to color gradients:

```rust
let ramp = ColorRamp::new(vec![
    ColorStop { position: 0.0, color: [0.0, 0.0, 0.0, 1.0] },
    ColorStop { position: 0.5, color: [1.0, 0.0, 0.0, 1.0] },
    ColorStop { position: 1.0, color: [1.0, 1.0, 1.0, 1.0] },
]);

TextureNode::ColorRamp {
    input: noise_id,
    ramp,
}
```

### Filter Nodes

Various image processing filters:

```rust
// Invert colors
TextureNode::Invert { input: noise_id }

// Clamp to range
TextureNode::Clamp { input: noise_id, min: 0.0, max: 1.0 }

// Power function (adjust contrast curve)
TextureNode::Power { input: noise_id, exponent: 2.0 }

// Binary threshold
TextureNode::Threshold { input: noise_id, threshold: 0.5 }

// Adjust contrast
TextureNode::Contrast { input: noise_id, amount: 0.5 }

// Adjust brightness
TextureNode::Brightness { input: noise_id, amount: 0.2 }
```

## Example Recipes

### Marble Texture

```rust
let mut graph = TextureGraph::new();

// Multi-octave noise for detail
let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 12.0,
    octaves: 6,
    persistence: 0.6,
    lacunarity: 2.0,
});

// Enhance contrast
let power_id = graph.add_node(TextureNode::Power {
    input: noise_id,
    exponent: 2.0,
});

// Apply marble colors
let ramp = ColorRamp::new(vec![
    ColorStop { position: 0.0, color: [0.2, 0.1, 0.05, 1.0] },  // Dark veins
    ColorStop { position: 0.3, color: [0.9, 0.8, 0.7, 1.0] },   // Light marble
    ColorStop { position: 0.7, color: [0.6, 0.5, 0.4, 1.0] },   // Medium tone
    ColorStop { position: 1.0, color: [0.3, 0.2, 0.15, 1.0] },  // Dark marble
]);

let ramp_id = graph.add_node(TextureNode::ColorRamp {
    input: power_id,
    ramp,
});

graph.set_output(ramp_id);
```

### Wood Grain

```rust
let mut graph = TextureGraph::new();

let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 20.0,
    octaves: 5,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Stretch vertically for grain lines
let transform_id = graph.add_node(TextureNode::Transform {
    input: noise_id,
    params: TransformParams {
        offset: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::new(1.0, 8.0),  // 8x vertical stretch
    },
});

// Wood colors
let ramp = ColorRamp::new(vec![
    ColorStop { position: 0.0, color: [0.3, 0.15, 0.05, 1.0] },
    ColorStop { position: 0.5, color: [0.5, 0.3, 0.1, 1.0] },
    ColorStop { position: 1.0, color: [0.2, 0.1, 0.03, 1.0] },
]);

let ramp_id = graph.add_node(TextureNode::ColorRamp {
    input: transform_id,
    ramp,
});

graph.set_output(ramp_id);
```

### Cloud Texture

```rust
let mut graph = TextureGraph::new();

// Large-scale base
let noise1_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 4.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Small-scale detail
let noise2_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Simplex,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Combine layers
let blend_id = graph.add_node(TextureNode::Blend {
    input_a: noise1_id,
    input_b: noise2_id,
    mode: BlendMode::Add,
    factor: 0.5,
});

// Soften appearance
let power_id = graph.add_node(TextureNode::Power {
    input: blend_id,
    exponent: 1.5,
});

// Cloud colors
let ramp = ColorRamp::new(vec![
    ColorStop { position: 0.0, color: [0.7, 0.8, 0.9, 1.0] },
    ColorStop { position: 1.0, color: [1.0, 1.0, 1.0, 1.0] },
]);

let ramp_id = graph.add_node(TextureNode::ColorRamp {
    input: power_id,
    ramp,
});

graph.set_output(ramp_id);
```

### Cellular Stone

```rust
let mut graph = TextureGraph::new();

let worley_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Worley,
    scale: 16.0,
    octaves: 1,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Invert for stone cells
let invert_id = graph.add_node(TextureNode::Invert {
    input: worley_id,
});

// Enhance contrast
let power_id = graph.add_node(TextureNode::Power {
    input: invert_id,
    exponent: 1.5,
});

// Stone colors
let ramp = ColorRamp::new(vec![
    ColorStop { position: 0.0, color: [0.3, 0.3, 0.3, 1.0] },
    ColorStop { position: 0.5, color: [0.6, 0.6, 0.6, 1.0] },
    ColorStop { position: 1.0, color: [0.4, 0.4, 0.4, 1.0] },
]);

let ramp_id = graph.add_node(TextureNode::ColorRamp {
    input: power_id,
    ramp,
});

graph.set_output(ramp_id);
```

## Performance

### GPU Compute

- Textures generated entirely on GPU
- Typical generation time: <10ms for 512×512
- Scales well with texture size due to parallelism

### Caching

- Identical graphs + parameters = instant cache hit
- Default limits: 1000 textures, 512 MB memory
- LRU eviction when limits exceeded
- Cache statistics available for monitoring

### Best Practices

1. **Reuse Graphs**: Create graphs once, generate multiple times with different seeds
2. **Cache Warming**: Pre-generate common textures at startup
3. **Size Selection**: 512×512 is a good balance for most textures
4. **Monitor Cache**: Check cache statistics to tune limits

## Advanced Topics

### Graph Validation

Always validate graphs before generation:

```rust
if let Err(e) = graph.validate() {
    eprintln!("Invalid graph: {}", e);
}
```

Validation checks:
- Output node exists
- All inputs reference valid nodes
- No cycles in graph (DAG structure)

### Random Seeds

Use different seeds for variation:

```rust
// Same graph, different results
let params1 = TextureGenerationParams { width: 512, height: 512, seed: 0 };
let params2 = TextureGenerationParams { width: 512, height: 512, seed: 1 };

let texture1 = manager.generate_texture(&graph, params1)?;
let texture2 = manager.generate_texture(&graph, params2)?;
```

### Cache Management

```rust
// Get statistics
let stats = manager.cache_statistics();
println!("Hit rate: {:.1}%", stats.hit_rate());
println!("Memory: {} KB", stats.memory_used / 1024);

// Clear cache
manager.clear_cache();

// Generate without caching
let texture = manager.generate_texture_uncached(&graph, params)?;
```

## See Also

- [Procedural Textures API Reference](../../reference/procedural-textures-api.md) - API documentation
- [praxis_procedural Crate](../../../crates/praxis_procedural/README.md) - Crate documentation
- [Rendering Guide](../rendering.md) - Graphics system overview
- [praxis_graphics Crate](../../../crates/praxis_graphics/README.md) - Texture management documentation
