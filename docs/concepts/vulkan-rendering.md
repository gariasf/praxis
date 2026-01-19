# Vulkan Rendering

Praxis uses Vulkan for graphics rendering via the `vulkano` crate, providing high-performance, cross-platform 3D graphics.

## Graphics Pipeline Overview

```
Application → Command Buffer → GPU Queue → Swapchain → Display
     ↓              ↓              ↓            ↓
  Draw calls   GPU commands   Execution    Present
```

### Key Vulkan Concepts

**Device**: Represents the GPU. Created once at startup.

**Queue**: Where GPU commands are submitted. Praxis uses a graphics queue.

**Swapchain**: Double/triple buffering for smooth presentation. Manages images shown on screen.

**Command Buffer**: Records GPU commands (draw calls, state changes). Submitted to queue.

**Pipeline**: Complete GPU state for rendering—shaders, vertex format, blend modes.

**Descriptor Sets**: Bind resources (textures, buffers) to shaders.

## Rendering Frame Lifecycle

1. **Acquire swapchain image** - Get next available framebuffer
2. **Record command buffer** - Build list of draw commands
3. **Submit to queue** - Send commands to GPU
4. **Present** - Display the rendered frame

```rust
// Simplified frame loop
let (image_index, acquire_future) = swapchain.acquire_next_image()?;
let command_buffer = record_commands(image_index)?;
let execution = acquire_future
    .then_execute(queue, command_buffer)?
    .then_swapchain_present(swapchain, image_index);
execution.flush()?;
```

## Shader Pipeline

### Vertex Shader
Transforms vertices from model space to clip space:

```glsl
layout(set = 0, binding = 0) uniform Uniforms {
    mat4 model;
    mat4 view;
    mat4 proj;
};

void main() {
    gl_Position = proj * view * model * vec4(position, 1.0);
}
```

### Fragment Shader
Computes final pixel color:

```glsl
layout(set = 0, binding = 1) uniform sampler2D tex;

void main() {
    vec4 color = texture(tex, uv);
    // Apply lighting, materials, etc.
    f_color = color;
}
```

## Descriptor Sets

Descriptor sets bind resources to shader bindings:

```
Set 0: Per-frame data (view/projection matrices)
Set 1: Per-material data (textures, material properties)
Set 2: Per-object data (model matrix)
```

Praxis batches objects by material to minimize descriptor set changes.

## Memory Management

**Host-visible memory**: CPU can write, slower GPU access. Used for uniforms.

**Device-local memory**: Fast GPU access, CPU can't directly write. Used for meshes/textures.

**Staging buffers**: Temporary host-visible buffers for uploading to device memory.

## Synchronization

Vulkan requires explicit synchronization:

- **Fences**: CPU waits for GPU
- **Semaphores**: GPU-GPU synchronization between queue operations
- **Pipeline barriers**: Synchronize memory access within command buffer

`vulkano` handles most synchronization automatically through its future system.

## Performance Considerations

### Batching
Group draw calls by:
1. Pipeline state (least to most expensive to change)
2. Descriptor sets
3. Vertex/index buffers

### Command Buffer Recording
- Use secondary command buffers for static geometry
- Re-record primary buffers each frame for dynamic content

### Memory Allocation
- Pool allocations to avoid fragmentation
- Use suballocation for small resources

## See Also

- [Beginner's Guide: Rendering Pipeline Flow](../beginners-guide.md#rendering-pipeline-flow) - Complete pipeline walkthrough
- [Beginner's Guide: Uniform Buffers](../beginners-guide.md#uniform-buffers-and-descriptor-sets) - Descriptor set deep dive
- [Rendering Guide](../guides/rendering.md) - Practical usage
- [Rendering API Reference](../reference/rendering-api.md) - API documentation
- [Rendering Learning Path](../learning-paths/rendering.md) - Structured learning progression
- [praxis_graphics Crate](../../crates/praxis_graphics/README.md) - Crate documentation
