# Terrain System

The Praxis terrain system provides comprehensive tools for rendering large-scale outdoor environments with heightmap-based terrain, chunked LOD, texture splatting, and GPU-instanced vegetation.

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Getting Started](#getting-started)
- [Heightmap System](#heightmap-system)
- [Chunked LOD](#chunked-lod)
- [Texture Splatting](#texture-splatting)
- [Vegetation Instancing](#vegetation-instancing)
- [Terrain Editing](#terrain-editing)
- [Performance Optimization](#performance-optimization)
- [Advanced Topics](#advanced-topics)

## Overview

The terrain system is designed to handle large outdoor environments efficiently by:

- **Streaming**: Loading and unloading terrain chunks based on camera position
- **LOD**: Multiple levels of detail to balance quality and performance
- **Instancing**: GPU instancing for rendering millions of vegetation objects
- **Parallel Processing**: Using Rayon for multi-threaded terrain generation
- **Material Blending**: Up to 8 texture layers blended using splat maps

### Key Features

- Heightmap-based terrain with bilinear interpolation
- Chunked LOD system with seamless transitions
- Texture splatting with normal mapping
- GPU-instanced vegetation with wind animation
- Real-time terrain editing tools
- Frustum culling for chunks and vegetation
- Parallel mesh generation and vegetation distribution

## Architecture

```
TerrainSystem (High-level coordinator)
├── TerrainHeightmap (Elevation data)
│   ├── 2D grid of height values
│   ├── Bilinear interpolation
│   └── Normal calculation
│
├── TerrainChunk[] (Spatial partitioning)
│   ├── ChunkLod (LOD state)
│   ├── GpuMesh[4] (One per LOD level)
│   └── Bounding box
│
├── TerrainMaterial (Material configuration)
│   └── TerrainMaterialLayer[] (Up to 8 layers)
│       ├── Texture references
│       ├── Height range
│       ├── Slope range
│       └── Tiling factor
│
├── SplatMap (Material blend weights)
│   └── RGBA control map
│
├── VegetationLayer[] (Vegetation configuration)
│   └── VegetationInstance[] (Instance transforms)
│       ├── Position
│       ├── Rotation
│       ├── Scale
│       └── Color variation
│
├── TerrainRenderer (Rendering system)
│   └── Specialized shaders for terrain
│
└── VegetationRenderer (Instancing system)
    └── Per-instance data buffers
```

## Getting Started

### Basic Setup

```rust
use praxis_terrain::{TerrainConfig, TerrainHeightmap, TerrainSystem};
use praxis_math::Vec3;

// Create heightmap from procedural noise
let heightmap = TerrainHeightmap::from_noise(
    512,    // width
    512,    // height
    100.0,  // max_height
    4.0,    // scale
    6       // octaves
);

// Configure terrain
let config = TerrainConfig {
    chunk_size: 64.0,
    vertices_per_chunk: 65,
    max_height: 100.0,
    lod_levels: 4,
    lod_distances: vec![50.0, 100.0, 200.0, 400.0],
    world_size: 1024.0,
    world_scale: 1.0,
    enable_frustum_culling: true,
    enable_occlusion_culling: false,
};

// Create terrain system
let mut terrain = TerrainSystem::new(config, heightmap)?;

// Initialize rendering (requires Vulkan device)
terrain.initialize_rendering(
    device,
    memory_allocator,
    command_buffer_allocator
);
```

### Frame Update

```rust
// Call every frame to update LOD and load/unload chunks
let camera_pos = Vec3::new(player_x, player_y, player_z);
terrain.update(camera_pos);
```

## Heightmap System

The heightmap stores elevation data in a 2D grid with floating-point precision.

### Loading from Image

```rust
// Load 8-bit grayscale image (PNG, JPEG)
let heightmap = TerrainHeightmap::from_file("terrain.png", 100.0)?;

// Black (0) = low elevation
// White (255) = max elevation (100.0m in this case)
```

### Procedural Generation

```rust
// Generate using Perlin noise
let heightmap = TerrainHeightmap::from_noise(
    512,    // width in samples
    512,    // height in samples
    100.0,  // maximum height
    4.0,    // noise scale (larger = more features)
    6       // octaves (more = more detail)
);
```

### Custom Data

```rust
// Create from raw height data
let width = 512;
let height = 512;
let mut heights = vec![0.0; (width * height) as usize];

// Fill with custom data
for y in 0..height {
    for x in 0..width {
        let idx = (y * width + x) as usize;
        heights[idx] = calculate_height(x, y);
    }
}

let heightmap = TerrainHeightmap::from_heights(
    width,
    height,
    heights,
    100.0  // max_height
);
```

### Querying Heights

```rust
// Get height at grid coordinates
let height = heightmap.get_height(256, 256);

// Get interpolated height at world position
let height = heightmap.get_height_at(
    world_x,
    world_z,
    terrain.config.world_size
);

// Calculate normal vector for lighting
let normal = heightmap.calculate_normal(
    grid_x,
    grid_z,
    terrain.config.world_scale
);
```

### Smoothing

```rust
// Apply Gaussian-like smoothing filter
heightmap.smooth(3);  // 3 iterations
```

## Chunked LOD

The terrain is divided into chunks that can be loaded, unloaded, and rendered at different LOD levels.

### Configuration

```rust
let config = TerrainConfig {
    chunk_size: 64.0,              // Size in world units
    vertices_per_chunk: 65,        // Vertices per side (must be power of 2 + 1)
    lod_levels: 4,                 // Number of LOD levels
    lod_distances: vec![
        50.0,   // LOD 0 distance (full detail)
        100.0,  // LOD 1 distance (half detail)
        200.0,  // LOD 2 distance (quarter detail)
        400.0,  // LOD 3 distance (eighth detail)
    ],
    // ...
};
```

### LOD Calculation

Each LOD level reduces vertex density by half:

- **LOD 0**: 65×65 = 4,225 vertices per chunk (full detail)
- **LOD 1**: 33×33 = 1,089 vertices per chunk (half detail)
- **LOD 2**: 17×17 = 289 vertices per chunk (quarter detail)
- **LOD 3**: 9×9 = 81 vertices per chunk (eighth detail)

### Chunk Streaming

Chunks are automatically loaded and unloaded based on camera position:

```rust
// Update every frame
terrain.update(camera_pos);

// Get current chunk count
let active_chunks = terrain.chunk_count();

// Manual chunk access
let chunk_id = TerrainChunkId::new(5, 3);  // x=5, z=3
if let Some(chunk) = terrain.get_chunk(chunk_id) {
    println!("Chunk LOD: {}", chunk.lod.current_level);
}
```

### Skirt Geometry

Skirts are vertical quads around chunk edges that prevent gaps between different LOD levels. They're automatically generated during mesh creation.

## Texture Splatting

Blend up to 8 material layers using control maps and automatic height/slope-based distribution.

### Material Layers

```rust
use praxis_terrain::TerrainMaterialLayer;

// Grass at low elevations
let grass = TerrainMaterialLayer::new(
    "grass",         // name
    "grass_albedo",  // albedo texture
    0.0,             // min_height
    30.0             // max_height
)
.with_normal("grass_normal")  // normal map
.with_tiling(10.0);           // texture repetition

terrain.material.add_layer(grass);

// Rock on steep slopes
let rock = TerrainMaterialLayer::new("rock", "rock_albedo", 30.0, 70.0)
    .with_slope(20.0, 90.0)   // only on slopes 20-90 degrees
    .with_tiling(15.0)
    .with_normal("rock_normal");

terrain.material.add_layer(rock);

// Snow at high elevations
let snow = TerrainMaterialLayer::new("snow", "snow_albedo", 70.0, 100.0)
    .with_tiling(8.0);

terrain.material.add_layer(snow);
```

### Splat Maps

Splat maps are RGBA textures where each channel controls a material layer:

- **R channel**: Layer 0 weight
- **G channel**: Layer 1 weight
- **B channel**: Layer 2 weight
- **A channel**: Layer 3 weight

For layers 4-7, use a second splat map.

```rust
// Create splat map matching heightmap resolution
let splatmap = SplatMap::new(heightmap.width, heightmap.height);

// Load from image
let splatmap = SplatMap::from_file("terrain_splat.png")?;

// Save to image
splatmap.save_to_file("terrain_splat_edited.png")?;
```

### Shader Integration

The terrain fragment shader automatically:
1. Samples the splat map
2. Normalizes blend weights
3. Samples all layer textures
4. Blends albedo and normal maps
5. Applies lighting

## Vegetation Instancing

Render millions of grass, trees, rocks, and other objects efficiently using GPU instancing.

### Vegetation Layers

```rust
use praxis_terrain::VegetationLayer;

// Dense grass on low, flat areas
let grass = VegetationLayer::new(
    "grass",       // name
    "grass_mesh",  // mesh to instance
    "grass_mat",   // material
    5.0            // density (instances per m²)
)
.with_height_range(0.0, 40.0)      // only between 0-40m elevation
.with_slope_range(0.0, 30.0)       // only on slopes 0-30 degrees
.with_scale_range(0.8, 1.2)        // random scale 0.8-1.2x
.with_wind_strength(1.5)           // wind animation intensity
.with_color_variation(0.15)        // ±15% color variation
.with_random_rotation(true);       // random Y rotation

terrain.vegetation_layers.push(grass);
```

### Generation

```rust
// Generate all vegetation (uses parallel processing)
terrain.generate_vegetation()?;

// Generate in specific area
terrain.generate_vegetation_in_area(
    0,                          // layer_index
    Vec3::new(100.0, 0.0, 100.0),  // center
    50.0,                       // radius
    10.0                        // density
)?;
```

### Performance

- **Poisson Disc Sampling**: Ensures natural, even distribution
- **Parallel Processing**: Uses Rayon for multi-threaded generation
- **Frustum Culling**: Only visible instances are rendered
- **GPU Instancing**: Single draw call per layer, regardless of instance count

### Wind Animation

Wind is automatically applied in the vertex shader:

```glsl
// Wind displacement based on instance data
float wind_phase = instance_color_and_wind.w;
float wind_factor = position.y;  // More wind at top
vec2 wind_offset = wind_direction * sin(time + wind_phase) * wind_strength * wind_factor;
```

Control wind globally:

```rust
// Set wind for a specific layer
vegetation_layer.wind_strength = 2.0;  // stronger wind

// Or set during creation
let layer = VegetationLayer::new(...)
    .with_wind_strength(2.0);
```

## Terrain Editing

The terrain system provides tools for real-time terrain modification integrated with the Praxis editor.

### Height Sculpting

```rust
use praxis_terrain::{HeightmapBrush, TerrainEditOperation, BrushShape, BrushFalloff};

// Create brush
let brush = HeightmapBrush::new(5.0, 0.5)  // radius=5m, strength=0.5
    .with_shape(BrushShape::Circle)
    .with_falloff(BrushFalloff::Smooth);

// Apply operation
brush.apply(
    &mut terrain.heightmap,
    world_x,
    world_z,
    terrain.config.world_size,
    TerrainEditOperation::Raise,  // or Lower, Smooth, Flatten
    delta_time
)?;

// Update affected chunks
terrain.mark_area_chunks_dirty(
    Vec3::new(world_x, 0.0, world_z),
    brush.radius
);
terrain.regenerate_dirty_chunks()?;
```

### Material Painting

```rust
use praxis_terrain::PaintBrush;

// Create paint brush for layer 1
let brush = PaintBrush::new(
    5.0,   // radius
    0.5,   // strength
    1      // layer_index (0-3 for first splat map)
);

// Paint at position
brush.apply(
    &mut terrain.splatmap,
    world_x,
    world_z,
    terrain.config.world_size,
    delta_time
)?;
```

### Vegetation Painting

```rust
use praxis_terrain::VegetationPainter;

// Create painter
let painter = VegetationPainter::new(5.0, 2.0);  // radius, density

// Paint vegetation
let layer = &mut terrain.vegetation_layers[0];
painter.paint(
    layer,
    world_x,
    world_z,
    |x, z| terrain.heightmap.get_height_at(x, z, terrain.config.world_size),
    |x, z| {
        let grid_x = (x / terrain.config.world_size * terrain.heightmap.width as f32) as u32;
        let grid_z = (z / terrain.config.world_size * terrain.heightmap.height as f32) as u32;
        terrain.heightmap.calculate_normal(grid_x, grid_z, terrain.config.world_scale)
    }
)?;

// Erase vegetation
painter.erase(layer, world_x, world_z)?;
```

### Editor Integration

The `TerrainPanel` in `praxis_editor` provides a UI for:
- Height sculpting tools (raise, lower, smooth, flatten)
- Brush configuration (size, strength, shape, falloff)
- Material painting
- Vegetation painting
- Real-time preview

## Performance Optimization

### Configuration Tips

#### Chunk Size
- **32-64m**: Better culling, more draw calls, good for dense areas
- **128-256m**: Fewer draw calls, less culling precision, good for open areas
- **Recommended**: 64m for most cases

#### LOD Levels
- **More levels (5-6)**: Smoother transitions, more memory
- **Fewer levels (3-4)**: Less memory, more abrupt transitions
- **Recommended**: 4 levels for balanced quality/performance

#### LOD Distances
Scale to your world size:
```rust
// Small world (1km×1km)
lod_distances: vec![25.0, 50.0, 100.0, 200.0]

// Medium world (4km×4km)
lod_distances: vec![50.0, 100.0, 200.0, 400.0]

// Large world (16km×16km)
lod_distances: vec![100.0, 200.0, 400.0, 800.0]
```

#### Vegetation Density
Balance visual quality and performance:
- **Grass**: 2-5 instances/m² (very dense)
- **Flowers**: 0.5-2 instances/m² (moderate)
- **Trees**: 0.01-0.5 instances/m² (sparse)
- **Rocks**: 0.05-0.2 instances/m² (sparse)

Keep total instances under 1-2 million for smooth performance.

### Memory Management

```rust
// Limit active chunks by view distance
let config = TerrainConfig {
    lod_distances: vec![50.0, 100.0, 200.0, 400.0],
    // Chunks beyond 400m * 1.5 = 600m are unloaded
    // ...
};

// Reduce chunk generation rate
terrain.max_chunks_per_frame = 4;  // default: 8
```

### Parallel Processing

The terrain system uses Rayon for parallel processing:
- **Mesh Generation**: Chunks generated in parallel
- **Vegetation Distribution**: Layers processed in parallel
- **Chunk Updates**: Multiple chunks regenerated concurrently

### Profiling

```rust
use praxis_profiling::profile_scope;

{
    let _scope = profile_scope!("terrain_update");
    terrain.update(camera_pos);
}

{
    let _scope = profile_scope!("terrain_regenerate");
    terrain.regenerate_dirty_chunks()?;
}
```

## Advanced Topics

### Custom Heightmap Algorithms

```rust
impl TerrainHeightmap {
    pub fn apply_erosion(&mut self, iterations: u32) {
        for _ in 0..iterations {
            // Thermal erosion algorithm
            // Hydraulic erosion simulation
            // etc.
        }
    }
    
    pub fn apply_terracing(&mut self, levels: u32, smoothness: f32) {
        // Quantize heights to create terraces
        for height in self.heights_mut() {
            *height = (*height / levels as f32).round() * levels as f32;
        }
        self.smooth(smoothness as u32);
    }
}
```

### Custom Material Distribution

```rust
// Override automatic blend weight calculation
for y in 0..heightmap.height {
    for x in 0..heightmap.width {
        let height = heightmap.get_height(x, y);
        let normal = heightmap.calculate_normal(x, y, 1.0);
        let slope = normal.angle_between(Vec3::Y).to_degrees();
        
        // Custom logic
        let weights = if slope > 45.0 {
            [0.0, 1.0, 0.0, 0.0]  // all rock
        } else if height > 80.0 {
            [0.0, 0.0, 1.0, 0.0]  // all snow
        } else {
            [1.0, 0.0, 0.0, 0.0]  // all grass
        };
        
        terrain.splatmap.set_weights(x, y, weights);
    }
}
```

### Streaming from Disk

```rust
// Load heightmap tiles on demand
struct TerrainStreamer {
    cache: HashMap<(i32, i32), TerrainHeightmap>,
}

impl TerrainStreamer {
    fn load_tile(&mut self, tile_x: i32, tile_z: i32) -> Result<&TerrainHeightmap> {
        if !self.cache.contains_key(&(tile_x, tile_z)) {
            let path = format!("terrain/tile_{}_{}.png", tile_x, tile_z);
            let heightmap = TerrainHeightmap::from_file(path, 100.0)?;
            self.cache.insert((tile_x, tile_z), heightmap);
        }
        Ok(&self.cache[&(tile_x, tile_z)])
    }
}
```

### Multi-threaded Mesh Generation

```rust
use rayon::prelude::*;

// Generate multiple chunks in parallel
let chunk_ids = vec![
    TerrainChunkId::new(0, 0),
    TerrainChunkId::new(0, 1),
    TerrainChunkId::new(1, 0),
    TerrainChunkId::new(1, 1),
];

let meshes: Vec<_> = chunk_ids
    .par_iter()
    .map(|&chunk_id| {
        terrain.generate_chunk_mesh(chunk_id, 0)
    })
    .collect();
```

## See Also

- [Procedural Textures](procedural_textures.md) - For generating terrain textures
- [LOD System](lod_system.md) - General LOD concepts
- [Spatial Optimization](guides/spatial-optimization.md) - For frustum culling
- [Editor System](editor_system.md) - For terrain editing UI

## Examples

Run the terrain demo:
```bash
cargo run --example terrain_demo --release
```

The demo showcases:
- Procedural heightmap generation
- Multi-layer material setup
- Vegetation distribution across 4 layers  
- Real-time LOD updates
- Performance statistics

## License

MIT License - See LICENSE file for details.
