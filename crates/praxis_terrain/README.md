# Praxis Terrain

Comprehensive terrain rendering system for the Praxis engine with heightmap-based terrain, chunked LOD, texture splatting, vegetation instancing, and editor tools.

## Features

### Heightmap-Based Terrain
- **Flexible heightmap sources**: Load from images, procedural noise, or custom data
- **High precision**: Uses floating-point height values for smooth terrain
- **Bilinear interpolation**: Smooth height queries at arbitrary world positions
- **Normal calculation**: Automatic normal computation for lighting
- **Smoothing filter**: Built-in smoothing for terrain refinement

### Chunked LOD System
- **Seamless LOD transitions**: Smooth transitions between detail levels
- **Distance-based culling**: Automatic chunk activation/deactivation
- **Multiple LOD levels**: Configurable LOD levels (default: 4)
- **Skirt geometry**: Prevents gaps between different LOD levels
- **Efficient memory**: Only active chunks are kept in memory
- **Parallel processing**: Chunk generation and updates use Rayon for parallelization

### Texture Splatting
- **Multi-layer materials**: Up to 8 material layers per terrain
- **Height-based blending**: Automatic material distribution by height
- **Slope-based blending**: Assign materials based on terrain steepness
- **Control maps**: Paint custom material distributions with splat maps
- **Tiling control**: Independent texture tiling for each layer
- **Normal mapping**: Per-layer normal maps for detailed surfaces
- **PBR properties**: Metallic, roughness, and other PBR properties per layer

### Vegetation Instancing
- **GPU instancing**: Render millions of vegetation instances efficiently
- **Multiple layers**: Support for grass, trees, rocks, flowers, etc.
- **Procedural placement**: Poisson disc sampling for natural distribution
- **Height/slope filtering**: Control where vegetation appears
- **Wind animation**: Per-instance wind simulation in shaders
- **Color variation**: Randomized colors for natural appearance
- **Scale variation**: Random scale factors for diversity
- **Distance culling**: Automatic frustum culling for performance

### Terrain Editing Tools
- **Height sculpting**: Raise, lower, smooth, and flatten terrain
- **Brush shapes**: Circle and square brushes with configurable falloff
- **Material painting**: Paint splat maps to blend material layers
- **Vegetation painting**: Place and remove vegetation instances interactively
- **Undo/redo support**: Full integration with editor command system
- **Real-time preview**: See changes immediately in the viewport
- **Area-based updates**: Only affected chunks are regenerated

## Architecture

```
TerrainSystem
├── TerrainHeightmap         (Height data storage)
├── TerrainChunk[]           (Spatial partitioning)
│   ├── ChunkLod             (LOD state per chunk)
│   └── GpuMesh[]            (One mesh per LOD level)
├── TerrainMaterial          (Material layer configuration)
├── SplatMap                 (Material blend weights)
├── VegetationLayer[]        (Vegetation configuration)
│   └── VegetationInstance[] (Instance transforms)
├── TerrainRenderer          (Terrain rendering system)
└── VegetationRenderer       (Vegetation instancing system)
```

## Usage

### Basic Terrain Setup

```rust
use praxis_terrain::{TerrainConfig, TerrainHeightmap, TerrainSystem};

// Create heightmap from procedural noise
let heightmap = TerrainHeightmap::from_noise(512, 512, 100.0, 4.0, 6);

// Configure terrain system
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
terrain.initialize_rendering(device, memory_allocator, command_buffer_allocator);
```

### Loading Heightmap from Image

```rust
use praxis_terrain::TerrainHeightmap;

// Load from 8-bit grayscale PNG/JPEG
let heightmap = TerrainHeightmap::from_file("terrain_height.png", 100.0)?;

// Or create from raw data
let width = 512;
let height = 512;
let heights: Vec<f32> = vec![0.0; (width * height) as usize];
let heightmap = TerrainHeightmap::from_heights(width, height, heights, 100.0);
```

### Adding Material Layers

