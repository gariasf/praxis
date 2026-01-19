# Assets Learning Path

Master asset loading, management, and the content pipeline.

## Path Overview

**Time Investment**: 5-7 days  
**Prerequisites**: Basic file I/O understanding, tokio basics (for async section)  
**Final Goal**: Efficient asset pipeline with async loading

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
Advanced (2-3 days)
├── Custom asset loaders
├── Asset hot-reload
├── Async asset loading with tokio
├── Channel-based completion notification
├── Concurrent load management
└── Asset streaming and cancellation
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
1. Read [Assets Guide: GLTF Section](../guides/assets.md)
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

**Practice** (8-10 hours):
1. Create custom asset loader
2. Implement hot-reload
3. Build asset processing
4. Implement async asset loading

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

**Async Asset Loading**:

Non-blocking asset loading with tokio and channel-based completion:

```rust
use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};

// Create async loader
let loader = AsyncMeshLoader::new();

// Start loading asynchronously
let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;

// Do other work while loading...
println!("Loading in background: {}", handle.path().display());

// Non-blocking check
if let Ok(result) = receiver.try_recv() {
    match result {
        Ok(mesh_data) => println!("Loaded {} vertices", mesh_data.positions.len()),
        Err(e) => eprintln!("Load failed: {}", e),
    }
} else {
    println!("Still loading...");
}

// Or wait for completion (blocking)
let mesh_data = receiver.recv().unwrap()?;
```

**Multiple Concurrent Loads**:

```rust
use praxis_assets::async_loader::{AsyncBatchLoader, AsyncMeshLoader};

let loader = AsyncMeshLoader::new();
let mut batch = AsyncBatchLoader::new();

// Queue multiple loads
batch.add(loader.load_async("models/player.obj").await?);
batch.add(loader.load_async("models/enemy.obj").await?);
batch.add(loader.load_async("models/prop.obj").await?);

// Check progress in game loop
while !batch.is_complete() {
    let completed = batch.try_receive_completed();
    for result in completed {
        match result {
            Ok(mesh) => println!("Asset loaded!"),
            Err(e) => eprintln!("Failed: {}", e),
        }
    }
    
    println!("Progress: {}/{}", 
        batch.completed_count(), 
        batch.total_count()
    );
    
    // Update game while loading
    tokio::time::sleep(Duration::from_millis(16)).await;
}
```

**Load Cancellation**:

```rust
// Start loading
let (handle, receiver) = loader.load_async("large_asset.obj").await?;

// Cancel if user changes scene
if scene_changed {
    handle.cancel();
}

// Or use with timeout
tokio::select! {
    result = async { receiver.recv().unwrap() } => {
        println!("Loaded in time!");
    }
    _ = tokio::time::sleep(Duration::from_secs(5)) => {
        handle.cancel();
        println!("Load timed out");
    }
}
```

### Exercises

**Exercise 1: Basic Async Loading**
- Load 3 meshes asynchronously
- Print status while loading
- Verify all complete successfully
- Compare load times with synchronous loading

**Exercise 2: Game Level Loader**
- Create a level with 10+ assets
- Load all assets concurrently
- Display loading progress bar
- Handle individual load failures gracefully

**Exercise 3: Streaming System**
- Implement priority-based loading queue
- Load high-priority assets first
- Stream in background assets while playing
- Cancel loads when assets move out of range

**Exercise 4: Channel-Based Integration**
- Create a loading screen with progress updates
- Use channels to send completion notifications to main thread
- Update UI in real-time as assets complete
- Handle errors with user-friendly messages

**Exercise 5: Resource Budget System**
- Track total memory used by loaded assets
- Load assets until memory budget reached
- Unload oldest assets when over budget
- Use async loading to check file sizes before loading

### Checkpoint
- [ ] Created custom loader
- [ ] Hot-reload working
- [ ] Asset pipeline established
- [ ] Async loading with tokio working
- [ ] Channel-based completion notifications implemented
- [ ] Multiple concurrent loads managed
- [ ] Load cancellation working

**Time**: 10-12 hours

---

## Cross-References

- [Rendering Path](rendering.md) - Use loaded assets in rendering
- [Animation Path](animation.md) - Load skeletal animations from GLTF
- [Audio Path](audio.md) - Load audio files and sound banks

---

[← Back to Learning Paths](README.md)
