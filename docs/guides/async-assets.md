# Async Asset Loading

Praxis provides non-blocking asset loading using tokio's async runtime and crossbeam channels. The `AsyncAssetLoader` trait and implementations (`AsyncMeshLoader`, `AsyncGltfLoader`) enable loading assets without freezing the game loop.

## Overview

Async asset loading solves the problem of frame hitches during asset I/O by moving loading operations to background tasks. Results are delivered through channels, allowing the main thread to poll for completion.

**Benefits:**
- **Non-blocking**: Game loop continues during asset load
- **Concurrent**: Load multiple assets simultaneously
- **Cancellable**: Cancel loading operations in progress
- **Progress tracking**: Monitor loading state
- **Efficient**: Uses tokio thread pool for I/O

**Architecture:**
- **`AsyncAssetLoader<T>`**: Core trait for async loading
- **`LoadHandle`**: Handle to track loading operation
- **`Receiver<Result<T>>`**: Channel to receive loaded asset
- **`AsyncBatchLoader`**: Manage multiple concurrent loads

## Basic Usage

### Loading a Single Asset

```rust
use praxis_assets::async_loader::{AsyncAssetLoader, AsyncMeshLoader};

// Create loader
let loader = AsyncMeshLoader::new();

// Start loading (returns immediately)
let (handle, receiver) = loader.load_async("assets/models/cube.obj").await?;

// Do other work while loading...
update_game_state();
render_frame();

// Check if ready (non-blocking)
match receiver.try_recv() {
    Ok(result) => {
        let mesh_data = result?;
        println!("Loaded {} vertices", mesh_data.positions.len());
    }
    Err(_) => {
        println!("Still loading...");
    }
}
```

### Blocking Wait for Completion

```rust
let (handle, receiver) = loader.load_async("cube.obj").await?;

// Block until complete
let mesh_data = receiver.recv().unwrap()?;
println!("Loaded!");
```

## Async Loaders

### Mesh Loader

Load OBJ mesh files asynchronously:

```rust
use praxis_assets::async_loader::AsyncMeshLoader;
use praxis_graphics::MeshData;

let loader = AsyncMeshLoader::new();

let (handle, receiver) = loader.load_async("assets/models/character.obj").await?;

// Non-blocking check
if let Ok(result) = receiver.try_recv() {
    let mesh: MeshData = result?;
    // Upload to GPU
    mesh_manager.upload_mesh("character", mesh)?;
}
```

### GLTF Loader

Load GLTF/GLB files asynchronously:

```rust
use praxis_assets::async_loader::AsyncGltfLoader;
use praxis_assets::loader::GltfAsset;

let loader = AsyncGltfLoader::new();

let (handle, receiver) = loader.load_async("assets/models/scene.gltf").await?;

let asset: GltfAsset = receiver.recv().unwrap()?;
println!("Loaded {} meshes", asset.meshes.len());
```

## Load Handles

Handles provide status tracking and cancellation:

```rust
let (handle, receiver) = loader.load_async("model.obj").await?;

// Check if finished
if handle.is_finished() {
    println!("Loading complete!");
}

// Get path
println!("Loading: {}", handle.path().display());

// Cancel operation
handle.cancel();

if handle.is_cancelled() {
    println!("Cancelled");
}
```

**Note**: Cancellation sets a flag but the I/O operation may complete anyway. The result is still sent to the channel.

## Concurrent Loading

### Loading Multiple Assets

```rust
let loader = AsyncMeshLoader::new();

// Start multiple loads
let (handle1, receiver1) = loader.load_async("cube.obj").await?;
let (handle2, receiver2) = loader.load_async("sphere.obj").await?;
let (handle3, receiver3) = loader.load_async("cylinder.obj").await?;

// All load concurrently in background

// Collect results
let mesh1 = receiver1.recv().unwrap()?;
let mesh2 = receiver2.recv().unwrap()?;
let mesh3 = receiver3.recv().unwrap()?;

println!("All assets loaded!");
```

### Batch Loading Helper

Use `load_many_async` for cleaner syntax:

```rust
let loader = AsyncMeshLoader::new();

let paths = vec!["cube.obj", "sphere.obj", "cylinder.obj"];

let loads = loader.load_many_async(paths).await?;

for (handle, receiver) in loads {
    let mesh_data = receiver.recv().unwrap()?;
    println!("Loaded: {}", handle.path().display());
}
```

## Batch Loader

The `AsyncBatchLoader` manages multiple concurrent loads with progress tracking:

