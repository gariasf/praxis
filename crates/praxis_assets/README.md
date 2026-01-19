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
use praxis_graphics::MeshManager;
use color_eyre::Result;

fn load_cube(mesh_manager: &mut MeshManager) -> Result<()> {
    // Load OBJ file and upload to GPU
    load_obj_mesh(
        mesh_manager,
        "cube",                     // Mesh name
        "assets/models/cube.obj"    // File path
    )?;
    
    Ok(())
}
```

### GLTF Loading

```rust
use praxis_assets::{GltfLoader, GltfAssetManager};
use praxis_graphics::RenderContext;
use color_eyre::Result;

fn load_gltf_scene(
    render_context: &mut RenderContext
) -> Result<()> {
    // Option 1: Direct loading
    let loader = GltfLoader::new();
    let asset = loader.load_gltf("assets/models/scene.gltf")?;
    
    // Upload all meshes to GPU
    for (i, mesh) in asset.meshes.iter().enumerate() {
        render_context.mesh_manager_mut()
            .load_mesh(format!("mesh_{}", i), mesh.clone())?;
    }
    
    Ok(())
}

fn load_gltf_cached() -> Result<()> {
    // Option 2: Cached loading (reuses loaded assets)
    let mut manager = GltfAssetManager::new();
    let asset = manager.load("assets/models/scene.gltf")?;
    
    // Asset is cached; subsequent calls return cached version
    let same_asset = manager.load("assets/models/scene.gltf")?;
    
    Ok(())
}
```

### Async Loading

```rust
use praxis_assets::async_loader::{AsyncMeshLoader, AsyncBatchLoader};
use color_eyre::Result;

#[tokio::main]
async fn async_load_example() -> Result<()> {
    // Single asset async loading
    let loader = AsyncMeshLoader::new();
    let (handle, receiver) = loader.load_async("cube.obj").await?;
    
    // Do other work while loading...
    println!("Loading in background...");
    
    // Wait for completion
    let mesh = receiver.recv()
        .expect("Channel should not be closed")?;
    
    println!("Mesh loaded with {} vertices", mesh.vertices.len());
    
    Ok(())
}

#[tokio::main]
async fn batch_load_example() -> Result<()> {
    // Batch loading multiple assets
    let batch_loader = AsyncBatchLoader::new();
    
    let assets = vec![
        "cube.obj",
        "sphere.obj",
        "cylinder.obj",
    ];
    
    let (handles, receiver) = batch_loader.load_batch(assets).await?;
    
    // Receive loaded assets as they complete
    for result in receiver.iter() {
        match result {
            Ok(mesh) => println!("Loaded mesh with {} vertices", mesh.vertices.len()),
            Err(e) => eprintln!("Failed to load mesh: {}", e),
        }
    }
    
    Ok(())
}
```

### Loading GLTF with Animations

```rust
use praxis_assets::GltfLoader;
use praxis_scene::{AnimationPlayer, Skeleton, AnimatedPose};
use praxis_ecs::World;
use color_eyre::Result;

fn load_animated_character(world: &mut World) -> Result<()> {
    let loader = GltfLoader::new();
    let asset = loader.load_gltf("assets/models/character.gltf")?;
    
    // Extract skeleton from first skin
    if let Some(skin) = asset.skins.first() {
        let skeleton = skin.skeleton.clone();
        
        // Create animation player and add all clips
        let mut player = AnimationPlayer::new();
        for animation in &asset.animations {
            let name = animation.name.clone()
                .unwrap_or_else(|| "unnamed".to_string());
            player.add_clip(name, animation.clip.clone());
        }
        
        // Create animated pose
        let pose = AnimatedPose::new(skeleton.bone_count());
        
        // Spawn entity with animation components
        world.spawn((
            skeleton,
            player,
            pose,
        ));
    }
    
    Ok(())
}
```

## Supported Formats

**OBJ:**
- Vertex positions
- Normals
- Texture coordinates (UVs)
- Faces with automatic triangulation

**GLTF 2.0:**
- Meshes with multiple primitives
- PBR materials (metallic/roughness workflow)
- Textures (PNG/JPEG)
- Scene hierarchies
- Skeletal animations (via `praxis_scene`)

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
# OBJ loading example
cargo run --example obj_loader_demo

# GLTF loading example
cargo run --example gltf_loader_demo

# GLTF animation loading
cargo run --example gltf_animation_loader_demo
```

## Dependencies

- `tobj` 4.0: OBJ parsing
- `gltf` 1.4: GLTF 2.0 parsing
- `tokio` 1.40: Async runtime
