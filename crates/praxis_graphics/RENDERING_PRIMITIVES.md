# Rendering Primitives

This document describes the core rendering primitives in the Praxis graphics system, including vertex structures, mesh management, buffer abstractions, texture handling, descriptor set management, and GPU resource lifetime tracking.

## Overview

The rendering primitives provide the foundation for all GPU rendering operations:

- **Vertex Structure**: `Vertex3D` - Complete vertex format with position, normal, UV, tangent, and skinning data
- **Mesh System**: `MeshData`, `GpuMesh` - CPU-side mesh definition and GPU buffer management with staging
- **Buffer Abstractions**: `GpuBuffer<T>`, `StagingBuffer<T>`, `BufferManager` - Generic buffer management with lifetime tracking
- **Texture System**: `Texture`, `TextureManager` - Image loading and GPU texture management
- **Descriptor Management**: `DescriptorSetCache`, `ResourceLifetimeTracker` - Descriptor set pooling and resource lifetime tracking

## Vertex Structure

### Vertex3D

The `Vertex3D` structure is the primary vertex format for 3D rendering. It uses `bytemuck::Pod` for safe zero-copy conversion between Rust types and GPU memory.

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, Vertex)]
pub struct Vertex3D {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],      // 12 bytes - 3D position

    #[format(R32G32B32_SFLOAT)]
    pub normal: [f32; 3],        // 12 bytes - Normal vector

    #[format(R32G32B32_SFLOAT)]
    pub color: [f32; 3],         // 12 bytes - RGB color

    #[format(R32G32_SFLOAT)]
    pub uv: [f32; 2],            // 8 bytes - Texture coordinates

    #[format(R32G32B32A32_SFLOAT)]
    pub tangent: [f32; 4],       // 16 bytes - Tangent (xyz) + handedness (w)

    #[format(R32G32B32A32_SINT)]
    pub bone_indices: [i32; 4],  // 16 bytes - Bone indices for skinning

    #[format(R32G32B32A32_SFLOAT)]
    pub bone_weights: [f32; 4],  // 16 bytes - Bone weights for skinning
}
// Total: 92 bytes per vertex
```

**Key Features:**

- **Memory Layout**: `#[repr(C)]` ensures predictable layout for GPU uploads
- **Zero-Copy Conversion**: `bytemuck::Pod` allows safe reinterpretation as byte slices
- **Complete Attributes**: Supports position, normal, color, UV, tangent, and skeletal animation
- **Shader Binding**: Maps directly to vertex shader input locations 0-6

**Constructor Methods:**

```rust
// Basic vertex with position and color
Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0])

// With texture coordinates
Vertex3D::with_uv([0.0, 0.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5])

// With all attributes
Vertex3D::with_all([0.0, 0.0, 0.0], [0.0, 1.0, 0.0], [1.0, 1.0, 1.0], [0.5, 0.5])

// With tangent for normal mapping
Vertex3D::with_tangent(position, normal, color, uv, [1.0, 0.0, 0.0, 1.0])

// With skeletal animation data
Vertex3D::with_skinning(position, normal, color, uv, tangent, bone_indices, bone_weights)
```

## Mesh System

### MeshData (CPU-side)

`MeshData` is the CPU-side mesh representation used before GPU upload:

```rust
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub colors: Option<Vec<[f32; 3]>>,
    pub normals: Option<Vec<[f32; 3]>>,
    pub uvs: Option<Vec<[f32; 2]>>,
    pub tangents: Option<Vec<[f32; 4]>>,
    pub indices: Vec<u16>,
}
```

**Usage:**

```rust
// Create mesh data
let mesh_data = MeshData::with_uvs(
    vec![
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
    ],
    vec![
        [0.0, 0.0],
        [1.0, 0.0],
        [0.5, 1.0],
    ],
    vec![0, 1, 2],
);

// Convert to vertices for GPU upload
let vertices = mesh_data.to_vertices();
```

**Methods:**

- `new()` - Basic mesh with positions and indices
- `with_colors()` - Mesh with vertex colors
- `with_uvs()` - Mesh with texture coordinates
- `with_colors_and_uvs()` - Mesh with both colors and UVs
- `to_vertices()` - Convert to `Vec<Vertex3D>` for GPU upload
- `calculate_normals()` - Auto-generate normals from geometry
- `calculate_tangents()` - Auto-generate tangents for normal mapping

