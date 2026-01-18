# Post-Processing Framework

Flexible, efficient system for applying screen-space effects to rendered scenes.

## Overview

The post-processing framework provides:

- **`PostProcessPass`**: Trait for implementing effects
- **`RenderTarget`**: Offscreen framebuffers for render-to-texture
- **`RenderTargetPool`**: Manages reusable render targets
- **`FullScreenQuad`**: Geometry for full-screen effects
- **`PostProcessChain`**: Chains multiple passes together

## Quick Start

```rust
use praxis_graphics::{
    PostProcessChain, RenderTargetPool, GrayscalePass,
};

// Initialize render target pool
let mut pool = RenderTargetPool::new(
    memory_allocator.clone(),
    render_pass.clone(),
    Format::R8G8B8A8_UNORM,
);

// Create post-processing chain
let mut chain = PostProcessChain::new(
    command_buffer_allocator.clone(),
    graphics_queue.clone(),
);

// Add passes
let grayscale = GrayscalePass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
)?;
chain.add_pass(Box::new(grayscale));

// Render loop
let scene_texture = pool.acquire([width, height])?;
render_scene_to_texture(&scene_texture);

let output_texture = pool.acquire([width, height])?;
chain.process(&scene_texture, &output_texture, &mut pool)?;

blit_to_screen(&output_texture);

pool.release(scene_texture);
pool.release(output_texture);
```

## Architecture

### Render-to-Texture Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Main      │────▶│    Pass 1   │────▶│    Pass 2   │────▶ Screen
│   Render    │     │  (Texture)  │     │  (Texture)  │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Components

### PostProcessPass

Trait for implementing effects:

```rust
pub trait PostProcessPass: Send + Sync {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()>;

    fn name(&self) -> &str;
}
```

### RenderTarget

Offscreen render target:

```rust
let target = RenderTarget::new(
    memory_allocator,
    render_pass,
    [1920, 1080],
    Format::R8G8B8A8_UNORM,
)?;
```

**Components:**
- Color attachment image
- Image view for sampling
- Framebuffer for rendering
- Sampler for shader access

### RenderTargetPool

Manages render target lifecycle:

```rust
let mut pool = RenderTargetPool::new(
    memory_allocator,
    render_pass,
    Format::R8G8B8A8_UNORM,
);

// Acquire
let target = pool.acquire([1920, 1080])?;

// Use...

// Release back to pool
pool.release(target);
```

Automatically reuses targets with matching dimensions.

### FullScreenQuad

Geometry for full-screen rendering:

```rust
let quad = FullScreenQuad::new(memory_allocator)?;

builder
    .bind_vertex_buffers(0, quad.vertex_buffer().clone())
    .bind_index_buffer(quad.index_buffer().clone())
    .draw_indexed(quad.index_count(), 1, 0, 0, 0)?;
```

Covers viewport in clip space [-1, 1] with UV [0, 1].

### PostProcessChain

Chains multiple passes:

```rust
let mut chain = PostProcessChain::new(
    command_buffer_allocator,
    graphics_queue,
);

chain.add_pass(Box::new(grayscale_pass));
chain.add_pass(Box::new(blur_pass));

chain.process(&input, &output, &mut pool)?;
```

## Built-in Passes

### CopyPass

Simple passthrough for testing:

```rust
let pass = CopyPass::new(device, memory_allocator, format)?;
```

### GrayscalePass

Color to grayscale conversion:

```rust
let pass = GrayscalePass::new(device, memory_allocator, format)?;
```

**Formula:** `luminance = 0.299*R + 0.587*G + 0.114*B`

## Creating Custom Passes

### Step 1: Write Shaders

**Vertex (`effect.vert`):**
```glsl
#version 450

layout(location = 0) in vec2 position;
layout(location = 1) in vec2 uv;
layout(location = 0) out vec2 out_uv;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_uv = uv;
}
```

