# Terrain API Reference

API reference for heightmap terrain with LOD and vegetation.

## Core Types

### TerrainSystem

Main terrain management system.

```rust
pub struct TerrainSystem { /* ... */ }
```

**Methods:**
- `new(config: TerrainConfig, heightmap: TerrainHeightmap) -> Result<Self>`
- `initialize_rendering(device, memory_allocator, cmd_allocator)`
- `update(camera_pos: Vec3) -> Result<()>` - Update LOD based on camera
- `get_height_at(x: f32, z: f32) -> Option<f32>` - Query height
- `set_height_at(x: f32, z: f32, height: f32)` - Modify terrain
- `generate_vegetation() -> Result<()>`
- `clear_vegetation()`

### TerrainConfig

Configuration for terrain generation and rendering.

```rust
pub struct TerrainConfig {
    pub chunk_size: f32,              // Size of each chunk in world units
    pub vertices_per_chunk: usize,    // Vertex resolution per chunk
    pub max_height: f32,              // Maximum terrain height
    pub lod_levels: usize,            // Number of LOD levels
    pub lod_distances: Vec<f32>,      // Distance thresholds for each LOD
    pub world_size: f32,              // Total terrain size
}
```

**Methods:**
- `default()` - Standard configuration
- `with_lod_levels(levels: usize, base_distance: f32)` - Auto-generate distances

### TerrainHeightmap

Height data for terrain.

```rust
pub struct TerrainHeightmap { /* ... */ }
```

**Methods:**
- `from_image(path: &str, max_height: f32) -> Result<Self>`
- `from_noise(width: usize, height: usize, max_height: f32, scale: f32, octaves: u32) -> Self`
- `from_data(width: usize, height: usize, data: Vec<f32>) -> Self`
- `get_height(x: usize, y: usize) -> f32`
- `set_height(x: usize, y: usize, height: f32)`
- `width() -> usize`
- `height() -> usize`
- `sample_bilinear(x: f32, y: f32) -> f32` - Interpolated height

## Materials

### TerrainMaterial

Multi-layer material system.

```rust
pub struct TerrainMaterial {
    pub layers: Vec<TerrainMaterialLayer>,
}
```

**Methods:**
- `new()` - Empty material
- `add_layer(layer: TerrainMaterialLayer)`
- `remove_layer(index: usize)`
- `layer_count() -> usize`

### TerrainMaterialLayer

Individual material layer with blending rules.

```rust
pub struct TerrainMaterialLayer {
    pub name: String,
    pub albedo_texture: String,
    pub normal_texture: Option<String>,
    pub height_range: (f32, f32),     // Min/max elevation for this layer
    pub slope_range: (f32, f32),      // Min/max slope in degrees
    pub tiling: f32,                  // Texture tiling factor
    pub blend_sharpness: f32,         // Blending smoothness
}
```

**Methods:**
- `new(name: &str, albedo: &str, min_height: f32, max_height: f32)` - Create layer
- `with_normal(normal: &str)` - Add normal map
- `with_slope(min: f32, max: f32)` - Set slope range
- `with_tiling(tiling: f32)` - Set texture tiling
- `with_blend_sharpness(sharpness: f32)` - Control blending

## Vegetation

### VegetationLayer

Defines procedural vegetation placement.

```rust
pub struct VegetationLayer {
    pub name: String,
    pub mesh_name: String,
    pub material_name: String,
    pub density: f32,                 // Instances per square unit
    pub height_range: (f32, f32),
    pub slope_range: (f32, f32),
    pub scale_range: (f32, f32),      // Min/max random scale
    pub rotation_variance: f32,       // Rotation randomness (0-1)
    pub wind_strength: f32,           // Wind animation intensity
}
```

**Methods:**
- `new(name: &str, mesh: &str, material: &str, density: f32)` - Create layer
- `with_height_range(min: f32, max: f32)`
- `with_slope_range(min: f32, max: f32)`
- `with_scale_range(min: f32, max: f32)`
- `with_rotation_variance(variance: f32)`
- `with_wind_strength(strength: f32)`

### VegetationConfig

Global vegetation settings.

```rust
pub struct VegetationConfig {
    pub max_view_distance: f32,       // Cull beyond this distance
    pub lod_distances: Vec<f32>,      // LOD switching distances
    pub update_frequency: f32,        // Updates per second
}
```

## Editing

### TerrainEditor

Tools for runtime terrain editing.

```rust
pub struct TerrainEditor { /* ... */ }
```

**Methods:**
- `new(terrain: &mut TerrainSystem) -> Self`
- `sculpt(center: Vec2, radius: f32, strength: f32, delta: f32)` - Raise/lower
- `smooth(center: Vec2, radius: f32, strength: f32)`
- `flatten(center: Vec2, radius: f32, target_height: f32, strength: f32)`
- `paint_layer(center: Vec2, radius: f32, layer_index: usize, strength: f32)`
- `place_vegetation(layer: &VegetationLayer, position: Vec3) -> bool`
- `remove_vegetation(radius: Vec3, position: Vec3)`

### BrushShape

Shape of editing brush.

```rust
pub enum BrushShape {
    Circle,
    Square,
    Custom { shape_data: Vec<f32> },
}
```

## Common Patterns

### Basic Terrain Setup

