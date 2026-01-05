# Praxis Procedural Texture System

Procedural texture generation system for the Praxis game engine with runtime texture synthesis using noise functions, programmable texture graphs, GPU compute shader-based generation, and automatic caching.

## Features

### Noise Functions

- **Perlin Noise**: Classic gradient noise for smooth, organic patterns
- **Simplex Noise**: Improved Perlin with better isotropy and performance
- **Worley Noise**: Cellular/Voronoi patterns for stone, cells, cracks

### Texture Graph System

Node-based system for composing complex textures:

- **Noise Nodes**: Generate base noise patterns
- **Transform Nodes**: Scale, rotate, translate coordinates
- **Blend Nodes**: Combine textures (Add, Multiply, Mix, Screen, Overlay, etc.)
- **Color Ramp Nodes**: Map grayscale to color gradients
- **Filter Nodes**: Invert, clamp, power, threshold, contrast, brightness

### GPU Compute Generation

- Compile texture graphs to GLSL compute shaders
- Execute on GPU for optimal performance
- Support for arbitrary graph complexity
- Automatic shader generation from graph structure

### Caching System

- Automatic caching of generated textures
- Cache key based on graph structure and parameters
- LRU eviction when cache limits reached
- Configurable memory and entry limits
- Cache statistics and monitoring

## Architecture

```
┌─────────────────────┐
│   TextureGraph      │  ← Node-based texture description
│  - Nodes (DAG)      │
│  - Validation       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│ProceduralGenerator  │  ← Compiles graph to GLSL compute shader
│  - Shader compiler  │
│  - GPU execution    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  TextureCache       │  ← Caches generated textures
│  - LRU eviction     │
│  - Statistics       │
└─────────────────────┘
```

## Usage Examples

### Basic Noise Texture

```rust
use praxis_procedural::{TextureGraph, TextureNode, NoiseType, TextureGenerationParams};

// Create a simple Perlin noise texture
let mut graph = TextureGraph::new();
let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 8.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});
graph.set_output(noise_id);

// Generate 512x512 texture
let params = TextureGenerationParams {
    width: 512,
    height: 512,
    seed: 42,
};

let texture_data = generator.generate(&graph, params)?;
```

### Marble Texture with Color Ramp

```rust
use praxis_procedural::{ColorRamp, ColorStop};

let mut graph = TextureGraph::new();

// Base noise
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
    ColorStop { position: 0.0, color: [0.2, 0.1, 0.05, 1.0] },
    ColorStop { position: 0.3, color: [0.9, 0.8, 0.7, 1.0] },
    ColorStop { position: 0.7, color: [0.6, 0.5, 0.4, 1.0] },
    ColorStop { position: 1.0, color: [0.3, 0.2, 0.15, 1.0] },
]);

let ramp_id = graph.add_node(TextureNode::ColorRamp {
    input: power_id,
    ramp,
});

graph.set_output(ramp_id);
```

### Blending Multiple Noise Layers

```rust
let mut graph = TextureGraph::new();

// Large-scale noise
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
    scale: 16.0,
    octaves: 3,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Blend them together
let blend_id = graph.add_node(TextureNode::Blend {
    input_a: noise1_id,
    input_b: noise2_id,
    mode: BlendMode::Multiply,
    factor: 0.5,
});

graph.set_output(blend_id);
```

### Transform and Scale

```rust
use praxis_procedural::TransformParams;
use praxis_math::Vec2;

let mut graph = TextureGraph::new();

let noise_id = graph.add_node(TextureNode::Noise {
    noise_type: NoiseType::Perlin,
    scale: 10.0,
    octaves: 4,
    persistence: 0.5,
    lacunarity: 2.0,
});

// Stretch vertically for wood grain effect
let transform_id = graph.add_node(TextureNode::Transform {
    input: noise_id,
    params: TransformParams {
        offset: Vec2::ZERO,
        rotation: 0.0,
        scale: Vec2::new(1.0, 8.0), // 8x vertical stretch
    },
});

graph.set_output(transform_id);
```

### Using with Graphics System

