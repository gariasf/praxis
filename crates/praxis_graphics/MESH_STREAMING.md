# Mesh Streaming System

The mesh streaming system provides async mesh loading with background thread processing, priority-based queuing, and frustum-based on-demand loading.

## Overview

Large game worlds often contain thousands of meshes. Loading all meshes upfront causes long load times and high memory usage. The mesh streaming system solves this by:

- **Background Loading**: Meshes load on a dedicated thread without blocking the render thread
- **Priority Queue**: High-priority meshes (close, visible) load first
- **Frustum Culling**: Only meshes entering the view frustum are loaded
- **Async GPU Upload**: Uses `GpuMesh::new_async()` for non-blocking GPU transfers
- **Bounding Spheres**: Fast visibility tests using per-mesh bounding spheres

## Architecture

### Components

1. **`MeshStreamingSystem`**: Main coordinator
   - Manages background thread
   - Tracks mesh loading state
   - Handles priority queue
   - Processes completed uploads

2. **`StreamingGpuMesh`**: Mesh container
   - Holds GPU mesh when loaded
   - Tracks loading state
   - Stores bounding sphere
   - Maintains priority

3. **`MeshStreamingState`**: Loading states
   - `Unloaded`: Not loaded yet
   - `Queued`: Waiting in priority queue
   - `Loading`: Currently loading
   - `Loaded`: Ready to render
   - `Failed`: Load error occurred

4. **Background Thread**: Loader worker
   - Processes priority queue
   - Calls `GpuMesh::new_async()`
   - Sends results back to main thread

## Usage

### Basic Setup

```rust
use praxis_graphics::{MeshStreamingSystem, MeshData};
use praxis_math::Vec3;
use praxis_spatial::Frustum;

// Create streaming system
let mut streaming = MeshStreamingSystem::new(
    allocator.clone(),
    command_buffer_allocator.clone(),
    transfer_queue.clone(),
);

// Register meshes for streaming
for (id, mesh_data) in meshes {
    streaming.register_mesh(id, mesh_data)?;
}
```

### Frame Update Loop

```rust
// 1. Update streaming system (process completed loads)
streaming.update();

// 2. Update priorities based on camera
let frustum = Frustum::from_view_projection(view_proj);
streaming.update_priorities(&frustum, camera_position);

// 3. Trigger loading for visible meshes
streaming.load_visible_meshes(&|id| mesh_database.get(id).cloned());

// 4. Render only loaded meshes
if streaming.is_mesh_loaded("cube") {
    let mesh = streaming.get_mesh("cube").unwrap();
    // Render mesh...
}
```

### Pre-loading Important Meshes

```rust
// Pre-load meshes that should always be ready
let (gpu_mesh, future) = mesh_data.upload_async(
    allocator.clone(),
    cmd_allocator.clone(),
    queue.clone(),
)?;

// Register as already loaded
streaming.register_loaded_mesh(
    "important_mesh",
    gpu_mesh,
    bounding_center,
    bounding_radius,
);
```

## Priority System

Meshes are assigned priority based on:

1. **Visibility**: In frustum vs out of frustum
2. **Distance**: Closer meshes get higher priority
3. **Zone Priority**: Near (100), medium (50), far (10)

```rust
// Priority calculation (automatic in update_priorities)
let distance = (world_pos - camera_pos).length();
let visibility_priority = if distance < radius * 2.0 {
    100.0  // Very close
} else if distance < radius * 10.0 {
    50.0   // Medium distance
} else {
    10.0   // Far
};
let distance_priority = 1000.0 / (distance + 1.0);
priority = visibility_priority + distance_priority;
```

## Bounding Spheres

Each mesh has a bounding sphere for fast frustum culling:

```rust
// Automatic calculation from mesh data
let (center, radius) = mesh_data.calculate_bounding_sphere();

// Manual specification
streaming.register_loaded_mesh(
    "sphere",
    gpu_mesh,
    Vec3::new(0.0, 1.0, 0.0),  // center
    5.0,                        // radius
);
```

## Performance Considerations

### Background Thread

- One loader thread shared by all meshes
- Processes highest priority first
- Sleeps 1ms when idle (low CPU usage)
- Graceful shutdown on `Drop`

### Memory Management

- Streaming meshes hold bounding data only (small)
- GPU meshes created only when loaded
- Failed loads don't retry automatically
- Clear system to free memory: `streaming.clear()`

### GPU Upload

Uses `GpuMesh::new_async()` for:
- Non-blocking transfer to GPU
- Staging buffer optimization
- Fence synchronization
- Returns immediately with future

### Frustum Culling