```rust
use praxis_assets::async_loader::{AsyncBatchLoader, AsyncMeshLoader};

let mesh_loader = AsyncMeshLoader::new();
let mut batch = AsyncBatchLoader::new();

// Queue multiple loads
batch.add(mesh_loader.load_async("cube.obj").await?);
batch.add(mesh_loader.load_async("sphere.obj").await?);
batch.add(mesh_loader.load_async("cylinder.obj").await?);

// Track progress
println!("Loading: {}/{}", batch.completed_count(), batch.total_count());

// Wait for all
let results = batch.wait_all();
println!("Loaded {} assets", results.len());
```

### Non-Blocking Progress Tracking

Poll for completed assets without blocking:

```rust
let mut batch = AsyncBatchLoader::new();

// Add loads...
for path in asset_paths {
    batch.add(loader.load_async(path).await?);
}

// Game loop
loop {
    // Try to receive completed assets
    let completed = batch.try_receive_completed();
    
    for result in completed {
        let mesh_data = result?;
        // Process loaded asset
        mesh_manager.add(mesh_data);
    }
    
    // Update progress UI
    let progress = batch.completed_count() as f32 / batch.total_count() as f32;
    ui.show_loading_bar(progress);
    
    // Check if all done
    if batch.is_complete() {
        println!("All assets loaded!");
        break;
    }
    
    // Continue game logic
    update();
    render();
}
```

### Cancelling Batch

Cancel all pending loads:

```rust
batch.cancel_all();

// Handles are marked cancelled but may still complete
```

## Integration Patterns

### Loading Screen

Display loading progress while loading level assets:

```rust
struct LoadingScreen {
    batch: AsyncBatchLoader<MeshData>,
    total_assets: usize,
}

impl LoadingScreen {
    async fn load_level(&mut self, level: &Level) -> Result<()> {
        let loader = AsyncMeshLoader::new();
        
        // Queue all level assets
        for asset_path in &level.assets {
            self.batch.add(loader.load_async(asset_path).await?);
        }
        
        self.total_assets = self.batch.total_count();
        
        Ok(())
    }
    
    fn update(&mut self) -> LoadingState {
        // Try to receive completed assets
        let completed = self.batch.try_receive_completed();
        
        for result in completed {
            match result {
                Ok(mesh_data) => {
                    // Upload to GPU
                    self.mesh_manager.upload(mesh_data);
                }
                Err(e) => {
                    eprintln!("Failed to load asset: {}", e);
                }
            }
        }
        
        if self.batch.is_complete() {
            LoadingState::Complete
        } else {
            let progress = self.batch.completed_count() as f32 / self.total_assets as f32;
            LoadingState::Loading(progress)
        }
    }
    
    fn render(&self, ui: &mut egui::Ui) {
        let progress = self.batch.completed_count() as f32 / self.total_assets as f32;
        
        ui.heading("Loading...");
        ui.add(egui::ProgressBar::new(progress).show_percentage());
        ui.label(format!("{}/{} assets", 
            self.batch.completed_count(), 
            self.total_assets
        ));
    }
}

enum LoadingState {
    Loading(f32), // Progress 0.0-1.0
    Complete,
}
```

### Lazy Loading

Load assets on-demand when first referenced:

```rust
use std::collections::HashMap;
use parking_lot::RwLock;

struct LazyAssetManager {
    loader: AsyncMeshLoader,
    pending: Arc<RwLock<HashMap<String, Receiver<Result<MeshData>>>>>,
    loaded: Arc<RwLock<HashMap<String, MeshData>>>,
}

impl LazyAssetManager {
    fn new() -> Self {
        Self {
            loader: AsyncMeshLoader::new(),
            pending: Arc::new(RwLock::new(HashMap::new())),
            loaded: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn request(&self, path: &str) -> Result<()> {
        let path = path.to_string();
        
        // Check if already loaded
        if self.loaded.read().contains_key(&path) {
            return Ok(());
        }
        
        // Check if already loading
        if self.pending.read().contains_key(&path) {
            return Ok(());
        }
        
        // Start loading
        let (handle, receiver) = self.loader.load_async(&path).await?;
        self.pending.write().insert(path.clone(), receiver);
        
        Ok(())
    }
    
    fn update(&self) {
        let mut pending = self.pending.write();
        let mut loaded = self.loaded.write();
        
        pending.retain(|path, receiver| {
            match receiver.try_recv() {
                Ok(Ok(mesh_data)) => {
                    // Asset loaded successfully
                    loaded.insert(path.clone(), mesh_data);
                    false // Remove from pending
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to load {}: {}", path, e);
                    false // Remove from pending
                }
                Err(_) => {
                    true // Still loading, keep in pending
                }
            }
        });
    }
    
    fn get(&self, path: &str) -> Option<MeshData> {
        self.loaded.read().get(path).cloned()
    }
    
    fn is_loaded(&self, path: &str) -> bool {
        self.loaded.read().contains_key(path)
    }
    
    fn is_loading(&self, path: &str) -> bool {
        self.pending.read().contains_key(path)
    }
}
```

