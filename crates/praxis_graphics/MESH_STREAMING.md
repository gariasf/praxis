# Mesh Streaming System

Asynchronous mesh loading with background thread processing, priority-based queuing, and frustum-based streaming.

## Overview

Large game worlds contain thousands of meshes. Loading all upfront causes long load times and high memory usage. The mesh streaming system provides:

- **Background Loading**: Dedicated thread without blocking render
- **Priority Queue**: High-priority meshes (close, visible) load first
- **Frustum Culling**: Only load meshes entering view frustum
- **Async GPU Upload**: Non-blocking GPU transfers
- **Bounding Spheres**: Fast visibility tests

## Quick Start

```rust
use praxis_graphics::MeshStreamingSystem;

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

// Per-frame update
streaming.update();
streaming.update_priorities(&frustum, camera_position);
streaming.load_visible_meshes(&|id| mesh_database.get(id).cloned());

// Render loaded meshes
if streaming.is_mesh_loaded("cube") {
    let mesh = streaming.get_mesh("cube").unwrap();
    // Render mesh...
}
```

## Architecture

### Components

**`MeshStreamingSystem`**
- Main coordinator
- Manages background thread
- Tracks mesh loading state
- Handles priority queue

**`StreamingGpuMesh`**
- Holds GPU mesh when loaded
- Tracks loading state
- Stores bounding sphere
- Maintains priority

**`MeshStreamingState`**
- `Unloaded`: Not loaded yet
- `Queued`: Waiting in priority queue
- `Loading`: Currently loading
- `Loaded`: Ready to render
- `Failed`: Load error occurred

**Background Thread**
- Processes priority queue
- Calls `GpuMesh::new_async()`
- Sends results to main thread

### Data Flow

```
Main Thread                    Background Thread
    |                                |
    |-- register_mesh() ------------>|
    |                                |
    |-- load_visible_meshes() ------>|
    |                                |-- GpuMesh::new_async()
    |                                |
    |<-- send result ---------------|
    |                                |
    |-- update() (receive)           |
    |                                |
    |-- render loaded meshes         |
```

## Priority System

Meshes are prioritized based on:

1. **Visibility**: In frustum vs out of frustum
2. **Distance**: Closer meshes get higher priority
3. **Zone Priority**: Near (100), medium (50), far (10)

```rust
// Priority calculation (automatic)
let distance = (world_pos - camera_pos).length();
let visibility_priority = if distance < radius * 2.0 {
    100.0  // Very close
} else if distance < radius * 10.0 {
    50.0   // Medium distance
} else {
    10.0   // Far
};
let distance_priority = 1000.0 / (distance + 1.0);
let priority = visibility_priority + distance_priority;
```

## Usage Patterns

### Basic Setup

```rust
// Create streaming system
let mut streaming = MeshStreamingSystem::new(
    allocator.clone(),
    cmd_allocator.clone(),
    queue.clone(),
);

// Register meshes
streaming.register_mesh("cube", cube_mesh_data)?;
streaming.register_mesh("sphere", sphere_mesh_data)?;
```

### Frame Update Loop

```rust
// 1. Update streaming (process completed loads)
streaming.update();

// 2. Update priorities based on camera
let frustum = Frustum::from_view_projection(view_proj);
streaming.update_priorities(&frustum, camera_position);

// 3. Trigger loading for visible meshes
streaming.load_visible_meshes(&|id| {
    mesh_database.get(id).cloned()
});

// 4. Render only loaded meshes
for entity in entities {
    if streaming.is_mesh_loaded(&entity.mesh_id) {
        let mesh = streaming.get_mesh(&entity.mesh_id).unwrap();
        // Render mesh...
    }
}
```

### Pre-loading Important Meshes