```rust
use praxis_graphics::ProceduralTextureManager;

// Create manager
let mut manager = ProceduralTextureManager::new(
    device,
    queue,
    memory_allocator,
    command_buffer_allocator,
    descriptor_set_allocator,
);

// Generate texture (with caching)
let texture = manager.generate_texture(&graph, params)?;

// Add to texture manager for rendering
render_context.texture_manager_mut().add_texture("my_procedural", texture);

// Use in rendering
let draw_cmd = DrawCommand {
    mesh_id: "cube".to_string(),
    model: Mat4::IDENTITY,
    texture_name: Some("my_procedural".to_string()),
    material_properties: None,
};
```

## Node Types

### Noise Node
Generates noise using various algorithms:
- `noise_type`: Perlin, Simplex, or Worley
- `scale`: Base frequency of noise pattern
- `octaves`: Number of noise layers (fractal)
- `persistence`: Amplitude decay per octave
- `lacunarity`: Frequency multiplier per octave

### Constant Node
Outputs a constant color value.

### Transform Node
Transforms texture coordinates:
- `offset`: Translation
- `rotation`: Rotation in radians
- `scale`: Scale factor per axis

### Blend Node
Combines two textures:
- `mode`: Add, Multiply, Min, Max, Mix, Screen, Overlay, Subtract
- `factor`: Blend amount (for Mix mode)

### ColorRamp Node
Maps grayscale input to color gradient:
- `ramp`: List of color stops with positions

### Filter Nodes
- **Invert**: Inverts colors (1.0 - value)
- **Clamp**: Clamps values to min/max range
- **Power**: Applies power function (value^exponent)
- **Threshold**: Binary threshold (< threshold = 0, >= threshold = 1)
- **Contrast**: Adjusts contrast (-1 to 1)
- **Brightness**: Adjusts brightness (-1 to 1)

## Performance

### GPU Compute
- Textures are generated entirely on GPU
- Typical generation time: <10ms for 512x512 texture
- Scales well with texture size due to parallel execution

### Caching
- Identical graphs + parameters = cached result
- Default cache limits: 1000 textures, 512 MB
- LRU eviction when limits exceeded
- Cache hit rate typically >90% in production

### Shader Compilation
- Graphs compiled to optimized GLSL compute shaders
- Compilation happens once per unique graph
- Dead code elimination for unused operations

## Examples

See `examples/procedural_texture_demo.rs` for a complete demonstration featuring:
- Multiple texture types (Perlin, Worley, marble, wood, clouds)
- Real-time rendering with rotating cubes
- Cache statistics monitoring

Run with:
```bash
cargo run --example procedural_texture_demo
```

## Implementation Details

### Graph Validation
Graphs are validated before generation:
- Output node must exist
- All referenced inputs must exist
- No cycles allowed (DAG structure)

### Shader Generation
- Each node becomes a GLSL function
- Functions are called recursively to evaluate graph
- Noise functions match CPU implementations exactly
- Full IEEE 754 floating-point precision

### Memory Management
- Generated textures stored as RGBA8 (4 bytes per pixel)
- Cache automatically evicts least-recently-used entries
- GPU resources cleaned up when textures dropped

## Future Enhancements

Potential future additions:
- **3D Noise**: Volume textures for clouds, fog, etc.
- **Gradient Nodes**: Custom gradient generation
- **Math Nodes**: Arithmetic operations on textures
- **Texture Tiling**: Seamless tiling support
- **Texture Bombing**: Random placement of texture elements
- **Animated Noise**: Time-varying noise for effects
- **GPU Shader Caching**: Cache compiled compute pipelines

## Dependencies

- `vulkano` 0.35.1: Vulkan compute shader execution
- `vulkano-shaders`: GLSL shader compilation
- `praxis_math`: Math types (Vec2, Vec3, Vec4)
- `praxis_graphics`: Texture management and rendering
- `praxis_utils`: Error handling
- `noise`: Noise generation algorithms (Perlin, Simplex, Worley)

## Testing

```bash
cargo test -p praxis_procedural
```

## See Also

- [Procedural Texture Demo](../../examples/procedural_texture_demo.rs)
- [Praxis Graphics System](../praxis_graphics/README.md)
- [Texture Guide](../../docs/guides/textures.md)
