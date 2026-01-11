# Praxis Assets

Asset loading, management, and caching for the Praxis game engine.

## Features

- **OBJ File Loading**: Load Wavefront OBJ mesh files with automatic triangulation
- **GLTF/GLB File Loading**: Load GLTF 2.0 files with meshes, materials, textures, and node hierarchies
- **Async Asset Loading**: Non-blocking loading with tokio and channel-based completion notification
- **Asset Loader Trait**: Extensible trait-based architecture for any asset type
- **Asset Caching**: `GltfAssetManager` provides caching for GLTF assets
- **Batch Loading**: Load multiple assets concurrently with progress tracking
- **Seamless Integration**: Direct integration with MeshAssetManager for GPU upload
- **Error Handling**: Comprehensive error reporting for file I/O and parsing

## OBJ File Loading

### Quick Start

The simplest way to load an OBJ mesh:

```rust
use praxis_assets::load_obj_mesh;
use praxis_graphics::RenderContext;

fn init(render_context: &mut RenderContext) -> praxis_utils::Result<()> {
    // Load and upload in one call
    load_obj_mesh(
        render_context.mesh_manager_mut(),
        "spaceship",
        "assets/models/spaceship.obj"
    )?;
    
    Ok(())
}
```

### Three Loading Methods

**Method 1: High-level convenience function**

```rust
use praxis_assets::load_obj_mesh;

load_obj_mesh(mesh_manager, "model_id", "path/to/model.obj")?;
```

**Method 2: Using AssetLoader trait**

```rust
use praxis_assets::{AssetLoader, MeshLoader};

let loader = MeshLoader::new();
let mesh_data = loader.load("path/to/model.obj")?;
mesh_manager.load_mesh("model_id", mesh_data)?;
```

**Method 3: Load for processing**

```rust
use praxis_assets::load_obj;

let mut mesh_data = load_obj("path/to/model.obj")?;
// Process mesh_data (calculate normals, optimize, etc.)
mesh_manager.load_mesh("model_id", mesh_data)?;
```

## Supported OBJ Features

### ✅ Supported

- Vertex positions (`v`)
- Vertex normals (`vn`)
- Texture coordinates (`vt`)
- Face definitions (`f`)
- Automatic triangulation
- Single index format (automatic conversion)

### ❌ Not Supported

- Material definitions (`.mtl` files)
- Multiple objects per file (only first is loaded)
- Vertex colors (not in OBJ spec)

## GLTF File Loading

### Quick Start

Load a GLTF file with all its data:

```rust
use praxis_assets::{GltfLoader, GltfAssetManager};

// Direct loading
let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/scene.gltf")?;

println!("Loaded {} meshes", asset.meshes.len());
println!("Loaded {} materials", asset.materials.len());
println!("Loaded {} textures", asset.textures.len());

// Cached loading with manager
let mut manager = GltfAssetManager::new();
let asset = manager.load("assets/models/scene.gltf")?;
```

### Supported GLTF Features

#### ✅ Supported

- **Meshes**
  - Vertex positions (required)
  - Vertex normals
  - Texture coordinates (UV)
  - Tangent vectors
  - Triangle primitives
  - Multiple primitives per mesh
  
- **Materials**
  - PBR metallic-roughness workflow
  - Base color factor
  - Metallic factor
  - Roughness factor
  - Base color texture
  - Normal map texture
  
- **Textures**
  - Embedded images
  - External image files
  - PNG and JPEG formats
  - RGBA and RGB formats
  
- **Scene Hierarchy**
  - Node transforms (matrix form)
  - Parent-child relationships
  - Multiple root nodes
  - Named nodes
  - Mesh references

#### ❌ Not Supported

- Animations (use `praxis_scene` for animations)
- Skins/skeletal animation
- Morph targets
- Cameras (data loaded but not interpreted)
- Lights (data loaded but not interpreted)
- Extensions

### Working with GLTF Assets

#### Accessing Meshes

```rust
let asset = loader.load_gltf("model.gltf")?;

// Upload all meshes to GPU
for (i, mesh) in asset.meshes.iter().enumerate() {
    render_context
        .mesh_manager_mut()
        .load_mesh(format!("mesh_{}", i), mesh.clone())?;
}
```

#### Traversing the Scene Graph

```rust
// Find all nodes with meshes
for (node_index, node) in asset.nodes_with_meshes() {
    let mesh_index = node.mesh_index.unwrap();
    println!("Node {} has mesh {}", node_index, mesh_index);
}

// Depth-first traversal
asset.traverse_depth_first(|node_index, node, depth| {
    let indent = "  ".repeat(depth);
    println!("{}{}: {:?}", indent, node_index, node.name);
});
```

#### Using Materials

```rust
for material in &asset.materials {
    let props = material.to_material_properties();
    
    // Use with graphics system
    if let Some(tex_index) = material.base_color_texture_index {
        let texture = &asset.textures[tex_index];
        // Upload texture to GPU
    }
}
```

## Async Asset Loading

For non-blocking asset loading that doesn't block the main thread:

### Basic Async Loading

```rust
use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};

async fn load_async() -> praxis_utils::Result<()> {
    let loader = AsyncMeshLoader::new();
    
    // Start loading asynchronously
    let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
    
    // Do other work while loading...
    println!("Loading in background...");
    
    // Wait for completion
    let mesh_data = receiver.recv().unwrap()?;
    println!("Loaded {} vertices", mesh_data.positions.len());
    
    Ok(())
}
```

### Non-Blocking Check

