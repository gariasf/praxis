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
use praxis_terrain::{
    TerrainConfig, TerrainHeightmap, TerrainSystem
};
use praxis_math::Vec3;
use color_eyre::Result;

fn create_terrain(
    device: Arc<Device>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    command_buffer_allocator: Arc<StandardCommandBufferAllocator>
) -> Result<TerrainSystem> {
    // Generate heightmap from noise
    let heightmap = TerrainHeightmap::from_noise(
        512,   // Width
        512,   // Height
        100.0, // Max height
        4.0,   // Frequency
        6      // Octaves
    );
    
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
    
    // Create terrain system
    let mut terrain = TerrainSystem::new(config, heightmap)?;
    
    // Initialize rendering resources
    terrain.initialize_rendering(
        device,
        memory_allocator,
        command_buffer_allocator
    );
    
    Ok(terrain)
}

fn update_terrain_each_frame(
    terrain: &mut TerrainSystem,
    camera_position: Vec3
) -> Result<()> {
    // Update LOD based on camera position
    terrain.update(camera_position)?;
    
    Ok(())
}
```

## Material Layers

```rust
use praxis_terrain::TerrainMaterialLayer;
use color_eyre::Result;

fn setup_terrain_materials(
    terrain: &mut TerrainSystem
) -> Result<()> {
    // Grass at low elevations
    terrain.material.add_layer(
        TerrainMaterialLayer::new(
            "grass",
            "grass_albedo",  // Albedo texture
            0.0,             // Min height
            30.0             // Max height
        )
        .with_normal("grass_normal")
        .with_tiling(10.0)
    );
    
    // Dirt at mid elevations
    terrain.material.add_layer(
        TerrainMaterialLayer::new(
            "dirt",
            "dirt_albedo",
            25.0,
            60.0
        )
        .with_normal("dirt_normal")
        .with_tiling(12.0)
    );
    
    // Rock on steep slopes and high elevations
    terrain.material.add_layer(
        TerrainMaterialLayer::new(
            "rock",
            "rock_albedo",
            30.0,
            100.0  // Max height (terrain max)
        )
        .with_slope(20.0, 90.0)  // 20-90 degree slopes
        .with_normal("rock_normal")
        .with_tiling(15.0)
    );
    
    // Snow at peaks
    terrain.material.add_layer(
        TerrainMaterialLayer::new(
            "snow",
            "snow_albedo",
            70.0,  // Only at high elevations
            100.0
        )
        .with_slope(0.0, 45.0)  // Only on gentle slopes
        .with_normal("snow_normal")
        .with_tiling(8.0)
    );
    
    Ok(())
}
```

## Vegetation

```rust
use praxis_terrain::VegetationLayer;
use color_eyre::Result;

fn add_vegetation(terrain: &mut TerrainSystem) -> Result<()> {
    // Grass
    terrain.vegetation_layers.push(
        VegetationLayer::new(
            "grass",
            "grass_mesh",     // Mesh name
            "grass_material", // Material name
            5.0               // Density (instances per unit²)
        )
        .with_height_range(0.0, 40.0)    // Only below 40 units
        .with_slope_range(0.0, 30.0)     // Only on gentle slopes
        .with_scale_variance(0.8, 1.2)   // Random size variation
        .with_wind_strength(1.5)
    );
    
    // Trees
    terrain.vegetation_layers.push(
        VegetationLayer::new(
            "trees",
            "tree_mesh",
            "tree_material",
            0.5  // Lower density for larger objects
        )
        .with_height_range(10.0, 50.0)
        .with_slope_range(0.0, 20.0)     // Only on flat ground
        .with_scale_variance(0.7, 1.5)
    );
    
    // Rocks
    terrain.vegetation_layers.push(
        VegetationLayer::new(
            "rocks",
            "rock_mesh",
            "rock_material",
            1.0
        )
        .with_height_range(30.0, 80.0)
        .with_slope_range(15.0, 60.0)    // Prefer slopes
        .with_scale_variance(0.5, 2.0)
    );
    
    // Generate all vegetation instances
    terrain.generate_vegetation()?;
    
    Ok(())
}
```

## Heightmap Loading

```rust
use praxis_terrain::TerrainHeightmap;
use color_eyre::Result;

fn load_heightmap_from_image() -> Result<TerrainHeightmap> {
    // Load from image file (grayscale values = height)
    let heightmap = TerrainHeightmap::from_image(
        "assets/terrain/heightmap.png",
        100.0  // Max height
    )?;
    
    Ok(heightmap)
}

fn create_custom_heightmap() -> TerrainHeightmap {
    let width = 256;
    let height = 256;
    let mut data = vec![0.0; width * height];
    
    // Create custom height data (e.g., a cone)
    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - width as f32 / 2.0;
            let dy = y as f32 - height as f32 / 2.0;
            let distance = (dx * dx + dy * dy).sqrt();
            let max_dist = (width as f32 / 2.0);
            
            // Cone shape: highest at center, zero at edges
            data[y * width + x] = (max_dist - distance).max(0.0) / max_dist;
        }
    }
    
    TerrainHeightmap::from_data(width, height, data, 100.0)
}
```

## Documentation

**Comprehensive Guide:**
- [Terrain Guide](../../docs/guides/terrain.md) - Complete terrain system guide

**Reference:**
- [Terrain API Reference](../../docs/reference/terrain-api.md)

## Examples

```bash
# Full terrain demo with LOD and vegetation (use --release for better performance)
cargo run --example terrain_demo --release
```

## Performance

- **Memory:** ~4 bytes/heightmap sample, 2-8 KB/chunk/LOD
- **Rendering:** ~75% triangle reduction with LOD system
- **Vegetation:** O(layers) instancing cost, not O(instances)
  - 1 million grass instances = ~1-2ms GPU time
  - Uses GPU instancing, minimal CPU overhead

## Dependencies

- `bevy_ecs` 0.14: ECS integration
- `rayon`: Parallel chunk processing
- `noise`: Procedural heightmap generation
- `vulkano`: Rendering
