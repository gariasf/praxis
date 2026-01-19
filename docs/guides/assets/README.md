# Assets Guide

Practical guide to loading and managing meshes, textures, and other assets in Praxis using OBJ and GLTF formats.

## In This Section

- **[GLTF Loading](gltf.md)** - Comprehensive GLTF format support
- **[OBJ Loading](obj.md)** - Simple OBJ mesh loading
- **[Procedural Textures](procedural-textures.md)** - Runtime texture generation

## Quick Start

### Load an OBJ Mesh

```rust
use praxis_assets::load_obj_mesh;
use praxis_graphics::RenderContext;

fn init(render_context: &mut RenderContext) -> Result<()> {
    // Load and upload mesh in one call
    load_obj_mesh(
        render_context.mesh_manager_mut(),
        "spaceship",
        "assets/models/spaceship.obj"
    )?;
    
    Ok(())
}
```

### Use Loaded Mesh

```rust
use praxis_ecs::{MeshHandle, Transform};

world.spawn((
    Transform::from_xyz(0.0, 0.0, 0.0),
    MeshHandle::new("spaceship"),
));
```

## OBJ File Loading

### Supported OBJ Features

**Supported:**
- Vertex positions (`v`)
- Vertex normals (`vn`)
- Texture coordinates (`vt`)
- Face definitions (`f`)
- Automatic triangulation

**Not Supported:**
- Material files (`.mtl`)
- Multiple objects per file
- Vertex colors

### Loading Methods

#### Method 1: High-Level Function

Simplest approach:

```rust
use praxis_assets::load_obj_mesh;

load_obj_mesh(mesh_manager, "cube", "assets/models/cube.obj")?;
load_obj_mesh(mesh_manager, "sphere", "assets/models/sphere.obj")?;
```

#### Method 2: Using AssetLoader Trait

More control over the loading process:

```rust
use praxis_assets::{AssetLoader, MeshLoader};

let loader = MeshLoader::new();
let mesh_data = loader.load("assets/models/character.obj")?;

// Optional: process mesh_data here
// - Optimize vertices
// - Calculate tangents
// - Apply transforms

mesh_manager.load_mesh("character", mesh_data)?;
```

#### Method 3: Manual Loading

Full control for custom processing:

```rust
use praxis_assets::load_obj;

let mut mesh_data = load_obj("assets/models/terrain.obj")?;

// Custom processing
mesh_data.calculate_normals();
mesh_data.optimize_vertices();

mesh_manager.load_mesh("terrain", mesh_data)?;
```

## GLTF File Loading

GLTF is the recommended format for complex assets with materials, textures, and animations.

### Basic GLTF Loading

```rust
use praxis_assets::GltfLoader;

let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/character.gltf")?;

println!("Loaded {} meshes", asset.meshes.len());
println!("Loaded {} materials", asset.materials.len());
println!("Loaded {} textures", asset.textures.len());
```

### Upload Meshes from GLTF

```rust
// Upload all meshes
for (i, mesh) in asset.meshes.iter().enumerate() {
    render_context
        .mesh_manager_mut()
        .load_mesh(format!("mesh_{}", i), mesh.clone())?;
}

// Spawn entities with meshes
for (node_index, node) in asset.nodes_with_meshes() {
    if let Some(mesh_index) = node.mesh_index {
        world.spawn((
            Transform::from_matrix(node.transform),
            MeshHandle::new(format!("mesh_{}", mesh_index)),
        ));
    }
}
```

### Complete GLTF Scene Loading

```rust
fn load_gltf_scene(
    world: &mut World,
    render_context: &mut RenderContext,
    path: &str,
) -> Result<()> {
    let loader = GltfLoader::new();
    let asset = loader.load_gltf(path)?;
    
    // Upload meshes
    for (i, mesh) in asset.meshes.iter().enumerate() {
        render_context
            .mesh_manager_mut()
            .load_mesh(format!("gltf_mesh_{}", i), mesh.clone())?;
    }
    
    // Upload textures
    for (i, texture) in asset.textures.iter().enumerate() {
        render_context
            .texture_manager_mut()
            .load_texture(format!("gltf_tex_{}", i), texture.clone())?;
    }
    
    // Spawn entities
    for node in &asset.nodes {
        let mut entity = world.spawn(Transform::from_matrix(node.transform));
        
        if let Some(mesh_index) = node.mesh_index {
            entity.insert(MeshHandle::new(format!("gltf_mesh_{}", mesh_index)));
        }
        
        if let Some(name) = &node.name {
            entity.insert(Name::new(name.clone()));
        }
    }
    
    Ok(())
}
```

