# Assets Learning Path

Master asset loading, management, and the content pipeline.

## Path Overview

**Time Investment**: 4-6 days  
**Prerequisites**: Basic file I/O understanding  
**Final Goal**: Efficient asset pipeline

## Progression Map

```
Beginner (2 days)
├── Loading meshes (OBJ, GLTF)
├── Loading textures
├── Loading audio
└── Asset handles
    ↓
Intermediate (2 days)
├── GLTF scenes
├── Skeletal meshes
├── Animations from GLTF
└── Asset management patterns
    ↓
Advanced (2 days)
├── Custom asset loaders
├── Asset hot-reload
├── Asset streaming
└── Build pipelines
```

---

## Beginner: Loading Assets

**Practice** (4-6 hours):
1. Read [Assets Guide](../guides/assets.md)
2. Load basic assets

**Loading Meshes**:
```rust
use praxis_assets::{ObjLoader, GltfLoader};

// OBJ mesh
let mesh = ObjLoader::load("models/cube.obj")?;
mesh_manager.add_mesh("cube", mesh)?;

// GLTF mesh
let gltf = GltfLoader::load("models/character.gltf")?;
for (name, mesh) in gltf.meshes {
    mesh_manager.add_mesh(&name, mesh)?;
}
```

**Loading Textures**:
```rust
let texture = texture_manager.load("textures/brick.png")?;
```

**Loading Audio**:
```rust
let sound = audio_manager.load("sounds/jump.ogg")?;
```

**Asset Handles**:
```rust
// Store reference, not the asset itself
#[derive(Component)]
struct MeshHandle {
    id: String,  // Reference to mesh in manager
}

// Retrieve when needed
let mesh = mesh_manager.get(&handle.id)?;
```

### Checkpoint
- [ ] Can load meshes, textures, audio
- [ ] Understand asset managers
- [ ] Using handles correctly

**Time**: 6-8 hours

---

## Intermediate: Advanced Loading

**Practice** (6-8 hours):
1. Read [GLTF Loading](../gltf-loading.md)
2. Load complex scenes

**GLTF Scenes**:
```rust
let scene = GltfLoader::load_scene("scenes/level.gltf")?;

for node in scene.nodes {
    world.spawn((
        Transform::from_matrix(node.transform),
        MeshHandle::new(&node.mesh),
        node.name,
    ));
}
```

**Skeletal Meshes**:
```rust
let gltf = GltfLoader::load("characters/hero.gltf")?;

// Skeleton
let skeleton = gltf.skeleton?;

// Animations
for (name, clip) in gltf.animations {
    animation_manager.add_clip(&name, clip);
}

// Skinned mesh
world.spawn((
    Transform::default(),
    SkinnedMesh::new(gltf.mesh, skeleton),
    AnimationPlayer::new(),
));
```

### Checkpoint
- [ ] Load complete GLTF scenes
- [ ] Import skeletal animations
- [ ] Manage complex asset graphs

**Time**: 8-10 hours

---

## Advanced: Custom Pipeline

**Practice** (6-8 hours):
1. Create custom asset loader
2. Implement hot-reload
3. Build asset processing

**Custom Loader**:
```rust
pub struct CustomLoader;

impl AssetLoader<CustomAsset> for CustomLoader {
    fn load(&self, path: impl AsRef<Path>) -> Result<CustomAsset> {
        let data = std::fs::read(path)?;
        // Parse custom format
        Ok(CustomAsset::from_bytes(&data)?)
    }

    fn extensions(&self) -> &[&str] {
        &["custom"]
    }
}

// Register
asset_manager.register_loader(CustomLoader);
```

**Hot-Reload**:
```rust
// Watch for file changes
asset_manager.enable_hot_reload()?;

// In game loop
asset_manager.check_for_changes()?;
// Assets automatically reload!
```

### Checkpoint
- [ ] Created custom loader
- [ ] Hot-reload working
- [ ] Asset pipeline established

**Time**: 8-10 hours

---

## Cross-References

- [Rendering Path](rendering.md) - Use loaded assets
- [Animation Path](animation.md) - Load animations
- [Procedural Textures](../procedural-textures.md) - Generate at runtime

---

[← Back to Learning Paths](README.md)
