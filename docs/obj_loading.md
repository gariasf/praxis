# OBJ File Loading Implementation

This document describes the OBJ file loading system in the Praxis engine.

## Overview

The OBJ loading system provides a flexible, trait-based architecture for loading 3D mesh assets from Wavefront OBJ files. It integrates seamlessly with the existing mesh management system for automatic GPU upload.

## Architecture

### Core Components

1. **`AssetLoader<T>` Trait** (`praxis_assets::loader`)
   - Generic trait for loading any asset type
   - Provides a common interface for all asset loaders
   - Can be extended to support textures, audio, etc.

2. **`MeshLoader`** (`praxis_assets::loader`)
   - Concrete implementation of `AssetLoader<MeshData>`
   - Uses the `tobj` crate for OBJ parsing
   - Supports positions, normals, and texture coordinates

3. **Integration Functions** (`praxis_assets`)
   - `load_obj_mesh()` - High-level convenience function
   - `load_obj()` - Load without GPU upload
   - Direct integration with `MeshAssetManager`

### Data Flow

```
OBJ File
    ↓
MeshLoader::load()
    ↓
MeshData (CPU-side)
    ↓
MeshAssetManager::load_mesh()
    ↓
GpuMesh (GPU buffers)
    ↓
Rendering
```

## Usage

### Method 1: High-Level Convenience Function

The simplest way to load and use an OBJ mesh:

```rust
use praxis_assets::load_obj_mesh;
use praxis_graphics::RenderContext;

fn init(render_context: &mut RenderContext) -> praxis_utils::Result<()> {
    // Load and upload in one call
    load_obj_mesh(
        render_context.mesh_manager_mut(),
        "my_model",
        "assets/models/spaceship.obj"
    )?;
    
    Ok(())
}
```

### Method 2: Using AssetLoader Trait

More flexible approach with explicit loader instance:

```rust
use praxis_assets::{AssetLoader, MeshLoader};

fn load_mesh(render_context: &mut RenderContext) -> praxis_utils::Result<()> {
    let loader = MeshLoader::new();
    
    // Load the mesh data
    let mesh_data = loader.load("assets/models/cube.obj")?;
    
    // Upload to GPU
    render_context
        .mesh_manager_mut()
        .load_mesh("cube", mesh_data)?;
    
    Ok(())
}
```

### Method 3: Load for Processing

Load mesh data without immediate GPU upload for processing:

```rust
use praxis_assets::load_obj;

fn process_mesh() -> praxis_utils::Result<()> {
    // Load mesh data
    let mut mesh_data = load_obj("assets/models/model.obj")?;
    
    // Inspect or modify mesh data
    println!("Vertices: {}", mesh_data.positions.len());
    println!("Has normals: {}", mesh_data.normals.is_some());
    
    // Process mesh_data...
    // (e.g., calculate normals, optimize, combine with other meshes)
    
    // Later: upload to GPU when ready
    // render_context.mesh_manager_mut().load_mesh("processed", mesh_data)?;
    
    Ok(())
}
```

## Supported OBJ Features

### Supported

- ✅ Vertex positions (`v`)
- ✅ Vertex normals (`vn`)
- ✅ Texture coordinates (`vt`)
- ✅ Face definitions (`f`)
- ✅ Automatic triangulation
- ✅ Single index format (automatic conversion)

### Not Supported

- ❌ Material definitions (`.mtl` files)
- ❌ Vertex colors (not in OBJ spec)
- ❌ Multiple objects per file (only first is loaded)
- ❌ Quads or polygons (must use triangulation)
- ❌ Groups (merged into single mesh)

## File Format Requirements

### Basic Requirements

1. **Positions**: Must be present (required)
2. **Indices**: Faces must reference valid vertices
3. **Triangulation**: All faces should be triangles (or use `triangulate: true`)
4. **Vertex Count**: Must be ≤ 65,535 (u16 index limit)

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

## Implementation Details

### tobj Configuration