## GLTF Scene Hierarchy

### Traverse Scene Graph

```rust
// Depth-first traversal
asset.traverse_depth_first(|node_index, node, depth| {
    let indent = "  ".repeat(depth);
    println!("{}{}: {:?}", indent, node_index, node.name);
});

// Find nodes with meshes
for (node_index, node) in asset.nodes_with_meshes() {
    println!("Node {} has mesh {}", node_index, node.mesh_index.unwrap());
}
```

### Extract Transforms

```rust
for node in &asset.nodes {
    let (translation, rotation, scale) = node.decompose_transform();
    
    world.spawn(Transform {
        translation,
        rotation,
        scale,
    });
}
```

## Materials and Textures

### GLTF Materials

```rust
use praxis_graphics::MaterialProperties;

for material in &asset.materials {
    let props = MaterialProperties {
        albedo: material.base_color_factor,
        metallic: material.metallic_factor,
        roughness: material.roughness_factor,
        emissive: 0.0,
        _padding: 0.0,
    };
    
    // Use material properties...
}
```

### Load Textures from GLTF

```rust
// Upload all textures
for (i, texture) in asset.textures.iter().enumerate() {
    let texture_id = format!("texture_{}", i);
    
    render_context
        .texture_manager_mut()
        .load_texture_from_data(
            texture_id,
            texture.width,
            texture.height,
            &texture.data
        )?;
}

// Apply texture to entity
if let Some(tex_index) = material.base_color_texture_index {
    entity.insert(TextureHandle::new(format!("texture_{}", tex_index)));
}
```

## Asset Manager with Caching

### GLTF Asset Manager

Automatically caches loaded assets:

```rust
use praxis_assets::GltfAssetManager;

let mut manager = GltfAssetManager::new();

// First load: reads from disk
let asset1 = manager.load("assets/models/character.gltf")?;

// Second load: returns cached version (instant)
let asset2 = manager.load("assets/models/character.gltf")?;

// Check if loaded
if manager.is_loaded("assets/models/character.gltf") {
    println!("Asset is cached");
}

// Unload when done
manager.unload("assets/models/character.gltf");
```

## Animation Loading from GLTF

```rust
use praxis_scene::{AnimationPlayer, Skeleton, AnimatedPose};

let asset = loader.load_gltf("assets/models/animated_char.gltf")?;

// Extract skeleton
if let Some(skin) = asset.skins.first() {
    let skeleton = skin.skeleton.clone();
    
    // Create animation player
    let mut player = AnimationPlayer::new();
    for animation in &asset.animations {
        let name = animation.name.clone()
            .unwrap_or_else(|| format!("Anim_{}", player.clip_count()));
        player.add_clip(name, animation.clip.clone());
    }
    
    // Spawn animated entity
    let pose = AnimatedPose::new(skeleton.bone_count());
    world.spawn((skeleton, player, pose));
}
```

## Common Patterns

### Asset Loading System

```rust
#[derive(Resource)]
struct AssetRegistry {
    gltf_manager: GltfAssetManager,
    loaded_models: HashMap<String, Entity>,
}

fn load_asset_on_demand(
    mut registry: ResMut<AssetRegistry>,
    requests: Query<(&AssetRequest, Entity), Added<AssetRequest>>,
    mut commands: Commands,
) {
    for (request, entity) in requests.iter() {
        match registry.gltf_manager.load(&request.path) {
            Ok(asset) => {
                commands.entity(entity).insert(AssetLoaded);
                registry.loaded_models.insert(request.path.clone(), entity);
            }
            Err(e) => {
                tracing::error!("Failed to load {}: {}", request.path, e);
            }
        }
    }
}
```

### Preload Assets

```rust
fn preload_assets(
    mut render_context: ResMut<RenderContext>,
) -> Result<()> {
    let assets = vec![
        ("player", "assets/models/player.obj"),
        ("enemy", "assets/models/enemy.obj"),
        ("terrain", "assets/models/terrain.obj"),
    ];
    
    for (name, path) in assets {
        load_obj_mesh(render_context.mesh_manager_mut(), name, path)?;
    }
    
    Ok(())
}
```

### Spawn Prefab from GLTF