```rust
use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};

async fn load_with_check() -> praxis_utils::Result<()> {
    let loader = AsyncMeshLoader::new();
    let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;
    
    // Check if ready (non-blocking)
    loop {
        match receiver.try_recv() {
            Ok(result) => {
                let mesh_data = result?;
                println!("Ready! Loaded {} vertices", mesh_data.positions.len());
                break;
            }
            Err(_) => {
                println!("Still loading...");
                // Do other work
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
    
    Ok(())
}
```

### Batch Loading

Load multiple assets concurrently:

```rust
use praxis_assets::async_loader::{AsyncBatchLoader, AsyncMeshLoader};

async fn batch_load() -> praxis_utils::Result<()> {
    let loader = AsyncMeshLoader::new();
    let mut batch = AsyncBatchLoader::new();
    
    // Queue multiple loads
    batch.add(loader.load_async("cube.obj").await?);
    batch.add(loader.load_async("sphere.obj").await?);
    batch.add(loader.load_async("cylinder.obj").await?);
    
    // Check progress
    println!("Loading: {}/{}", 
        batch.completed_count(), 
        batch.total_count());
    
    // Wait for all
    let results = batch.wait_all();
    println!("Loaded {} assets", results.len());
    
    Ok(())
}
```

### Async GLTF Loading

```rust
use praxis_assets::async_loader::{AsyncAssetLoader, AsyncGltfLoader};

async fn load_gltf_async() -> praxis_utils::Result<()> {
    let loader = AsyncGltfLoader::new();
    let (handle, receiver) = loader.load_async("assets/models/scene.gltf").await?;
    
    let asset = receiver.recv().unwrap()?;
    println!("Loaded {} meshes", asset.meshes.len());
    
    Ok(())
}
```

### Load Handle Operations

```rust
let (handle, receiver) = loader.load_async("model.obj").await?;

// Check if finished
if handle.is_finished() {
    println!("Loading complete!");
}

// Get path
println!("Loading: {}", handle.path().display());

// Cancel (best effort)
handle.cancel();
```

## AssetLoader Trait

The `AssetLoader<T>` trait provides a generic interface for loading any asset type:

```rust
pub trait AssetLoader<T> {
    fn load(&self, path: impl AsRef<Path>) -> Result<T>;
    fn supported_extensions(&self) -> &[&str];
}
```

The `AsyncAssetLoader<T>` trait provides async loading:

```rust
#[async_trait::async_trait]
pub trait AsyncAssetLoader<T>: Send + Sync {
    async fn load_async(
        &self,
        path: impl AsRef<Path> + Send,
    ) -> Result<(LoadHandle, Receiver<Result<T>>)>;
    
    async fn load_many_async(
        &self,
        paths: impl IntoIterator<Item = impl AsRef<Path> + Send> + Send,
    ) -> Result<Vec<(LoadHandle, Receiver<Result<T>>)>>;
}
```

These traits can be implemented for textures, audio, configurations, and other asset types.

## Error Handling

All loading functions return `Result<T>` with descriptive errors:

```rust
// File not found
Err("Failed to load OBJ file 'path.obj': No such file or directory")

// Empty file
Err("OBJ file 'path.obj' contains no models")

// Too many vertices
Err("Mesh has too many vertices for u16 indices")
```

## Performance Considerations

- **Synchronous Loading**: File I/O blocks the calling thread
- **Async Loading**: Uses tokio runtime and background threads for non-blocking I/O
- **GPU Upload**: Meshes are uploaded immediately when `load_mesh()` is called
- **Memory**: Mesh data is duplicated during upload (CPU + GPU copy)
- **Batching**: Load multiple meshes before entering render loop for best performance
- **Concurrent Loading**: `AsyncBatchLoader` can load multiple assets in parallel

## Examples

Run the asset loading demos:

```bash
# OBJ loading
cargo run --example obj_loader_demo

# GLTF loading
cargo run --example gltf_loader_demo

# Animation loading from GLTF
cargo run --example gltf_animation_loader_demo
```

## Testing

```bash
# Unit tests
cargo test -p praxis_assets

# Integration tests with examples
cargo run --example comprehensive_scene_demo
```

## Dependencies

- `tobj` 4.0: OBJ/MTL parsing
- `gltf` 1.4: GLTF 2.0 parsing with image loading
- `tokio` 1.40: Async runtime for non-blocking I/O
- `crossbeam-channel` 0.5: Thread-safe channels for completion notification
- `async-trait` 0.1: Async trait support
- `praxis_utils`: Error handling, logging
- `praxis_graphics`: MeshData, MeshAssetManager integration
- `praxis_math`: Matrix and vector math (Mat4, Vec3, Quat)
- `praxis_scene`: Scene hierarchy and animation support

## Future Enhancements

### OBJ Loader
- Material (`.mtl`) support
- Vertex deduplication optimization
- Streaming large files

### GLTF Loader
- Animation support (in progress via `praxis_scene`)
- Skeletal animation/skinning
- Morph targets
- KHR extensions support
- Sparse accessor support
- Streaming large files

### Async Loading
- Progress reporting with percentage
- Priority queues for load ordering
- Resource pools for limiting concurrent loads
- Hot-reloading/watch mode

## See Also

- [Assets Guide](../../docs/guides/assets/README.md)
- [GLTF Loading Guide](../../docs/guides/assets/gltf.md)
- [OBJ Loading Guide](../../docs/guides/assets/obj.md)
- [Async Assets Guide](../../docs/guides/async-assets.md)
- [Scene System](../praxis_scene/README.md)
- [Mesh API Reference](../../docs/reference/mesh-api.md)
