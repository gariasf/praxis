# Procedural Textures API Reference

API reference for GPU-accelerated procedural texture generation.

## Core Types

### TextureGraph

Node-based graph for texture generation.

```rust
pub struct TextureGraph { /* ... */ }
```

**Methods:**
- `new()` - Create empty graph
- `add_node(node: TextureNode) -> NodeId` - Add node, returns ID
- `connect(from: NodeId, to: NodeId, input: usize) -> Result<()>`
- `disconnect(to: NodeId, input: usize)`
- `remove_node(id: NodeId)`
- `set_output(id: NodeId)` - Set as final output
- `clear()` - Remove all nodes
- `node_count() -> usize`

### TextureNode

Individual processing node in graph.

```rust
pub enum TextureNode {
    Noise {
        noise_type: NoiseType,
        scale: f32,
        octaves: u32,
        persistence: f32,
        lacunarity: f32,
    },
    Transform {
        scale: (f32, f32),
        rotation: f32,
        translation: (f32, f32),
    },
    Blend {
        mode: BlendMode,
        factor: f32,
    },
    ColorRamp {
        gradient: Vec<(f32, [f32; 4])>,
    },
    Filter {
        filter_type: FilterType,
    },
}
```

### NoiseType

Noise function algorithms.

```rust
pub enum NoiseType {
    Perlin,    // Classic Perlin noise
    Simplex,   // Simplex noise (faster, fewer artifacts)
    Worley,    // Cellular/Voronoi noise
}
```

### BlendMode

Blend operation for combining textures.

```rust
pub enum BlendMode {
    Add,        // A + B
    Multiply,   // A × B
    Mix,        // lerp(A, B, factor)
    Screen,     // 1 - (1-A)(1-B)
    Overlay,    // Overlay blend
    Subtract,   // A - B
}
```

### FilterType

Post-processing filters.

```rust
pub enum FilterType {
    Invert,
    Clamp { min: f32, max: f32 },
    Power { exponent: f32 },
    Threshold { value: f32 },
    Contrast { amount: f32 },
    Brightness { amount: f32 },
}
```

### TextureGenerationParams

Parameters for texture generation.

```rust
pub struct TextureGenerationParams {
    pub width: u32,
    pub height: u32,
    pub seed: u32,
}
```

**Methods:**
- `new(width: u32, height: u32)` - Random seed
- `with_seed(width: u32, height: u32, seed: u32)`

## Manager

### ProceduralTextureManager

Manages texture generation and caching.

```rust
pub struct ProceduralTextureManager { /* ... */ }
```

**Methods:**
- `new(device, queue, allocator) -> Self`
- `generate_texture(graph: &TextureGraph, params: TextureGenerationParams) -> Result<Arc<ImageView>>`
- `clear_cache()` - Remove all cached textures
- `cache_size() -> usize` - Number of cached textures
- `set_cache_limit(max_entries: usize)`

## Common Patterns

### Basic Noise Texture

```rust
use praxis_procedural::{TextureGraph, TextureNode, NoiseType, TextureGenerationParams};

let mut graph = TextureGraph::new();

let noise = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

graph.set_output(noise);

let params = TextureGenerationParams::new(512, 512);
let texture = manager.generate_texture(&graph, params)?;
```

### Layered Noise

```rust
// Create base layer
let base = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 4.0,
    octaves: 3,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Create detail layer
let detail = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Simplex,
    scale: 16.0,
    octaves: 2,
    persistence: 0.3,
    lacunarity: 2.5,
});

// Blend layers
let blend = graph.add_node(TextureNode::Blend {
    mode: BlendMode::Add,
    factor: 0.5,
});

graph.connect(base, blend, 0)?;
graph.connect(detail, blend, 1)?;
graph.set_output(blend);
```

### Color Mapped Noise