```rust
use praxis_terrain::TerrainMaterialLayer;

// Grass at low elevations
let grass = TerrainMaterialLayer::new("grass", "grass_albedo", 0.0, 30.0)
    .with_normal("grass_normal")
    .with_tiling(10.0);
terrain.material.add_layer(grass);

// Rock on steep slopes
let rock = TerrainMaterialLayer::new("rock", "rock_albedo", 30.0, 70.0)
    .with_slope(20.0, 90.0)
    .with_tiling(15.0)
    .with_normal("rock_normal");
terrain.material.add_layer(rock);

// Snow at high elevations
let snow = TerrainMaterialLayer::new("snow", "snow_albedo", 70.0, 100.0)
    .with_tiling(8.0);
terrain.material.add_layer(snow);
```

### Adding Vegetation

```rust
use praxis_terrain::VegetationLayer;

// Dense grass on low, flat areas
let grass = VegetationLayer::new("grass", "grass_mesh", "grass_mat", 5.0)
    .with_height_range(0.0, 40.0)
    .with_slope_range(0.0, 30.0)
    .with_scale_range(0.8, 1.2)
    .with_wind_strength(1.5)
    .with_color_variation(0.15);
terrain.vegetation_layers.push(grass);

// Sparse trees on mid-elevation slopes
let trees = VegetationLayer::new("trees", "tree_mesh", "tree_mat", 0.5)
    .with_height_range(20.0, 60.0)
    .with_slope_range(0.0, 25.0)
    .with_scale_range(0.8, 1.5)
    .with_random_rotation(true);
terrain.vegetation_layers.push(trees);

// Generate all vegetation instances (uses parallel processing)
terrain.generate_vegetation()?;
```

### Runtime Updates

```rust
use praxis_math::Vec3;

// Update terrain LOD based on camera position (call every frame)
let camera_pos = Vec3::new(100.0, 50.0, 200.0);
terrain.update(camera_pos);

// Regenerate chunks after height editing
terrain.mark_all_chunks_dirty();
terrain.regenerate_dirty_chunks()?;

// Or mark only affected area
terrain.mark_area_chunks_dirty(edit_center, edit_radius);
terrain.regenerate_dirty_chunks()?;
```

### Terrain Editing

```rust
use praxis_terrain::{HeightmapBrush, TerrainEditOperation, BrushShape, BrushFalloff};

// Create a sculpting brush
let brush = HeightmapBrush::new(5.0, 0.5)
    .with_shape(BrushShape::Circle)
    .with_falloff(BrushFalloff::Smooth);

// Raise terrain at a position
brush.apply(
    &mut terrain.heightmap,
    world_x,
    world_z,
    terrain.config.world_size,
    TerrainEditOperation::Raise,
    delta_time,
)?;

// Mark affected chunks for regeneration
terrain.mark_area_chunks_dirty(Vec3::new(world_x, 0.0, world_z), brush.radius);
terrain.regenerate_dirty_chunks()?;
```

### Material Painting

```rust
use praxis_terrain::PaintBrush;

// Create a paint brush for layer 1 (rock)
let paint_brush = PaintBrush::new(5.0, 0.5, 1);

// Paint at a position
paint_brush.apply(
    &mut terrain.splatmap,
    world_x,
    world_z,
    terrain.config.world_size,
    delta_time,
)?;
```

### Vegetation Painting

```rust
use praxis_terrain::VegetationPainter;

// Create a vegetation painter
let painter = VegetationPainter::new(5.0, 2.0);

// Paint vegetation instances
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
    },
)?;

// Erase vegetation in an area
painter.erase(layer, world_x, world_z)?;

// Or generate in specific area
terrain.generate_vegetation_in_area(layer_index, center, radius, density)?;
```

## Performance Characteristics

### Memory Usage
- **Base terrain**: ~4 bytes per heightmap sample (512×512 = 1 MB)
- **Per chunk**: ~2-8 KB per LOD level depending on vertex count
- **Vegetation**: ~64 bytes per instance (transforms + color)
- **Splat maps**: 16 bytes per sample (4 RGBA channels)

### Rendering Performance
- **LOD system**: Reduces triangle count by ~75% at distance
- **Frustum culling**: Only visible chunks are rendered
- **Instancing**: Vegetation rendering is O(layers) not O(instances)
- **Typical scene**: 50-200 chunks active, 100k-1M vegetation instances
- **Parallel processing**: Chunk generation uses all CPU cores

