# Praxis Assets

Asset loading, management, and caching for the Praxis game engine.

## Features

- **OBJ File Loading**: Load Wavefront OBJ mesh files with automatic triangulation
- **Asset Loader Trait**: Extensible trait-based architecture for any asset type
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

## File Format Requirements

1. **Positions**: Must be present (required)
2. **Triangulation**: All faces must be triangles or use automatic triangulation
3. **Vertex Count**: Must be ≤ 65,535 (u16 index limit)

### Example OBJ File

```obj
# Simple triangle
v 0.0 1.0 0.0
v -1.0 -1.0 0.0
v 1.0 -1.0 0.0

vn 0.0 0.0 1.0

vt 0.5 1.0
vt 0.0 0.0
vt 1.0 0.0

f 1/1/1 2/2/1 3/3/1
```

## AssetLoader Trait

The `AssetLoader<T>` trait provides a generic interface for loading any asset type:

```rust
pub trait AssetLoader<T> {
    fn load(&self, path: impl AsRef<Path>) -> Result<T>;
    fn supported_extensions(&self) -> &[&str];
}
```

This trait can be implemented for textures, audio, configurations, and other asset types.

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

## Implementation Details

### Index Conversion

OBJ files use `u32` indices, but the engine uses `u16`. The loader validates that no index exceeds `u16::MAX`:

```rust
let indices: Vec<u16> = mesh.indices
    .iter()
    .map(|&i| {
        if i > u16::MAX as u32 {
            Err(eyre::eyre!("Mesh has too many vertices"))
        } else {
            Ok(i as u16)
        }
    })
    .collect::<Result<Vec<_>>>()?;
```

### Color Handling

OBJ files don't support per-vertex colors. Loaded meshes use default white color `[1.0, 1.0, 1.0]` for all vertices.

### tobj Configuration

```rust
tobj::LoadOptions {
    triangulate: true,      // Convert quads/polygons to triangles
    single_index: true,     // Use single index buffer
    ..Default::default()
}
```

## Performance Considerations

- **Loading**: File I/O is synchronous; consider background threads for large files (>10MB)
- **GPU Upload**: Meshes are uploaded immediately when `load_mesh()` is called
- **Memory**: Mesh data is duplicated during upload (CPU + GPU copy)
- **Batching**: Load multiple meshes before entering render loop for best performance

## Examples

See `examples/obj_loader_demo.rs` for a comprehensive demonstration:

```bash
cargo run --example obj_loader_demo
```

## Testing

```bash
# Unit tests
cargo test -p praxis_assets

# Integration test
cargo run --example obj_loader_demo
```

## Dependencies

- `tobj 4.0` - OBJ/MTL parsing
- `praxis_utils` - Error handling, logging
- `praxis_graphics` - MeshData, MeshAssetManager integration

## Future Enhancements

- Async/background loading
- Material (`.mtl`) support
- Multi-mesh loading per file
- Vertex deduplication optimization
- Compressed format support
- Progressive streaming for large meshes

## See Also

- [Mesh System Documentation](../../docs/mesh_system.md)
- [OBJ Loading Details](../../docs/obj_loading.md)
- [tobj crate documentation](https://docs.rs/tobj)