```rust
// Generate grayscale noise
let noise = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Apply color gradient
let ramp = graph.add_node(TextureNode::ColorRamp {
    gradient: vec![
        (0.0, [0.0, 0.0, 0.5, 1.0]),  // Dark blue
        (0.3, [0.0, 0.5, 1.0, 1.0]),  // Light blue
        (0.5, [0.8, 0.8, 0.3, 1.0]),  // Sandy
        (0.7, [0.2, 0.6, 0.2, 1.0]),  // Green
        (1.0, [1.0, 1.0, 1.0, 1.0]),  // White (peaks)
    ],
});

graph.connect(noise, ramp, 0)?;
graph.set_output(ramp);
```

### Transformed Noise

```rust
let noise = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Worley,
    scale: 10.0,
    octaves: 1,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Apply transformation
let transform = graph.add_node(TextureNode::Transform {
    scale: (2.0, 1.0),     // Stretch horizontally
    rotation: 45.0,        // Rotate 45 degrees
    translation: (0.5, 0.0),
});

graph.connect(noise, transform, 0)?;
graph.set_output(transform);
```

### Filtered Result

```rust
let noise = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Apply contrast
let contrast = graph.add_node(TextureNode::Filter {
    filter_type: FilterType::Contrast { amount: 1.5 },
});

// Apply threshold
let threshold = graph.add_node(TextureNode::Filter {
    filter_type: FilterType::Threshold { value: 0.5 },
});

graph.connect(noise, contrast, 0)?;
graph.connect(contrast, threshold, 0)?;
graph.set_output(threshold);
```

## Performance Tips

### Caching

The manager automatically caches generated textures. Same graph + parameters = instant retrieval.

```rust
// First call: generates texture (~5-10ms)
let tex1 = manager.generate_texture(&graph, params)?;

// Second call: cache hit (~0ms)
let tex2 = manager.generate_texture(&graph, params)?;
```

### Cache Management

```rust
// Limit cache size
manager.set_cache_limit(100);

// Clear cache when needed
manager.clear_cache();

// Monitor cache
let size = manager.cache_size();
```

### Generation Time

Typical generation times for 512×512 textures:
- Simple noise: ~3-5ms
- Layered (2-3 nodes): ~5-8ms
- Complex (5+ nodes): ~8-15ms

### Optimization Strategies

1. **Reuse graphs** - Create once, generate many times
2. **Smaller dimensions** - Use 256×256 for distant objects
3. **Seed variation** - Change seed instead of recreating graphs
4. **LOD textures** - Generate multiple resolutions

## Integration

### With Material System

```rust
// Generate texture
let texture = manager.generate_texture(&graph, params)?;

// Use in material
let material = Material {
    albedo_texture: Some(texture),
    // ...
};
```

### Runtime Generation

```rust
fn generate_textures_system(
    mut manager: ResMut<ProceduralTextureManager>,
    query: Query<&TextureGraph, Added<TextureGraph>>,
) {
    for graph in &query {
        let params = TextureGenerationParams::new(512, 512);
        let texture = manager.generate_texture(graph, params).unwrap();
        // Assign to entity
    }
}
```

## Noise Parameters Guide

### Perlin Noise

**Best for:** Smooth, natural terrain, clouds
- `scale`: 4.0-16.0 for terrain, 32.0-64.0 for clouds
- `octaves`: 4-6 typical, more for detail
- `persistence`: 0.5 standard, lower for smoother
- `lacunarity`: 2.0 standard

### Simplex Noise

**Best for:** Fast generation, organic patterns
- Similar parameters to Perlin
- Faster, fewer directional artifacts
- Good for real-time generation

### Worley Noise

**Best for:** Cellular patterns, stone, cracks
- `scale`: 5.0-20.0 for cell size
- `octaves`: Usually 1-2
- Creates distinctive cell/voronoi patterns

## See Also

- [Procedural Textures Guide](../guides/assets/procedural-textures.md) - Comprehensive usage guide
- [Assets Guides](../guides/assets/README.md) - Asset loading and management
- [praxis_procedural Crate](../../crates/praxis_procedural/README.md) - Crate documentation