Fast sphere-frustum tests:
- 6 plane checks per mesh
- Early-out on first failure
- Simple dot product math
- Runs on main thread (cheap)

## Best Practices

### 1. Register All Meshes Early

```rust
// At startup, register all meshes with metadata
for (id, mesh_meta) in mesh_database {
    streaming.register_mesh(id, mesh_meta.data)?;
}
```

### 2. Update Every Frame

```rust
// In game loop
streaming.update();  // Process completed loads
streaming.update_priorities(&frustum, camera_pos);
streaming.load_visible_meshes(&provider);
```

### 3. Handle Loading State

```rust
match streaming.get_mesh("cube") {
    Some(mesh) => {
        // Render mesh
    }
    None => {
        // Show placeholder or skip
    }
}
```

### 4. Provide Mesh Data Provider

```rust
// Efficient closure that retrieves mesh data on-demand
let mesh_db = Arc::new(mesh_database);
streaming.load_visible_meshes(&|id| {
    mesh_db.get(id).cloned()
});
```

### 5. Monitor Statistics

```rust
let loaded = streaming.loaded_count();
let total = streaming.total_count();
println!("Loaded: {}/{} meshes", loaded, total);
```

## Example: Open World Game

```rust
struct World {
    streaming: MeshStreamingSystem,
    mesh_database: HashMap<String, MeshData>,
    camera: Camera,
}

impl World {
    fn update(&mut self) {
        // Update streaming
        self.streaming.update();
        
        // Update priorities based on camera
        let frustum = self.camera.frustum();
        let pos = self.camera.position();
        self.streaming.update_priorities(&frustum, pos);
        
        // Load visible meshes
        let db = &self.mesh_database;
        self.streaming.load_visible_meshes(&|id| db.get(id).cloned());
    }
    
    fn render(&self) {
        for entity in &self.entities {
            if self.streaming.is_mesh_loaded(&entity.mesh_id) {
                let mesh = self.streaming.get_mesh(&entity.mesh_id).unwrap();
                // Render entity with mesh...
            }
        }
    }
}
```

## Limitations

1. **Single Background Thread**: One thread for all loading
   - Future: Could add thread pool
   
2. **No LOD Support**: Loads full-quality mesh
   - Future: Could integrate with LOD system
   
3. **No Unloading**: Meshes stay loaded once loaded
   - Future: Could add LRU eviction
   
4. **No Progress Tracking**: Binary loaded/not loaded
   - Future: Could add progress percentage
   
5. **Fixed World Position**: Uses `Vec3::ZERO` for visibility
   - Current: Need to track entity positions
   - Future: Pass world positions to `update_priorities`

## Thread Safety

- `MeshStreamingSystem`: Not `Send` (contains GPU resources)
- Background thread: Communicates via channels
- Mesh state: Protected by `RwLock`
- GPU resources: Created on main thread after transfer

## Error Handling

Failed loads are tracked:
```rust
// Check if load failed
let meshes = streaming.meshes.read();
if let Some(mesh) = meshes.get("failed_mesh") {
    if mesh.state == MeshStreamingState::Failed {
        // Handle error
    }
}
```

No automatic retry - application must handle failures.

## Integration with Existing Systems

### Asset Manager

```rust
// Load mesh files, create MeshData
let mesh_data = asset_manager.load_mesh("models/house.obj")?;

// Register with streaming
streaming.register_mesh("house", mesh_data)?;
```

### Scene Graph

```rust
// Entities reference mesh IDs
struct Entity {
    mesh_id: String,
    transform: Transform,
}

// Render only if mesh is loaded
for entity in scene.entities() {
    if streaming.is_mesh_loaded(&entity.mesh_id) {
        // Render entity
    }
}
```

### Level Streaming

```rust
// When loading a new level area
for mesh in level.meshes {
    streaming.register_mesh(mesh.id, mesh.data)?;
}

// When unloading an area
streaming.clear();  // Or selectively remove meshes
```

## Future Enhancements

1. **LOD Integration**: Load lower LODs first, upgrade to higher LODs
2. **Unloading**: Remove distant meshes to free memory
3. **Thread Pool**: Multiple loader threads for faster loading
4. **Progress Tracking**: Report loading progress per mesh
5. **Prefetching**: Predict camera movement, pre-load ahead
6. **Compression**: Decompress mesh data on background thread
7. **Caching**: Save GPU meshes to disk for faster startup

## See Also

- `mesh.rs` - Core mesh types
- `GpuMesh::new_async()` - Async GPU upload
- `praxis_spatial::Frustum` - Frustum culling
- `examples/mesh_streaming_demo.rs` - Complete example