The `MeshLoader` uses these `tobj` options:

```rust
tobj::LoadOptions {
    triangulate: true,      // Convert quads/polygons to triangles
    single_index: true,     // Use single index buffer
    ..Default::default()
}
```

### Index Conversion

OBJ files use `u32` indices, but the engine uses `u16` for compatibility. The loader validates that no index exceeds `u16::MAX` and returns an error if it does.

```rust
let indices: Vec<u16> = mesh.indices
    .iter()
    .map(|&i| {
        if i > u16::MAX as u32 {
            Err(eyre::eyre!("Mesh has too many vertices for u16 indices"))
        } else {
            Ok(i as u16)
        }
    })
    .collect::<Result<Vec<_>>>()?;
```

### Color Handling

OBJ files don't support per-vertex colors. When converting to `Vertex3D`, the loader sets all vertices to white `[1.0, 1.0, 1.0]` by leaving `MeshData.colors` as `None`.

## Error Handling

All loading functions return `Result<T>` with descriptive errors:

```rust
// File not found
Err("Failed to load OBJ file 'path.obj': No such file or directory")

// Empty file
Err("OBJ file 'path.obj' contains no models")

// Too many vertices
Err("Mesh has too many vertices for u16 indices (vertex index: 100000)")

// Invalid format
Err("Failed to load OBJ file 'path.obj': Invalid vertex format")
```

## Performance Considerations

### Loading Performance

- File I/O is performed synchronously
- For large files (>10MB), consider loading on a background thread
- `tobj` is optimized for fast parsing

### GPU Upload

- Meshes are uploaded to GPU immediately when `load_mesh()` is called
- For many meshes, batch loading before entering render loop
- Each mesh creates separate vertex/index buffers

### Memory Usage

- Mesh data is duplicated during upload (CPU + GPU copy)
- CPU copy can be dropped after upload
- For 1000 vertices: ~24KB CPU + ~24KB GPU = 48KB total

## Example: obj_loader_demo

The `obj_loader_demo` example demonstrates all three loading methods:

```bash
cargo run --example obj_loader_demo
```

This example:
1. Loads the same OBJ file three times using different methods
2. Displays mesh statistics (vertices, indices, attributes)
3. Renders the loaded meshes with different rotations
4. Shows fallback handling if loading fails

## Integration with MeshAssetManager

The `MeshAssetManager` provides:

- **Caching**: Meshes are stored by string ID
- **Replacement**: Loading same ID replaces old mesh
- **Query**: Check if mesh exists before loading
- **Cleanup**: Remove unused meshes

```rust
// Check before loading
if !render_context.mesh_manager().contains_mesh("model") {
    load_obj_mesh(render_context.mesh_manager_mut(), "model", "path.obj")?;
}

// Get mesh stats
let count = render_context.mesh_manager().mesh_count();
println!("Total meshes loaded: {}", count);

// Remove when done
render_context.mesh_manager_mut().remove_mesh("model");
```

## Future Enhancements

Potential future improvements:

1. **Async Loading**: Load files on background threads
2. **Material Support**: Parse `.mtl` files for materials
3. **Multi-mesh Support**: Load all objects from OBJ file
4. **Optimization**: Automatic vertex deduplication
5. **Compression**: Support compressed mesh formats
6. **Streaming**: Load large meshes progressively
7. **Validation**: More robust error checking and validation

## Testing

Test the OBJ loader with:

```bash
# Unit tests
cargo test -p praxis_assets

# Integration test with example
cargo run --example obj_loader_demo
```

## Dependencies

The OBJ loading system uses:

- `tobj 4.0` - OBJ/MTL parsing
- `praxis_utils` - Error handling, logging
- `praxis_graphics` - MeshData, MeshAssetManager integration

## See Also

- [Mesh System Documentation](mesh_system.md)
- [Asset Management Architecture](../crates/praxis_assets/src/lib.rs)
- [tobj crate documentation](https://docs.rs/tobj)