```rust
fn spawn_prefab(
    commands: &mut Commands,
    asset_manager: &mut GltfAssetManager,
    prefab_name: &str,
    position: Vec3,
) -> Result<Entity> {
    let path = format!("assets/prefabs/{}.gltf", prefab_name);
    let asset = asset_manager.load(&path)?;
    
    // Find root node
    let root_node = &asset.nodes[0];
    
    let entity = commands.spawn((
        Transform::from_translation(position),
        Name::new(prefab_name.to_string()),
    )).id();
    
    // Add mesh if present
    if let Some(mesh_index) = root_node.mesh_index {
        commands.entity(entity).insert(
            MeshHandle::new(format!("{}_{}", prefab_name, mesh_index))
        );
    }
    
    Ok(entity)
}
```

### Batch Load Multiple Assets

```rust
struct AssetList {
    items: Vec<(String, String)>,  // (name, path)
}

fn batch_load_obj(
    mesh_manager: &mut MeshAssetManager,
    assets: &AssetList,
) -> Result<()> {
    for (name, path) in &assets.items {
        match load_obj_mesh(mesh_manager, name, path) {
            Ok(_) => tracing::info!("Loaded: {}", name),
            Err(e) => tracing::error!("Failed to load {}: {}", name, e),
        }
    }
    Ok(())
}
```

### Asset Hot-Reload

```rust
#[derive(Resource)]
struct AssetWatcher {
    paths: HashMap<String, SystemTime>,
}

fn check_asset_changes(
    mut watcher: ResMut<AssetWatcher>,
    mut mesh_manager: ResMut<MeshAssetManager>,
) {
    for (name, last_modified) in watcher.paths.iter_mut() {
        if let Ok(metadata) = std::fs::metadata(name) {
            if let Ok(modified) = metadata.modified() {
                if modified > *last_modified {
                    tracing::info!("Reloading changed asset: {}", name);
                    
                    if let Err(e) = load_obj_mesh(&mut mesh_manager, name, name) {
                        tracing::error!("Failed to reload: {}", e);
                    } else {
                        *last_modified = modified;
                    }
                }
            }
        }
    }
}
```

## File Format Requirements

### OBJ Files

```obj
# vertices
v 0.0 1.0 0.0
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0

# normals
vn 0.0 0.0 1.0

# texture coordinates
vt 0.5 1.0
vt 0.0 0.0
vt 1.0 0.0

# faces (vertex/texcoord/normal)
f 1/1/1 2/2/1 3/3/1
```

**Requirements:**
- Must have vertex positions
- Faces must be triangles (or auto-triangulated)
- Vertex count must be ≤ 65,535 (u16 limit)

### GLTF Files

```json
{
  "scene": 0,
  "scenes": [{"nodes": [0]}],
  "nodes": [
    {
      "mesh": 0,
      "translation": [0, 0, 0]
    }
  ],
  "meshes": [
    {
      "primitives": [
        {
          "attributes": {
            "POSITION": 0,
            "NORMAL": 1,
            "TEXCOORD_0": 2
          },
          "indices": 3,
          "material": 0
        }
      ]
    }
  ]
}
```

**Supported Features:**
- Meshes with positions, normals, UVs, tangents
- PBR materials (metallic-roughness)
- Textures (PNG, JPEG)
- Node transforms and hierarchy
- Skeletal animations

## Error Handling

### Robust Loading

```rust
fn safe_load_asset(path: &str) -> Result<()> {
    match load_obj_mesh(mesh_manager, "model", path) {
        Ok(_) => {
            tracing::info!("Successfully loaded: {}", path);
            Ok(())
        }
        Err(e) => {
            tracing::error!("Failed to load {}: {}", path, e);
            
            // Load fallback model
            load_obj_mesh(mesh_manager, "model", "assets/models/fallback.obj")?;
            
            Err(e)
        }
    }
}
```

### Validate Assets

```rust
fn validate_mesh(mesh: &MeshData) -> Result<()> {
    if mesh.vertices.is_empty() {
        return Err(eyre::eyre!("Mesh has no vertices"));
    }
    
    if mesh.indices.is_empty() {
        return Err(eyre::eyre!("Mesh has no indices"));
    }
    
    if mesh.indices.len() % 3 != 0 {
        return Err(eyre::eyre!("Index count not divisible by 3"));
    }
    
    Ok(())
}
```

## Performance Tips

### Async Loading

Load assets in background threads:

```rust
use std::sync::Arc;
use std::sync::Mutex;

fn load_assets_async(paths: Vec<String>) -> Vec<JoinHandle<Result<MeshData>>> {
    paths.into_iter().map(|path| {
        std::thread::spawn(move || {
            load_obj(&path)
        })
    }).collect()
}
```

### Mesh Optimization

```rust
fn optimize_mesh(mesh_data: &mut MeshData) {
    // Remove duplicate vertices
    mesh_data.deduplicate_vertices();
    
    // Generate vertex cache-friendly index order
    mesh_data.optimize_vertex_cache();
    
    // Calculate tangents if needed
    if mesh_data.tangents.is_empty() {
        mesh_data.calculate_tangents();
    }
}
```

### LOD (Level of Detail)

```rust
#[derive(Component)]
struct LodMeshes {
    high: MeshHandle,
    medium: MeshHandle,
    low: MeshHandle,
}

fn select_lod(
    camera: Query<&Transform, With<Camera>>,
    mut objects: Query<(&Transform, &LodMeshes, &mut MeshHandle)>,
) {
    let camera_pos = camera.single().translation;
    
    for (transform, lods, mut mesh) in objects.iter_mut() {
        let distance = transform.translation.distance(camera_pos);
        
        *mesh = if distance < 10.0 {
            lods.high.clone()
        } else if distance < 50.0 {
            lods.medium.clone()
        } else {
            lods.low.clone()
        };
    }
}
```

## Debugging

### Log Asset Info

```rust
fn log_gltf_info(asset: &GltfAsset) {
    tracing::info!("GLTF Asset Info:");
    tracing::info!("  Meshes: {}", asset.meshes.len());
    tracing::info!("  Materials: {}", asset.materials.len());
    tracing::info!("  Textures: {}", asset.textures.len());
    tracing::info!("  Nodes: {}", asset.nodes.len());
    tracing::info!("  Animations: {}", asset.animations.len());
    
    for (i, mesh) in asset.meshes.iter().enumerate() {
        tracing::info!("  Mesh {}: {} vertices", i, mesh.vertices.len());
    }
}
```

### Visualize Mesh Bounds

```rust
fn debug_draw_mesh_bounds(
    query: Query<(&Transform, &MeshHandle)>,
    mesh_manager: Res<MeshAssetManager>,
    mut debug_lines: ResMut<DebugLines>,
) {
    for (transform, mesh_handle) in query.iter() {
        if let Some(mesh) = mesh_manager.get_mesh(mesh_handle.id()) {
            let bounds = mesh.calculate_bounds();
            debug_lines.box_bounds(transform.translation, bounds, Color::YELLOW);
        }
    }
}
```

## Troubleshooting

### OBJ File Not Loading

**Problem**: `Failed to load OBJ file`

**Solutions**:
- Verify file path is correct
- Check file exists and is readable
- Ensure OBJ file is valid (use Blender to re-export)
- Check vertex count is ≤ 65,535

### GLTF Meshes Not Appearing

**Problem**: GLTF loads but meshes don't render

**Solutions**:
- Verify meshes are uploaded to GPU
- Check node transforms are correct
- Ensure material/texture indices are valid
- Confirm entities have both Transform and MeshHandle

### Animations Not Working

**Problem**: GLTF animations don't play

**Solutions**:
- Verify GLTF has animations: `asset.animations.len() > 0`
- Check skeleton is extracted from skins
- Ensure AnimationPlayer component is added
- Add animation update system to schedule

### Out of Memory

**Problem**: Loading many assets causes OOM

**Solutions**:
- Load assets on-demand instead of all at once
- Implement asset streaming for large worlds
- Use texture compression
- Unload unused assets

## Examples

See working examples:
- `examples/obj_loader_demo.rs` - OBJ loading
- `examples/gltf_loader_demo.rs` - GLTF loading
- `examples/gltf_animation_loader_demo.rs` - Animated GLTF

Run with:
```bash
cargo run --example obj_loader_demo
```

## See Also

- [Mesh API Reference](../../reference/mesh-api.md) - Mesh architecture
- [Animation Guide](../animation.md) - Loading animated GLTF files
- [Assets Learning Path](../../learning-paths/assets.md) - Structured learning progression
- [praxis_assets Crate](../../../crates/praxis_assets/README.md) - Crate documentation
- [OBJ Format Specification](http://www.martinreddy.net/gfx/3d/OBJ.spec) - OBJ format details
- [GLTF Specification](https://registry.khronos.org/glTF/specs/2.0/glTF-2.0.html) - GLTF format