**Fragment (`effect.frag`):**
```glsl
#version 450

layout(location = 0) in vec2 in_uv;
layout(set = 0, binding = 0) uniform sampler2D input_texture;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(input_texture, in_uv);
    // Apply effect...
    out_color = color;
}
```

### Step 2: Implement PostProcessPass

```rust
pub struct MyEffectPass {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
    render_pass: Arc<RenderPass>,
}

impl MyEffectPass {
    pub fn new(
        device: Arc<Device>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        format: Format,
    ) -> Result<Self> {
        // Load shaders and create pipeline...
        // Create full-screen quad...
        // Create descriptor set allocator...
        Ok(Self { /* ... */ })
    }
}

impl PostProcessPass for MyEffectPass {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        // Create descriptor set with input texture
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            self.pipeline.layout().set_layouts()[0].clone(),
            [WriteDescriptorSet::image_view_sampler(
                0,
                input.image_view().clone(),
                input.sampler().clone(),
            )],
            [],
        )?;

        // Begin render pass
        builder.begin_render_pass(
            RenderPassBeginInfo {
                clear_values: vec![Some([0.0, 0.0, 0.0, 1.0].into())],
                ..RenderPassBeginInfo::framebuffer(output.framebuffer().clone())
            },
            SubpassBeginInfo::default(),
        )?;

        // Set viewport
        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
            depth_range: 0.0..=1.0,
        };
        builder.set_viewport(0, [viewport].into_iter().collect())?;

        // Bind and draw
        builder
            .bind_pipeline_graphics(self.pipeline.clone())?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )?
            .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())?
            .bind_index_buffer(self.quad.index_buffer().clone())?
            .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)?;

        builder.end_render_pass(SubpassEndInfo::default())?;

        Ok(())
    }

    fn name(&self) -> &str {
        "MyEffect"
    }
}
```

## Performance

### Render Target Pooling

Always use `RenderTargetPool`:

```rust
// ❌ Bad: Creates new every frame
let target = RenderTarget::new(...)?;

// ✅ Good: Reuses targets
let target = pool.acquire([width, height])?;
// ... use ...
pool.release(target);
```

### Pass Ordering

Order from cheapest to most expensive:

```rust
chain.add_pass(Box::new(cheap_pass));
chain.add_pass(Box::new(medium_pass));
chain.add_pass(Box::new(expensive_pass));
```

### Shader Optimization

- Use efficient algorithms (separable blur vs 2D)
- Minimize texture samples
- Leverage GPU hardware features

## Common Patterns

### Conditional Effects

```rust
chain.clear_passes();
if settings.grayscale {
    chain.add_pass(Box::new(grayscale_pass));
}
if settings.blur {
    chain.add_pass(Box::new(blur_pass));
}
```

### Multi-Pass Effects

```rust
// Bloom: extract bright + blur + combine
chain.add_pass(Box::new(extract_bright_pass));
chain.add_pass(Box::new(blur_horizontal_pass));
chain.add_pass(Box::new(blur_vertical_pass));
chain.add_pass(Box::new(combine_pass));
```

### Push Constants

```glsl
layout(push_constant) uniform Params {
    float intensity;
    vec2 direction;
} params;
```

```rust
builder.push_constants(
    pipeline.layout().clone(),
    0,
    bytemuck::bytes_of(&params),
)?;
```

## Troubleshooting

### Black Screen
- Check render targets have correct format
- Verify descriptor sets bound correctly
- Ensure viewport matches render target dimensions

### Performance Issues
- Use `RenderTargetPool` to avoid allocations
- Profile with GPU debugging tools
- Consider lower-resolution intermediate targets

### Incorrect Colors
- Check texture format matches shader
- Verify sampler settings
- Check color space conversions

## See Also

- [HDR Rendering](HDR_RENDERING.md)
- Implementation: `crates/praxis_graphics/src/post_processing.rs`
- Examples: `examples/post_process_demo.rs`