```rust
// Pre-load critical meshes during initialization
let (gpu_mesh, future) = mesh_data.upload_async(
    allocator.clone(),
    cmd_allocator.clone(),
    queue.clone(),
)?;

// Wait for upload
future.wait()?;

// Register as already loaded
streaming.register_loaded_mesh(
    "player",
    gpu_mesh,
    Vec3::ZERO,  // bounding center
    1.0,         // bounding radius
);
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

**Bounding sphere calculation:**
```rust
fn calculate_bounding_sphere(vertices: &[Vertex]) -> (Vec3, f32) {
    // Find center
    let center = vertices.iter()
        .map(|v| v.position)
        .sum::<Vec3>() / vertices.len() as f32;
    
    // Find max radius
    let radius = vertices.iter()
        .map(|v| (v.position - center).length())
        .max_by(|a, b| a.partial_cmp(b).unwrap())
        .unwrap_or(0.0);
    
    (center, radius)
}
```

## Performance

### Background Thread

- One loader thread shared by all meshes
- Processes highest priority first
- Sleeps 1ms when idle (low CPU usage)
- Graceful shutdown on `Drop`

### Memory Management

- Streaming meshes hold bounding data only (~32 bytes)
- GPU meshes created only when loaded
- Failed loads don't retry automatically
- Clear system to free memory: `streaming.clear()`

### GPU Upload

Uses `GpuMesh::new_async()` for:
- Non-blocking transfer to GPU
- Staging buffer optimization
- Fence synchronization
- Immediate return with future

### Frustum Culling

Fast sphere-frustum tests:
- 6 plane checks per mesh
- Early-out on first failure
- Simple dot product math
- Runs on main thread (very cheap)

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
streaming.load_visible_meshes(&mesh_provider);
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

### 4. Provide Efficient Data Provider

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
println!("Loaded: {}/{} meshes ({:.1}%)", 
         loaded, total, 
         100.0 * loaded as f32 / total as f32);
```

## Integration Examples

### With Asset Manager

```rust
// Load mesh files, create MeshData
let mesh_data = asset_manager.load_mesh("models/house.obj")?;

// Register with streaming
streaming.register_mesh("house", mesh_data)?;
```

### With Scene Graph

```rust
// Entities reference mesh IDs
struct Entity {
    mesh_id: String,
    transform: Transform,
}

// Render only if mesh is loaded
for entity in scene.entities() {
    if streaming.is_mesh_loaded(&entity.mesh_id) {
        let mesh = streaming.get_mesh(&entity.mesh_id)?;
        render_entity(entity, mesh);
    }
}
```

### With Level Streaming

```rust
// When loading a new level area
for mesh in level.meshes {
    streaming.register_mesh(mesh.id, mesh.data)?;
}

// When unloading an area
for mesh_id in old_level.mesh_ids {
    streaming.unregister_mesh(&mesh_id);
}
```

## Error Handling

Failed loads are tracked:

```rust
// Check if load failed
let meshes = streaming.meshes.read();
if let Some(mesh) = meshes.get("failed_mesh") {
    if mesh.state == MeshStreamingState::Failed {
        eprintln!("Failed to load mesh: {}", "failed_mesh");
        // Handle error (show placeholder, retry, etc.)
    }
}
```

No automatic retry - application must handle failures explicitly.

## Limitations

1. **Single Background Thread**: One thread for all loading (could use thread pool)
2. **No LOD Support**: Loads full-quality mesh (could integrate with LOD system)
3. **No Unloading**: Meshes stay loaded once loaded (could add LRU eviction)
4. **No Progress Tracking**: Binary loaded/not loaded (could add progress percentage)
5. **Fixed World Position**: Uses `Vec3::ZERO` for visibility (needs entity position tracking)

## Thread Safety

- `MeshStreamingSystem`: Not `Send` (contains GPU resources)
- Background thread: Communicates via channels
- Mesh state: Protected by `RwLock`
- GPU resources: Created on main thread after transfer

## Future Enhancements

- **LOD Integration**: Load lower LODs first, upgrade to higher LODs
- **Unloading**: Remove distant meshes to free memory (LRU cache)
- **Thread Pool**: Multiple loader threads for faster loading
- **Progress Tracking**: Report loading progress per mesh
- **Prefetching**: Predict camera movement, pre-load ahead
- **Compression**: Decompress mesh data on background thread
- **Persistent Cache**: Save GPU meshes to disk for faster startup

## See Also

- `mesh.rs` - Core mesh types
- `GpuMesh::new_async()` - Async GPU upload
- `praxis_spatial::Frustum` - Frustum culling
- Example: `examples/mesh_streaming_demo.rs`
