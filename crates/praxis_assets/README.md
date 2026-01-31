# praxis_assets

Asset loading system for Praxis engine: OBJ, GLTF.

## Overview

Loads 3D models, textures, and other assets from disk. Supports OBJ and GLTF formats.

## Features

- **OBJ Loading**: Simple mesh format
- **GLTF Loading**: Complex scenes with animations, materials, textures
- **Async Loading**: Non-blocking asset loading
- **Texture Loading**: PNG, JPEG formats
- **Resource Management**: Asset caching and lifetime tracking

## Example

```rust
use praxis_assets::{AssetManager, ModelAsset};

let mut asset_manager = AssetManager::new();

// Load OBJ model
let model = asset_manager.load_obj("models/cube.obj").await?;

// Load GLTF scene
let scene = asset_manager.load_gltf("models/character.gltf").await?;

// Access meshes
for mesh in &model.meshes {
    // Upload to GPU
}
```

## GLTF Support

- Meshes with multiple primitives
- Materials (PBR metallic-roughness)
- Textures (base color, normal, metallic-roughness)
- Skeletal animations
- Scene hierarchy
- Node transforms

## Dependencies

- `gltf`: GLTF format parsing
- `tobj`: OBJ format parsing
- `image`: Texture loading
- `tokio`: Async runtime

## Usage

```toml
praxis_assets = { path = "../praxis_assets", version = "0.1.0" }
```