### GpuMesh (GPU-side)

`GpuMesh` is the GPU-side representation with device-local vertex and index buffers:

```rust
pub struct GpuMesh {
    pub vertex_buffer: Subbuffer<[Vertex3D]>,
    pub index_buffer: Subbuffer<[u16]>,
    pub index_count: u32,
    pub vertex_count: u32,
}
```

**Staging Buffer Upload Pattern:**

The mesh system uses a two-stage upload process for optimal performance:

1. **Staging Buffers** (CPU-visible, PREFER_HOST memory):
   - Fast CPU writes
   - Used as temporary upload buffers
   - Automatically freed after transfer

2. **Device Buffers** (GPU-only, PREFER_DEVICE memory):
   - Optimal GPU access performance
   - Cannot be directly written from CPU
   - Permanent storage for rendering

**Synchronous Upload:**

```rust
let gpu_mesh = GpuMesh::new(
    memory_allocator,
    command_buffer_allocator,
    graphics_queue,
    vertices,
    indices,
)?;
// Blocks until transfer completes
```

**Asynchronous Upload:**

```rust
let (gpu_mesh, future) = GpuMesh::new_async(
    memory_allocator,
    command_buffer_allocator,
    graphics_queue,
    vertices,
    indices,
)?;

// Do other work while GPU transfer happens
do_other_initialization();

// Wait when mesh is needed
future.wait(None)?;
```

## Buffer Abstractions

### GpuBuffer<T>

Generic device-local buffer for any `bytemuck::Pod` type:

```rust
// Create vertex buffer
let vertices = vec![[0.0f32, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
let buffer = GpuBuffer::from_data(
    allocator,
    cmd_allocator,
    queue,
    BufferUsage::VERTEX_BUFFER,
    &vertices,
)?;

// Access underlying Vulkan buffer
let subbuffer = buffer.buffer();
```

**Features:**

- Generic over any `bytemuck::Pod` type
- Automatic staging buffer creation and transfer
- Device-local memory for optimal GPU performance
- Element count and size tracking

### StagingBuffer<T>

Host-visible buffer for CPU-to-GPU data transfer:

```rust
let data = vec![1.0f32, 2.0, 3.0, 4.0];
let staging = StagingBuffer::new(allocator, &data)?;

// Use for manual transfer to device buffer
let gpu_buffer = GpuBuffer::new(allocator, BufferUsage::UNIFORM_BUFFER, data.len() as u64)?;
gpu_buffer.copy_from_staging(cmd_allocator, queue, &staging)?;
```

**Use Cases:**

- Manual buffer uploads
- Reusable staging buffers for multiple transfers
- Fine-grained control over transfer timing

### BufferManager

Centralized buffer management with lifetime tracking:

```rust
let mut manager = BufferManager::new(allocator);

// Create buffers through manager
let buffer = manager.create_buffer_from_data(
    cmd_allocator,
    queue,
    BufferUsage::VERTEX_BUFFER,
    &vertices,
)?;

// Advance frame for lifetime tracking
manager.next_frame();
```

**Features:**

- Frame-based lifetime tracking
- Centralized allocation point
- Potential for future buffer pooling/recycling

## Texture System

### Texture

GPU-side texture with image, view, and sampler:

```rust
pub struct Texture {
    pub image: Arc<Image>,
    pub view: Arc<ImageView>,
    pub sampler: Arc<Sampler>,
    pub width: u32,
    pub height: u32,
}
```

**Creation from RGBA8 data:**

```rust
let texture = Texture::from_rgba8(
    allocator,
    cmd_allocator,
    queue,
    width,
    height,
    rgba_data,
)?;
```

**Features:**

- Automatic staging buffer upload
- Layout transitions handled automatically
- Configurable sampling (linear, nearest, wrap modes)
- Support for mipmap generation

### TextureManager

Centralized texture management with caching:

```rust
let mut texture_manager = TextureManager::new(
    allocator,
    cmd_allocator,
    queue,
);

// Load texture from file
texture_manager.load_texture("brick", "assets/textures/brick.png")?;

// Get cached texture
let texture = texture_manager.get_texture("brick")?;

// Create default textures
texture_manager.create_default_white_texture()?;
texture_manager.create_default_flat_normal()?;
```

