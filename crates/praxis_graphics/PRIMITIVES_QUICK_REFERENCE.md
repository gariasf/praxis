# Core Rendering Primitives - Quick Reference

Quick reference for using core rendering primitives in Praxis.

## Vertex Creation

```rust
use praxis_graphics::Vertex3D;

// Basic vertex
let v = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

// With UV coordinates
let v = Vertex3D::with_uv([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5]);

// With all attributes
let v = Vertex3D::with_all([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5]);

// Zero-copy to bytes
let bytes = bytemuck::bytes_of(&v);
```

## Mesh Creation

```rust
use praxis_graphics::{MeshData, GpuMesh};

// Create mesh data
let mesh_data = MeshData::with_colors(positions, colors, indices);

// Convert to vertices
let vertices = mesh_data.to_vertices();

// Upload to GPU (synchronous)
let gpu_mesh = GpuMesh::new(
    allocator,
    cmd_allocator,
    queue,
    vertices,
    indices,
)?;

// Upload to GPU (asynchronous)
let (gpu_mesh, future) = GpuMesh::new_async(
    allocator,
    cmd_allocator,
    queue,
    vertices,
    indices,
)?;
// ... do other work ...
future.wait(None)?;
```

## Generic Buffers

```rust
use praxis_graphics::buffer::{GpuBuffer, StagingBuffer};
use vulkano::buffer::BufferUsage;

// Create GPU buffer with automatic staging
let buffer = GpuBuffer::from_data(
    allocator,
    cmd_allocator,
    queue,
    BufferUsage::VERTEX_BUFFER,
    &data,
)?;

// Manual staging buffer workflow
let staging = StagingBuffer::new(allocator, &data)?;
let gpu_buffer = GpuBuffer::new(allocator, BufferUsage::UNIFORM_BUFFER, data.len() as u64)?;
gpu_buffer.copy_from_staging(cmd_allocator, queue, &staging)?;
```

## Buffer Manager

```rust
use praxis_graphics::buffer::BufferManager;

// Create manager
let mut manager = BufferManager::new(allocator);

// Create buffers through manager
let buffer = manager.create_buffer_from_data(
    cmd_allocator,
    queue,
    BufferUsage::VERTEX_BUFFER,
    &data,
)?;

// Advance frame
manager.next_frame();
```

## Texture Loading

```rust
use praxis_graphics::{Texture, TextureManager};

// Create texture manager
let mut texture_manager = TextureManager::new(allocator, cmd_allocator, queue);

// Load texture from file
texture_manager.load_texture("brick", "assets/textures/brick.png")?;

// Get cached texture
let texture = texture_manager.get_texture("brick")?;

// Create from RGBA8 data
let texture = Texture::from_rgba8(allocator, cmd_allocator, queue, width, height, rgba_data)?;
```

## Descriptor Set Caching

```rust
use praxis_graphics::descriptor_manager::{DescriptorSetCache, DescriptorSetKey};

// Create cache
let mut cache = DescriptorSetCache::new(allocator, layout);

// Get or create descriptor set
let key = DescriptorSetKey::from_hashable(&config);
let descriptor_set = cache.get_or_create(key, || {
    // Create descriptor set if not cached
    create_descriptor_set()
})?;

// Advance frame (runs LRU eviction)
cache.next_frame();
```

## Resource Lifetime Tracking

```rust
use praxis_graphics::descriptor_manager::ResourceLifetimeTracker;

// Create tracker (3 frames in flight)
let mut tracker = ResourceLifetimeTracker::new(3);

// Mark resource as used
tracker.mark_used(resource_id);

// Check if can be freed
if tracker.can_free(resource_id) {
    // Safe to free resource
}

// Advance frame
tracker.next_frame();
```

## Memory Types

| Type | Memory | Use Case |
|------|--------|----------|
| `StagingBuffer<T>` | Host-visible (PREFER_HOST) | Temporary upload buffers |
| `GpuBuffer<T>` | Device-local (PREFER_DEVICE) | Permanent rendering buffers |
| `Vertex3D` | `bytemuck::Pod` | Zero-copy GPU upload |

