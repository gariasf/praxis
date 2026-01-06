# GPU Procedural Generation Implementation

This document describes the GPU-based procedural texture generation system implementation.

## Overview

The procedural texture system generates textures entirely on the GPU using compute shaders. The process involves:

1. **Graph compilation**: Converting a `TextureGraph` into GLSL compute shader source code
2. **Shader compilation**: Compiling GLSL to SPIR-V bytecode using `shaderc`
3. **Pipeline creation**: Creating a Vulkan compute pipeline with the compiled shader
4. **GPU dispatch**: Dispatching compute work to generate the texture
5. **Readback**: Copying the generated texture data back to CPU memory

## Architecture

```
TextureGraph
     ↓
[Shader Generation]
     ↓
GLSL Source Code
     ↓
[shaderc Compiler]
     ↓
SPIR-V Bytecode
     ↓
[Vulkan Pipeline]
     ↓
GPU Compute Dispatch
     ↓
[Copy to Buffer]
     ↓
RGBA8 Texture Data
```

## Shader Generation

### Node-Based Code Generation

Each `TextureNode` in the graph is converted to a GLSL function:

```glsl
// Example: Perlin noise node
vec4 eval_node_0(vec2 uv) {
    float value = fbm_perlin_noise(uv * 8.0, SEED, 4, 0.5, 2.0);
    value = value * 0.5 + 0.5;
    return vec4(value, value, value, 1.0);
}
```

Functions are generated recursively for dependent nodes, ensuring proper evaluation order.

### Noise Functions

The noise functions are implemented in `shaders/noise_functions.glsl` and match the CPU implementations:

- **Perlin Noise**: Classic gradient noise with fade function
- **Simplex Noise**: Improved Perlin with skewed grid
- **Worley Noise**: Cellular/Voronoi patterns based on feature points

Each noise function supports Fractal Brownian Motion (fBm) for multi-octave detail.

### Compute Shader Structure

```glsl
#version 450

layout(local_size_x = 16, local_size_y = 16, local_size_z = 1) in;
layout(set = 0, binding = 0, rgba8) uniform writeonly image2D outputImage;

const uint SEED = 42u;
const uint WIDTH = 512u;
const uint HEIGHT = 512u;

// Noise functions (from noise_functions.glsl)
// ...

// Generated node evaluation functions
vec4 eval_node_0(vec2 uv) { /* ... */ }
vec4 eval_node_1(vec2 uv) { /* ... */ }
// ...

void main() {
    ivec2 pixel = ivec2(gl_GlobalInvocationID.xy);
    if (pixel.x >= WIDTH || pixel.y >= HEIGHT) return;
    
    vec2 uv = vec2(pixel) / vec2(WIDTH, HEIGHT);
    vec4 color = eval_node_N(uv);  // N = output node
    imageStore(outputImage, pixel, color);
}
```

## Pipeline Creation

### Shader Compilation

GLSL source is compiled to SPIR-V using the `shaderc` library:

```rust
let compiler = Compiler::new()?;
let mut options = CompileOptions::new()?;
options.set_target_env(TargetEnv::Vulkan, EnvVersion::Vulkan1_2);
options.set_optimization_level(OptimizationLevel::Performance);

let binary = compiler.compile_into_spirv(
    source,
    ShaderKind::Compute,
    "shader.comp",
    "main",
    Some(&options)
)?;
```

### Vulkan Pipeline

A compute pipeline is created with the compiled shader:

```rust
let shader_module = ShaderModule::from_bytes(device.clone(), &spirv_bytes)?;
let entry_point = shader_module.entry_point("main")?;
let stage = PipelineShaderStageCreateInfo::new(entry_point);

let layout = PipelineLayout::new(
    device.clone(),
    PipelineDescriptorSetLayoutCreateInfo::from_stages([&stage])
        .into_pipeline_layout_create_info(device.clone())?
)?;

let pipeline = ComputePipeline::new(
    device.clone(),
    None,
    ComputePipelineCreateInfo::stage_layout(stage, layout)
)?;
```

## GPU Execution

### Dispatch Configuration

Compute work is dispatched in 16x16 workgroups:

