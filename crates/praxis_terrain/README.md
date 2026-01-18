# Praxis Terrain

Heightmap-based terrain with chunked LOD, texture splatting, and vegetation for the Praxis game engine.

## Overview

Comprehensive terrain rendering with multi-layer materials, GPU instanced vegetation, and real-time editing.

**Key Features:**
- Heightmap terrain (image, noise, custom data)
- Chunked LOD system with seamless transitions
- Multi-layer texture splatting (up to 8 layers)
- GPU vegetation instancing (millions of instances)
- Terrain editing tools (sculpt, paint, vegetation)
- Parallel chunk generation with Rayon

## Quick Start

```rust
use praxis_terrain::{TerrainConfig, TerrainHeightmap, TerrainSystem};

// Create heightmap
let heightmap = TerrainHeightmap::from_noise(512, 512, 100.0, 4.0, 6);

// Configure terrain
let config = TerrainConfig {
    chunk_size: 64.0,
    vertices_per_chunk: 65,
    max_height: 100.0,
    lod_levels: 4,
    lod_distances: vec![50.0, 100.0, 200.0, 400.0],
    world_size: 1024.0,
    ..Default::default()
};

// Create terrain
let mut terrain = TerrainSystem::new(config, heightmap)?;
terrain.initialize_rendering(device, memory_allocator, command_buffer_allocator);

// Update each frame
terrain.update(camera_position)?;
```

## Material Layers

```rust
use praxis_terrain::TerrainMaterialLayer;

// Grass at low elevations
terrain.material.add_layer(
    TerrainMaterialLayer::new("grass", "grass_albedo", 0.0, 30.0)
        .with_normal("grass_normal")
        .with_tiling(10.0)
);

// Rock on steep slopes
terrain.material.add_layer(
    TerrainMaterialLayer::new("rock", "rock_albedo", 30.0, 70.0)
        .with_slope(20.0, 90.0)
        .with_tiling(15.0)
);
```

## Vegetation

```rust
use praxis_terrain::VegetationLayer;

terrain.vegetation_layers.push(
    VegetationLayer::new("grass", "grass_mesh", "grass_mat", 5.0)
        .with_height_range(0.0, 40.0)
        .with_slope_range(0.0, 30.0)
        .with_wind_strength(1.5)
);

terrain.generate_vegetation()?;
```

## Documentation

**Comprehensive Guide:**
- [Terrain Guide](../../docs/guides/terrain.md) - Complete terrain system guide

## Examples

```bash
cargo run --example terrain_demo --release
```

## Performance

- **Memory:** ~4 bytes/heightmap sample, 2-8 KB/chunk/LOD
- **Rendering:** ~75% triangle reduction with LOD
- **Vegetation:** O(layers) instancing, not O(instances)

## Dependencies

- `bevy_ecs` 0.14: ECS integration
- `rayon`: Parallel processing
- `noise`: Procedural generation
- `vulkano`: Rendering