Usage:

```rust
// Request asset
asset_manager.request("models/enemy.obj").await?;

// Later, in game loop
asset_manager.update();

// Check if available
if let Some(mesh) = asset_manager.get("models/enemy.obj") {
    // Use mesh
    render_enemy(&mesh);
} else if asset_manager.is_loading("models/enemy.obj") {
    // Show loading indicator
    render_placeholder();
}
```

### Streaming World

Stream in world chunks as player moves:

```rust
struct StreamingWorld {
    loader: AsyncMeshLoader,
    active_chunks: HashMap<ChunkCoord, ChunkData>,
    loading_chunks: HashMap<ChunkCoord, Receiver<Result<MeshData>>>,
    player_position: Vec3,
}

impl StreamingWorld {
    async fn update(&mut self) -> Result<()> {
        let player_chunk = self.world_to_chunk(self.player_position);
        
        // Determine required chunks (3x3 around player)
        let mut required_chunks = HashSet::new();
        for x in -1..=1 {
            for z in -1..=1 {
                required_chunks.insert(ChunkCoord {
                    x: player_chunk.x + x,
                    z: player_chunk.z + z,
                });
            }
        }
        
        // Start loading missing chunks
        for coord in &required_chunks {
            if !self.active_chunks.contains_key(coord) 
                && !self.loading_chunks.contains_key(coord) {
                self.start_loading_chunk(*coord).await?;
            }
        }
        
        // Process completed loads
        self.loading_chunks.retain(|coord, receiver| {
            match receiver.try_recv() {
                Ok(Ok(mesh_data)) => {
                    self.active_chunks.insert(*coord, ChunkData { mesh_data });
                    false // Remove from loading
                }
                Ok(Err(e)) => {
                    eprintln!("Failed to load chunk {:?}: {}", coord, e);
                    false
                }
                Err(_) => true, // Still loading
            }
        });
        
        // Unload distant chunks
        self.active_chunks.retain(|coord, _| {
            required_chunks.contains(coord)
        });
        
        Ok(())
    }
    
    async fn start_loading_chunk(&mut self, coord: ChunkCoord) -> Result<()> {
        let path = format!("chunks/chunk_{}_{}.obj", coord.x, coord.z);
        let (_, receiver) = self.loader.load_async(&path).await?;
        self.loading_chunks.insert(coord, receiver);
        Ok(())
    }
}
```

## Performance Considerations

### Tokio Runtime Overhead

Starting async operations has minimal overhead:

```rust
// Spawning 100 loads takes <100ms
let start = Instant::now();
for i in 0..100 {
    loader.load_async(format!("asset_{}.obj", i)).await?;
}
let elapsed = start.elapsed();
// Typical: 50-100ms
```

### Concurrent Load Limits

Tokio uses a thread pool (default: number of CPU cores):

```rust
// Too many concurrent I/O operations can saturate disk
// For HDDs, limit concurrent loads to 2-4
// For SSDs, 8-16 concurrent loads is fine

let semaphore = Arc::new(tokio::sync::Semaphore::new(8));

for path in paths {
    let permit = semaphore.clone().acquire_owned().await?;
    tokio::spawn(async move {
        let result = load_asset(path).await;
        drop(permit); // Release semaphore
        result
    });
}
```

### Memory Usage

Each pending load consumes:
- Channel: ~100 bytes
- Handle: ~200 bytes
- Task state: ~1-2 KB

**100 concurrent loads ≈ 200-300 KB overhead** (negligible)

### Optimization Tips

1. **Preload Common Assets**: Load frequently-used assets at startup
   ```rust
   async fn preload_common_assets(&mut self) -> Result<()> {
       let common = ["ui/button.obj", "effects/spark.obj", "player/hand.obj"];
       for path in common {
           let (_, receiver) = self.loader.load_async(path).await?;
           self.common_assets.insert(path.to_string(), receiver);
       }
       Ok(())
   }
   ```

2. **Priority Loading**: Load critical assets first
   ```rust
   // Load player assets immediately
   let player_loads = load_player_assets().await?;
   
   // Then start loading environment
   let env_loads = load_environment_assets().await?;
   
   // Wait for player first
   for (_, receiver) in player_loads {
       receiver.recv().unwrap()?;
   }
   ```

3. **Cached Results**: Don't reload the same asset
   ```rust
   if !asset_cache.contains(path) {
       let (_, receiver) = loader.load_async(path).await?;
       asset_cache.insert_pending(path, receiver);
   }
   ```

