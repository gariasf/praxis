# praxis_terrain

Terrain generation and rendering for Praxis engine.

## Overview

Scalable terrain system with procedural generation, LOD, and chunking.

## Features

### Terrain Generation

- **Height Maps**: 2D noise-based elevation
- **Procedural**: Runtime generation using noise
- **Chunking**: Divide terrain into manageable pieces
- **Biomes**: Multi-layered terrain with different characteristics

### Level of Detail (LOD)

- Distance-based LOD selection
- Seamless transitions between LOD levels
- Configurable LOD distances
- Reduces vertex count for distant terrain

### Rendering

- Efficient vertex buffer management
- Normal map generation
- Multi-texturing with splatmaps
- GPU-based tessellation (optional)

### Features

- **Streaming**: Load/unload chunks based on camera
- **Collision**: Generate physics colliders from heightmap
- **Editing**: Runtime height modification (optional)

## Example

```rust
use praxis_terrain::{TerrainConfig, TerrainGenerator, TerrainRenderer};

// Configure terrain
let config = TerrainConfig {
    chunk_size: 64,
    max_height: 100.0,
    lod_levels: 4,
    noise_scale: 0.01,
    octaves: 6,
};

// Generate terrain
let mut generator = TerrainGenerator::new(config);
let terrain = generator.generate_chunk(chunk_x, chunk_z);

// Render
renderer.render_terrain(&terrain, &camera);
```

## Noise-Based Generation

```rust
use praxis_terrain::{TerrainNoise, NoiseLayer};

let mut noise = TerrainNoise::new(seed);

// Base terrain
noise.add_layer(NoiseLayer {
    frequency: 0.01,
    amplitude: 50.0,
    octaves: 4,
});

// Add detail
noise.add_layer(NoiseLayer {
    frequency: 0.05,
    amplitude: 10.0,
    octaves: 2,
});

let height = noise.sample(x, z);
```

## LOD System

```rust
let lod = match distance {
    d if d < 50.0 => 0,   // Full detail
    d if d < 100.0 => 1,  // Half detail
    d if d < 200.0 => 2,  // Quarter detail
    _ => 3,               // Minimal detail
};
```

## Dependencies

- `noise`: Procedural generation
- `serde`: Serialization
- `rustc-hash`: Fast hash maps

## Usage

```toml
# In root Cargo.toml
[features]
terrain = ["praxis_terrain"]

# In your crate
praxis_terrain = { path = "../praxis_terrain", version = "0.1.0" }
```