### Generation Performance
- **Mesh generation**: Parallelized using Rayon, ~1ms per chunk
- **Vegetation distribution**: Poisson sampling, ~50-100ms for entire terrain
- **Heightmap smoothing**: ~10ms per iteration (512×512)
- **Chunk loading**: ~8 chunks per frame by default

## Configuration Tips

### Chunk Size vs LOD Levels
- Smaller chunks (32-64): Better culling, more draw calls
- Larger chunks (128-256): Fewer draw calls, less culling precision
- More LOD levels: Smoother transitions, more memory
- Fewer LOD levels: Less memory, more abrupt transitions

### Vegetation Density
- Grass: 2-10 instances/m² (very dense)
- Small plants: 0.5-2 instances/m² (moderate)
- Trees: 0.01-0.5 instances/m² (sparse)
- Total instances: Keep under 1-2 million for smooth performance

### Texture Tiling
- Ground textures: 5-15 tiles across chunk
- Rock/cliff textures: 10-20 tiles for detail
- Large features: 1-5 tiles for variety

### LOD Distances
- LOD 0: 0-50m (full detail)
- LOD 1: 50-100m (half detail)
- LOD 2: 100-200m (quarter detail)
- LOD 3: 200-400m (eighth detail)

## Integration with Editor

The terrain system integrates seamlessly with the Praxis editor:

```rust
use praxis_editor::TerrainPanel;

// Add terrain panel to editor (automatically included)
// The panel provides:
// - Height sculpting tools (raise, lower, smooth, flatten)
// - Material painting tools
// - Vegetation painting tools
// - Brush configuration (size, strength, shape, falloff)
// - Real-time preview of edits
```

## Shader Integration

The terrain system provides specialized shaders in `src/shaders/`:

### Terrain Shaders
- **`terrain.vert`**: Terrain vertex shader with TBN matrix calculation
- **`terrain.frag`**: Multi-layer texture splatting with normal mapping

Features:
- Up to 8 material layers
- Splat map sampling for blend weights
- Per-layer texture tiling
- Normal map blending
- Height and slope-based material distribution

### Vegetation Shaders
- **`vegetation.vert`**: Instanced vegetation with wind animation
- **`vegetation.frag`**: Alpha-tested foliage rendering

Features:
- GPU instancing for millions of instances
- Per-instance transform matrices
- Wind animation using sinusoidal wave
- Color variation per instance
- Alpha testing for foliage cutouts

## Examples

See `examples/terrain_demo.rs` for a complete working example demonstrating:
- Procedural heightmap generation
- Multi-layer material setup
- Vegetation distribution across 4 layers
- Runtime LOD updates
- Camera controls
- Performance statistics

Run with:
```bash
cargo run --example terrain_demo --release
```

## Performance Optimization

### Best Practices
1. **Use LOD aggressively**: Set LOD distances to match your scene scale
2. **Limit vegetation density**: More than 1M instances may impact performance
3. **Enable frustum culling**: Reduces rendering overhead significantly
4. **Batch chunk updates**: Regenerate multiple chunks in parallel
5. **Cache terrain meshes**: Keep generated meshes in GPU memory
6. **Use texture atlases**: Combine material textures to reduce bind calls

### Profiling
The terrain system integrates with `praxis_profiling`:
```rust
use praxis_profiling::profile_scope;

{
    let _scope = profile_scope!("terrain_update");
    terrain.update(camera_pos);
}
```

## Common Issues

### Gaps Between Chunks
- Ensure `world_scale` matches heightmap resolution
- Use skirt geometry (automatic in mesh generation)
- Check LOD transition distances

### Vegetation Not Appearing
- Verify height/slope ranges match terrain
- Check vegetation density (too low may not place instances)
- Ensure meshes are loaded before rendering

### Performance Issues
- Reduce LOD levels or increase LOD distances
- Lower vegetation density
- Enable frustum culling
- Reduce chunk size or vertex count

### Memory Usage
- Limit active chunk count by reducing view distance
- Use lower LOD levels
- Reduce vegetation instance count
- Use texture compression

## License

MIT License - See LICENSE file for details.