## Error Handling

### Handling Load Failures

```rust
let (handle, receiver) = loader.load_async("model.obj").await?;

match receiver.recv().unwrap() {
    Ok(mesh_data) => {
        // Success
        mesh_manager.add(mesh_data);
    }
    Err(e) => {
        eprintln!("Failed to load {}: {}", handle.path().display(), e);
        
        // Fallback strategies:
        
        // 1. Use placeholder asset
        mesh_manager.add_placeholder();
        
        // 2. Retry with timeout
        tokio::time::sleep(Duration::from_secs(1)).await;
        retry_load(handle.path()).await?;
        
        // 3. Skip and continue
        // (do nothing)
    }
}
```

### Batch Error Handling

```rust
let results = batch.wait_all();

let mut successes = Vec::new();
let mut failures = Vec::new();

for result in results {
    match result {
        Ok(mesh_data) => successes.push(mesh_data),
        Err(e) => failures.push(e),
    }
}

println!("Loaded: {}, Failed: {}", successes.len(), failures.len());

if !failures.is_empty() {
    // Handle failures
    for error in failures {
        eprintln!("Load error: {}", error);
    }
}
```

## Testing

### Test Non-Blocking Behavior

```rust
#[tokio::test]
async fn test_non_blocking_load() {
    let loader = AsyncMeshLoader::new();
    
    let start = Instant::now();
    let (handle, receiver) = loader.load_async("test.obj").await.unwrap();
    let elapsed = start.elapsed();
    
    // Should return immediately (< 50ms)
    assert!(elapsed.as_millis() < 50);
    
    // Actually loading happens in background
    let result = receiver.recv().unwrap();
    assert!(result.is_ok());
}
```

### Test Concurrent Loads

```rust
#[tokio::test]
async fn test_concurrent_loads() {
    let loader = AsyncMeshLoader::new();
    
    let mut handles = Vec::new();
    for i in 0..10 {
        let path = format!("test_{}.obj", i);
        handles.push(loader.load_async(path).await.unwrap());
    }
    
    // All should complete successfully
    for (_, receiver) in handles {
        assert!(receiver.recv().unwrap().is_ok());
    }
}
```

## Comparison with Sync Loading

| Aspect | Sync Loading | Async Loading |
|--------|-------------|---------------|
| **Blocking** | Blocks game loop | Non-blocking |
| **Performance** | Simple I/O | Concurrent I/O |
| **Complexity** | Simple | More complex |
| **Progress** | Not possible | Real-time progress |
| **Cancellation** | Not possible | Supported |
| **Memory** | Lower | Slightly higher |
| **Use Case** | Startup, small files | Runtime, large files |

**Recommendation**: Use async loading for runtime asset streaming and large files. Use sync loading for small startup assets where simplicity is preferred.

## Common Patterns

### Progressive Enhancement

Load low-quality first, then high-quality:

```rust
// Load low-res immediately (sync)
let low_res = loader_sync.load("model_lod2.obj")?;
mesh_manager.add("enemy", low_res);

// Load high-res in background (async)
let (_, receiver) = loader_async.load_async("model_lod0.obj").await?;

// Later, swap when ready
if let Ok(Ok(high_res)) = receiver.try_recv() {
    mesh_manager.replace("enemy", high_res);
}
```

### Timed Loading

Limit loading time per frame:

```rust
const MAX_LOAD_TIME_MS: u64 = 5;

fn process_asset_loads(&mut self) {
    let start = Instant::now();
    
    while start.elapsed().as_millis() < MAX_LOAD_TIME_MS {
        let completed = self.batch.try_receive_completed();
        
        if completed.is_empty() {
            break;
        }
        
        for result in completed {
            // Process loaded asset
            self.process_loaded_asset(result);
        }
    }
}
```

### Dependency Chain

Load assets with dependencies:

```rust
// Load model first
let (_, receiver) = mesh_loader.load_async("character.obj").await?;
let mesh = receiver.recv().unwrap()?;

// Then load textures (depends on knowing which textures the model needs)
let texture_paths = mesh.get_required_textures();
let texture_loads = texture_loader.load_many_async(texture_paths).await?;

// Wait for all textures
for (_, receiver) in texture_loads {
    let texture = receiver.recv().unwrap()?;
    // Upload texture
}
```

## See Also

- [Asset System](assets.md) - Synchronous asset loading
- [Mesh Loading](../reference/mesh-loading.md) - OBJ and GLTF formats
- [Texture Loading](../reference/texture-loading.md) - Image formats
- Example: `examples/async_asset_demo.rs`