```rust
let workgroup_size = 16;
let dispatch_x = (width + workgroup_size - 1) / workgroup_size;
let dispatch_y = (height + workgroup_size - 1) / workgroup_size;

builder
    .bind_pipeline_compute(pipeline)
    .bind_descriptor_sets(...)
    .dispatch([dispatch_x, dispatch_y, 1])?;
```

### Resource Management

1. **Output Image**: Created with `STORAGE` and `TRANSFER_SRC` usage
2. **Descriptor Set**: Binds output image to shader
3. **Readback Buffer**: Host-visible buffer for copying result

### Synchronization

```rust
let future = sync::now(device)
    .then_execute(queue, command_buffer)
    .then_signal_fence_and_flush()?;

future.wait(None)?;
```

## Performance Characteristics

### Benchmarks

For a 512x512 texture (1 megapixel, 4MB):

- **Simple Perlin**: ~5ms
- **Complex blend**: ~8ms
- **With color ramp**: ~6ms

Performance scales with:
- Texture resolution (linear with pixel count)
- Graph complexity (more nodes = more shader operations)
- Octave count (each octave doubles work)

### Optimization Strategies

1. **Workgroup Size**: 16x16 threads optimal for most GPUs
2. **Shader Optimization**: Performance level enabled in shaderc
3. **Early Out**: Bounds checking to skip out-of-range pixels
4. **Caching**: Avoid regenerating identical textures

## Memory Layout

### RGBA8 Format

Textures are generated in RGBA8_UNORM format:
- 4 bytes per pixel
- 8 bits per channel
- Value range: [0, 255]

### Buffer Layout

```
Pixel (0,0): [R, G, B, A]
Pixel (1,0): [R, G, B, A]
...
Pixel (width-1, height-1): [R, G, B, A]
```

## Error Handling

### Shader Compilation Errors

When shader compilation fails:
1. Error message includes line numbers
2. Source code can be logged for debugging
3. Warnings are logged but don't fail compilation

### GPU Errors

- Device lost: Propagated as error
- Out of memory: Caught and reported
- Invalid pipeline: Caught during creation

## Integration with Graphics System

The generated texture data is uploaded to GPU textures:

```rust
// Create GPU image
let image = Image::new(
    memory_allocator,
    ImageCreateInfo {
        format: Format::R8G8B8A8_SRGB,
        usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
        // ...
    },
    allocation_info
)?;

// Copy from staging buffer
builder.copy_buffer_to_image(
    CopyBufferToImageInfo::buffer_image(staging_buffer, image)
)?;

// Create texture
let texture = Texture {
    image,
    view: ImageView::new_default(image)?,
    sampler: Sampler::new(device, sampler_info)?,
    width,
    height,
};
```

## Future Enhancements

### Pipeline Caching

Currently, each unique graph compiles a new shader. Future optimization:
- Cache compiled pipelines by graph hash
- Reduce compilation overhead for repeated use

### Async Generation

Currently synchronous with GPU wait. Future improvement:
- Queue multiple generations
- Use fences for async readback
- Overlap CPU and GPU work

### 3D Textures

Extend to volume textures:
- 3D noise sampling
- Volumetric effects (clouds, fog)
- Additional workgroup dimension

### Persistent Kernels

For animation or interactive editing:
- Keep pipeline alive
- Update uniforms only
- Avoid shader recompilation

## Testing

### Unit Tests

- Shader source generation validation
- Node function generation correctness
- Graph traversal and code emission

### Integration Tests

- Full GPU generation (requires Vulkan)
- Cache hit/miss behavior
- Error handling paths

### Example Programs

- `procedural_texture_demo.rs`: Visual demonstration
- Multiple texture types
- Real-time regeneration with new seeds

## Dependencies

### Required

- **vulkano**: Vulkan bindings and resource management
- **shaderc**: GLSL to SPIR-V compilation
- **praxis_utils**: Logging and error handling

### Build Requirements

- Vulkan SDK (for shaderc)
- C++ compiler (shaderc dependency)

## Limitations

### Hardware Requirements

- Vulkan 1.2 support required
- Compute shader capability
- Image storage operations

### Size Limits

- Max texture size: GPU dependent (typically 16384x16384)
- Max workgroups: GPU dependent
- Memory: Limited by GPU VRAM

### Graph Complexity

- No practical limit on node count
- Deep graphs may hit shader recursion limits
- Very wide graphs create large shaders