```rust
use praxis_terrain::{TerrainConfig, TerrainHeightmap, TerrainSystem};

// Generate heightmap from noise
let heightmap = TerrainHeightmap::from_noise(512, 512, 100.0, 4.0, 6);

// Configure terrain
let config = TerrainConfig {
    chunk_size: 64.0,
    vertices_per_chunk: 65,
    max_height: 100.0,
    lod_levels: 4,
    lod_distances: vec![50.0, 100.0, 200.0, 400.0],
    world_size: 1024.0,
};

// Create terrain
let mut terrain = TerrainSystem::new(config, heightmap)?;
terrain.initialize_rendering(device, allocator, cmd_allocator);
```

### Multi-Layer Materials

```rust
use praxis_terrain::{TerrainMaterial, TerrainMaterialLayer};

let mut material = TerrainMaterial::new();

// Grass at low elevations
material.add_layer(
    TerrainMaterialLayer::new("grass", "grass_albedo", 0.0, 30.0)
        .with_normal("grass_normal")
        .with_tiling(10.0)
        .with_slope(0.0, 30.0)
);

// Dirt on slopes
material.add_layer(
    TerrainMaterialLayer::new("dirt", "dirt_albedo", 10.0, 60.0)
        .with_normal("dirt_normal")
        .with_tiling(12.0)
        .with_slope(25.0, 60.0)
);

// Rock on steep areas and peaks
material.add_layer(
    TerrainMaterialLayer::new("rock", "rock_albedo", 30.0, 100.0)
        .with_normal("rock_normal")
        .with_tiling(15.0)
        .with_slope(45.0, 90.0)
);

// Snow on peaks
material.add_layer(
    TerrainMaterialLayer::new("snow", "snow_albedo", 70.0, 100.0)
        .with_tiling(8.0)
        .with_slope(0.0, 45.0)
);
```

### Adding Vegetation

```rust
use praxis_terrain::VegetationLayer;

// Grass
terrain.vegetation_layers.push(
    VegetationLayer::new("grass", "grass_mesh", "grass_mat", 5.0)
        .with_height_range(0.0, 40.0)
        .with_slope_range(0.0, 30.0)
        .with_scale_range(0.8, 1.2)
        .with_wind_strength(1.5)
);

// Trees
terrain.vegetation_layers.push(
    VegetationLayer::new("trees", "tree_mesh", "tree_mat", 0.2)
        .with_height_range(10.0, 60.0)
        .with_slope_range(0.0, 25.0)
        .with_scale_range(0.9, 1.4)
        .with_wind_strength(0.8)
);

// Rocks
terrain.vegetation_layers.push(
    VegetationLayer::new("rocks", "rock_mesh", "rock_mat", 0.5)
        .with_height_range(0.0, 80.0)
        .with_slope_range(30.0, 90.0)
        .with_scale_range(0.5, 2.0)
        .with_rotation_variance(1.0)
);

terrain.generate_vegetation()?;
```

### Runtime Update

```rust
fn terrain_update_system(
    mut terrain: ResMut<TerrainSystem>,
    camera: Query<&Transform, With<Camera>>,
) {
    if let Ok(camera_transform) = camera.get_single() {
        let camera_pos = camera_transform.translation;
        terrain.update(camera_pos).unwrap();
    }
}
```

### Terrain Editing

```rust
use praxis_terrain::TerrainEditor;

fn terrain_sculpt_system(
    input: Res<InputState>,
    mut editor: ResMut<TerrainEditor>,
) {
    if input.is_mouse_button_pressed(MouseButton::Left) {
        let cursor_world_pos = get_cursor_world_position();
        let center = Vec2::new(cursor_world_pos.x, cursor_world_pos.z);
        
        // Sculpt terrain at cursor
        editor.sculpt(center, 5.0, 0.5, 0.016);
    }
}
```

### Height Queries

```rust
// Get height at world position
if let Some(height) = terrain.get_height_at(10.0, 20.0) {
    // Place object on terrain
    let position = Vec3::new(10.0, height, 20.0);
}

// Modify height
terrain.set_height_at(10.0, 20.0, 50.0);
```

## Performance Considerations

### LOD Configuration

```rust
// Aggressive LOD (better performance, more pop-in)
let config = TerrainConfig {
    lod_levels: 5,
    lod_distances: vec![30.0, 60.0, 120.0, 240.0, 480.0],
    ..Default::default()
};

// Conservative LOD (better quality, more expensive)
let config = TerrainConfig {
    lod_levels: 3,
    lod_distances: vec![100.0, 250.0, 500.0],
    ..Default::default()
};
```

### Chunk Size Trade-offs

- **Small chunks (32-64)**: Better culling, more draw calls
- **Large chunks (128-256)**: Fewer draw calls, less efficient culling
- **Recommended**: 64.0 for most cases

### Vegetation Density

```rust
// Dense vegetation (beautiful but expensive)
VegetationLayer::new("grass", "grass", "grass_mat", 10.0)

// Sparse vegetation (better performance)
VegetationLayer::new("grass", "grass", "grass_mat", 2.0)

// Use LOD for distant vegetation
let config = VegetationConfig {
    max_view_distance: 200.0,
    lod_distances: vec![50.0, 100.0],
    ..Default::default()
};
```

## See Also

- [Terrain Guide](../guides/terrain.md) - Comprehensive terrain system guide
- [praxis_terrain crate](../../crates/praxis_terrain/README.md) - Crate documentation
