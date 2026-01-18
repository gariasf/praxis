# Praxis Assets

Asset loading and management for the Praxis game engine.

## Overview

Synchronous and asynchronous loading for OBJ, GLTF meshes, textures, and animations.

**Key Features:**
- OBJ mesh loading with automatic triangulation
- GLTF 2.0 loading (meshes, materials, textures, hierarchies)
- Async asset loading with progress tracking
- Batch loading for multiple assets
- Asset caching (GLTF)
- Direct GPU integration

## Quick Start

### OBJ Loading

```rust
use praxis_assets::load_obj_mesh;

// Load and upload to GPU
load_obj_mesh(mesh_manager, "cube", "assets/models/cube.obj")?;
```

### GLTF Loading

```rust
use praxis_assets::{GltfLoader, GltfAssetManager};

// Direct loading
let loader = GltfLoader::new();
let asset = loader.load_gltf("assets/models/scene.gltf")?;

// Cached loading
let mut manager = GltfAssetManager::new();
let asset = manager.load("assets/models/scene.gltf")?;

// Upload meshes
for (i, mesh) in asset.meshes.iter().enumerate() {
    render_context.mesh_manager_mut()
        .load_mesh(format!("mesh_{}", i), mesh.clone())?;
}
```

### Async Loading

```rust
use praxis_assets::async_loader::{AsyncMeshLoader, AsyncBatchLoader};

let loader = AsyncMeshLoader::new();
let (handle, receiver) = loader.load_async("cube.obj").await?;

// Do other work...

let mesh = receiver.recv().unwrap()?;
```

## Supported Formats

**OBJ:** Positions, normals, UVs, faces (automatic triangulation)

**GLTF:** Meshes, PBR materials, textures (PNG/JPEG), scene hierarchies, animations (via `praxis_scene`)

## Documentation

**Comprehensive Guides:**
- [Assets Overview](../../docs/guides/assets/README.md)
- [OBJ Loading](../../docs/guides/assets/obj.md)
- [GLTF Loading](../../docs/guides/assets/gltf.md)
- [Async Assets](../../docs/guides/async-assets.md)

**Learning Path:**
- [Assets Learning Path](../../docs/learning-paths/assets.md)

**Reference:**
- [Mesh API Reference](../../docs/reference/mesh-api.md)

## Examples

```bash
cargo run --example obj_loader_demo
cargo run --example gltf_loader_demo
cargo run --example gltf_animation_loader_demo
```

## Dependencies

- `tobj` 4.0: OBJ parsing
- `gltf` 1.4: GLTF 2.0 parsing
- `tokio` 1.40: Async runtime
