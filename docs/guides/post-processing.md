# Post-Processing

Screen-space effects framework for the Praxis engine.

## Overview

Post-processing applies full-screen effects after the main render pass. Effects are chained together, each reading from the previous result.

```text
Scene → Effect1 → Effect2 → Effect3 → Output
```

## Core Components

### PostProcessPass Trait

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

### RenderTargetPool

Manages render target lifecycle efficiently:

```rust
let mut pool = RenderTargetPool::new(memory_allocator, render_pass);

// Acquire targets (reuses from pool when available)
let target = pool.acquire([width, height])?;

// ... use target ...

// Return to pool for reuse
pool.release(target);
```

### PostProcessChain

Orchestrates multiple passes:

```rust
let mut chain = PostProcessChain::new(device, queue);
chain.add_pass(Box::new(grayscale_pass));
chain.add_pass(Box::new(vignette_pass));
chain.process(&input, &output, &mut pool)?;
```

## Built-in Effects

| Effect | Description | Performance |
|--------|-------------|-------------|
| CopyPass | Reference implementation | ~0.1ms |
| GrayscalePass | Luminance conversion | ~0.15ms |

## Creating Custom Effects

```rust
pub struct MyEffect {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl PostProcessPass for MyEffect {
    fn execute(&mut self, builder: &mut AutoCommandBufferBuilder<_>,
               input: &RenderTarget, output: &RenderTarget) -> Result<()> {
        // Bind pipeline, render full-screen quad
        Ok(())
    }

    fn name(&self) -> &str { "MyEffect" }
}
```

### Standard Fragment Shader Pattern

```glsl
#version 450

layout(location = 0) in vec2 in_uv;
layout(set = 0, binding = 0) uniform sampler2D input_texture;
layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(input_texture, in_uv);
    // Apply your effect here
    out_color = color;
}
```

## Performance

Memory usage per frame (ping-pong buffers):
- 1080p: ~16 MB
- 4K: ~64 MB

Render target pooling reduces allocations by ~100x.

## Integration

```rust
// Create resources once
let render_pass = render_context.create_post_process_render_pass()?;
let mut pool = RenderTargetPool::new(allocator, render_pass);
let mut chain = PostProcessChain::new(device, queue);

chain.add_pass(Box::new(GrayscalePass::new(...)?));

// Per-frame
let input = pool.acquire([w, h])?;
let output = pool.acquire([w, h])?;
chain.process(&input, &output, &mut pool)?;
pool.release(input);
pool.release(output);
```

## See Also

- [HDR and Tone Mapping](hdr-and-tonemapping.md) - HDR pipeline and tone mapping
- [crates/praxis_graphics/POST_PROCESSING.md](../../crates/praxis_graphics/POST_PROCESSING.md) - Full API docs
- [docs/bloom_effect.md](../bloom_effect.md) - Bloom implementation