## Descriptor Set Management

### DescriptorSetCache

Efficient descriptor set pooling with LRU eviction:

```rust
let mut cache = DescriptorSetCache::new(allocator, layout);

// Get or create descriptor set
let key = DescriptorSetKey::from_hashable(&config);
let descriptor_set = cache.get_or_create(key, || {
    // Create descriptor set if not cached
    create_descriptor_set()
})?;

// Advance frame and evict unused sets
cache.next_frame();
```

**Features:**

- Automatic caching and reuse
- LRU eviction after 60 unused frames
- Frame-based lifetime tracking
- Reduces descriptor set allocation overhead by 100x+

### ResourceLifetimeTracker

Track GPU resource lifetimes across frames:

```rust
let mut tracker = ResourceLifetimeTracker::new(3); // 3 frames in flight

// Mark resource as used
tracker.mark_used(resource_id);

// Check if can be freed (after grace period)
if tracker.can_free(resource_id) {
    // Safe to free resource
}

// Advance frame
tracker.next_frame();
```

**Use Cases:**

- Ensuring resources remain alive while GPU uses them
- Delaying resource cleanup until safe
- Managing in-flight frame synchronization

## GPU Resource Lifetime Tracking

### The Frame-in-Flight Problem

GPUs process commands asynchronously. When you submit rendering commands, the GPU may not execute them immediately. This creates a lifetime problem:

```rust
// Frame 1: Submit draw command that references buffer
render_commands.push(DrawCommand { mesh, texture, ... });
submit_to_gpu(render_commands);

// Frame 2: GPU might still be processing Frame 1
// ❌ WRONG: Dropping buffer now causes crash
drop(buffer); 

// ✓ CORRECT: Keep buffer alive until GPU finishes
tracker.mark_used(buffer_id);
// ... wait N frames based on max frames in flight ...
if tracker.can_free(buffer_id) {
    drop(buffer);
}
```

### Lifetime Tracking Patterns

**Pattern 1: Frame-based tracking**

```rust
struct RenderContext {
    current_frame: u64,
    frame_resources: HashMap<u64, Vec<Arc<dyn Any>>>,
    frames_in_flight: usize,
}

impl RenderContext {
    fn track_resource(&mut self, resource: Arc<dyn Any>) {
        self.frame_resources
            .entry(self.current_frame)
            .or_default()
            .push(resource);
    }

    fn cleanup_old_frames(&mut self) {
        let cutoff = self.current_frame.saturating_sub(self.frames_in_flight as u64);
        self.frame_resources.retain(|&frame, _| frame > cutoff);
    }
}
```

**Pattern 2: Descriptor set lifetime tracking (used in RenderContext)**

```rust
// Descriptor sets tracked per frame to ensure they remain alive
frame_descriptor_sets: Vec<Arc<DescriptorSet>>,

// In render():
self.frame_descriptor_sets.clear(); // Safe after cleanup_finished()

// Track descriptor sets used this frame
self.frame_descriptor_sets.push(descriptor_set.clone());
```

**Pattern 3: Automatic cleanup with Arc**

Most resources in Praxis use `Arc<T>` (reference counting):

- Resources are cloned when used
- Automatically freed when last reference drops
- GPU synchronization ensures no in-flight references

## Data Conversion with bytemuck

### Why bytemuck?

`bytemuck` provides safe zero-copy conversion between Rust types and byte slices:

```rust
// Vertex3D implements bytemuck::Pod
let vertex = Vertex3D::new([0.0, 0.0, 0.0], [1.0, 1.0, 1.0]);

// Zero-copy conversion to bytes
let bytes: &[u8] = bytemuck::bytes_of(&vertex);

// Upload directly to GPU buffer
buffer.write(bytes)?;
```

### Requirements for bytemuck::Pod

Types must be "Plain Old Data" to implement `Pod`:

```rust
#[repr(C)]  // Stable memory layout
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct MyVertex {
    position: [f32; 3],  // ✓ Pod type
    color: [f32; 4],     // ✓ Pod type
    // String, Vec, etc. are NOT Pod
}
```

### Alignment Considerations

GPU buffers have alignment requirements:

- **Uniform buffers**: 256-byte alignment for dynamic offsets
- **Vertex buffers**: Aligned to vertex stride
- **Index buffers**: Aligned to index type (2 or 4 bytes)

Vulkano handles alignment automatically for buffer creation.

## Best Practices

### Memory Management

1. **Use device-local buffers for rendering**:
   - Vertex buffers, index buffers → PREFER_DEVICE
   - Uniform buffers → PREFER_DEVICE for static data

2. **Use host-visible buffers for uploads**:
   - Staging buffers → PREFER_HOST + HOST_SEQUENTIAL_WRITE
   - Dynamic uniform buffers → PREFER_HOST (if frequently updated)

3. **Minimize transfers**:
   - Upload meshes once at load time
   - Use dynamic uniform buffers for per-frame data
   - Batch updates when possible

### Descriptor Set Management

1. **Pool descriptor sets**:
   - Use `DescriptorSetCache` for automatic pooling
   - Share descriptor sets between similar objects
   - Evict unused sets to bound memory

2. **Track lifetimes**:
   - Keep descriptor sets alive for frames in flight
   - Use `ResourceLifetimeTracker` for explicit tracking
   - Clear per-frame resources after GPU synchronization

### Staging Buffers

1. **Synchronous uploads for initialization**:
   ```rust
   let mesh = GpuMesh::new(...)?; // Blocks until ready
   ```

2. **Asynchronous uploads for streaming**:
   ```rust
   let (mesh, future) = GpuMesh::new_async(...)?;
   // Do other work...
   future.wait(None)?;
   ```

3. **Reuse staging buffers**:
   ```rust
   let staging = StagingBuffer::new(allocator, &data)?;
   gpu_buffer1.copy_from_staging(cmd_alloc, queue, &staging)?;
   gpu_buffer2.copy_from_staging(cmd_alloc, queue, &staging)?;
   ```

## Performance Considerations

### Buffer Creation Overhead

- Creating buffers is expensive (Vulkan allocation + memory mapping)
- Pool and reuse buffers when possible
- Use large buffers with offsets instead of many small buffers

### Transfer Performance

- Staging buffers reduce transfer overhead
- Device-local buffers provide 10-100x faster GPU access
- Transfer queue allows parallel uploads with rendering

### Descriptor Set Overhead

Without pooling:
- 100 objects × 10 materials = 1000 descriptor set allocations per frame
- Significant CPU overhead from allocation and GPU binding

With pooling:
- 10 unique materials = 10 descriptor set allocations total
- 100x+ reduction in allocations
- Cache hit rate typically >95% after first frame

## Example: Complete Mesh Rendering Pipeline

```rust
// 1. Create mesh data
let mesh_data = MeshData::with_uvs(
    positions,
    uvs,
    indices,
);

// 2. Calculate normals and tangents
mesh_data.calculate_normals();
mesh_data.calculate_tangents();

// 3. Convert to vertices
let vertices = mesh_data.to_vertices();

// 4. Upload to GPU with staging
let gpu_mesh = GpuMesh::new(
    allocator,
    cmd_allocator,
    queue,
    vertices,
    indices,
)?;

// 5. Create descriptor sets (cached)
let descriptor_set = descriptor_cache.get_or_create(key, || {
    create_descriptor_set(&texture, &material)
})?;

// 6. Render
command_buffer
    .bind_vertex_buffers(0, gpu_mesh.vertex_buffer.clone())
    .bind_index_buffer(gpu_mesh.index_buffer.clone())
    .bind_descriptor_sets(PipelineBindPoint::Graphics, pipeline.layout().clone(), 0, descriptor_set)
    .draw_indexed(gpu_mesh.index_count, 1, 0, 0, 0)?;
```

## Summary

The rendering primitives provide:

- **Vertex3D**: Complete vertex format with bytemuck for zero-copy GPU upload
- **Mesh System**: CPU-side definition with efficient GPU upload via staging buffers
- **Buffer Abstractions**: Generic typed buffers with automatic staging and lifetime tracking
- **Texture System**: Image loading and GPU texture management with caching
- **Descriptor Management**: Pooling and LRU eviction for efficient descriptor set reuse
- **Lifetime Tracking**: Frame-based resource lifetime management for GPU synchronization

These primitives form the foundation for all rendering operations in the Praxis engine, providing safety, performance, and ease of use.