## Buffer Usage Flags

```rust
use vulkano::buffer::BufferUsage;

BufferUsage::VERTEX_BUFFER        // Vertex data
BufferUsage::INDEX_BUFFER         // Index data
BufferUsage::UNIFORM_BUFFER       // Uniform data (small, frequent updates)
BufferUsage::STORAGE_BUFFER       // Storage buffer (large, arbitrary access)
BufferUsage::TRANSFER_SRC         // Source for transfers (staging)
BufferUsage::TRANSFER_DST         // Destination for transfers (device)
```

## Common Patterns

### Pattern 1: Simple Mesh Upload

```rust
// Create vertices
let vertices = vec![
    Vertex3D::new([0.0, 0.5, 0.0], [1.0, 0.0, 0.0]),
    Vertex3D::new([-0.5, -0.5, 0.0], [0.0, 1.0, 0.0]),
    Vertex3D::new([0.5, -0.5, 0.0], [0.0, 0.0, 1.0]),
];
let indices = vec![0, 1, 2];

// Upload to GPU
let gpu_mesh = GpuMesh::new(allocator, cmd_allocator, queue, vertices, indices)?;
```

### Pattern 2: Descriptor Set Caching

```rust
// Hash material configuration
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

let mut hasher = DefaultHasher::new();
texture_name.hash(&mut hasher);
material_props.hash(&mut hasher);
let key = DescriptorSetKey::from_hash(hasher.finish());

// Get or create descriptor set
let descriptor_set = cache.get_or_create(key, || {
    // Only called if not cached
    DescriptorSet::new(allocator, layout, writes, [])
})?;
```

### Pattern 3: Frame-Based Resource Management

```rust
struct RenderContext {
    frame_resources: Vec<Arc<dyn Any>>,
    // ... other fields ...
}

impl RenderContext {
    fn render(&mut self) {
        // Clear previous frame's resources (after GPU finishes)
        self.frame_resources.clear();
        
        // Create and track resources for this frame
        let descriptor_set = create_descriptor_set();
        self.frame_resources.push(descriptor_set.clone());
        
        // Use resources...
    }
}
```

## Performance Tips

1. **Use device-local buffers for rendering**: 10-100x faster GPU access
2. **Pool descriptor sets**: 100x reduction in allocations
3. **Batch buffer uploads**: Create all meshes, then upload together
4. **Use async uploads for streaming**: Non-blocking, parallel with other work
5. **Track resource lifetimes**: Prevent premature cleanup

## Size Reference

```text
Vertex3D: 92 bytes
  - position:      12 bytes (3 × f32)
  - normal:        12 bytes (3 × f32)
  - color:         12 bytes (3 × f32)
  - uv:            8 bytes (2 × f32)
  - tangent:       16 bytes (4 × f32)
  - bone_indices:  16 bytes (4 × i32)
  - bone_weights:  16 bytes (4 × f32)
```

## Common Errors

### Error: "Buffer creation failed"
- **Cause**: Not enough VRAM
- **Fix**: Reduce buffer sizes or use compression

### Error: "Transfer timeout"
- **Cause**: GPU queue stalled
- **Fix**: Check for GPU errors, reduce batch size

### Error: "Descriptor set allocation failed"
- **Cause**: Descriptor pool exhausted
- **Fix**: Use `DescriptorSetCache` for pooling

### Error: "Access violation in GPU"
- **Cause**: Resource freed while GPU using it
- **Fix**: Use `ResourceLifetimeTracker` or Arc

## See Also

- **RENDERING_PRIMITIVES.md** - Comprehensive guide
- **CORE_PRIMITIVES_SUMMARY.md** - Implementation summary
- **examples/rendering_primitives_demo.rs** - Working example
- **Module documentation** - Rust docs for each module
