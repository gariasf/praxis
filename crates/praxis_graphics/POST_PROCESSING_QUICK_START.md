# Post-Processing Quick Start

Quick reference for using the Praxis post-processing framework.

## Basic Setup (One-Time)

```rust
use praxis_graphics::{PostProcessChain, RenderTargetPool, GrayscalePass};

// Create render pass for post-processing
let render_pass = render_context.create_post_process_render_pass()?;

// Create render target pool
let mut pool = RenderTargetPool::new(
    memory_allocator.clone(),
    render_pass.clone(),
    vulkano::format::Format::R8G8B8A8_UNORM,
);

// Create post-processing chain
let mut chain = PostProcessChain::new(
    command_buffer_allocator.clone(),
    graphics_queue.clone(),
);

// Add effects
let grayscale = GrayscalePass::new(
    device.clone(),
    memory_allocator.clone(),
    vulkano::format::Format::R8G8B8A8_UNORM,
)?;
chain.add_pass(Box::new(grayscale));
```

## Per-Frame Rendering

```rust
// 1. Acquire render targets
let scene_texture = pool.acquire([width, height])?;
let output_texture = pool.acquire([width, height])?;

// 2. Render scene to texture
// (render your 3D scene to scene_texture)

// 3. Apply post-processing
chain.process(&scene_texture, &output_texture, &mut pool)?;

// 4. Blit/copy output to swapchain
// (copy output_texture to screen)

// 5. Release render targets
pool.release(scene_texture);
pool.release(output_texture);
```

## Built-in Effects

```rust
// Copy (passthrough)
let copy = CopyPass::new(device, memory_allocator, format)?;
chain.add_pass(Box::new(copy));

// Grayscale
let grayscale = GrayscalePass::new(device, memory_allocator, format)?;
chain.add_pass(Box::new(grayscale));
```

## Custom Effect Template

```rust
use praxis_graphics::post_process::{PostProcessPass, RenderTarget};

pub struct MyEffect {
    pipeline: Arc<GraphicsPipeline>,
    quad: FullScreenQuad,
    descriptor_set_allocator: Arc<StandardDescriptorSetAllocator>,
}

impl PostProcessPass for MyEffect {
    fn execute(
        &mut self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        input: &RenderTarget,
        output: &RenderTarget,
    ) -> Result<()> {
        // 1. Create descriptor set with input texture
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

        // 2. Begin render pass
        builder.begin_render_pass(
            RenderPassBeginInfo::framebuffer(output.framebuffer().clone()),
            SubpassBeginInfo::default(),
        )?;

        // 3. Set viewport
        builder.set_viewport(0, [Viewport {
            offset: [0.0, 0.0],
            extent: [output.width() as f32, output.height() as f32],
            depth_range: 0.0..=1.0,
        }].into_iter().collect())?;

        // 4. Bind pipeline and draw
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

        // 5. End render pass
        builder.end_render_pass(SubpassEndInfo::default())?;

        Ok(())
    }

    fn name(&self) -> &str {
        "MyEffect"
    }
}
```

## Shader Template

**Vertex Shader** (use standard `post_process.vert`):
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

**Fragment Shader**:
```glsl
#version 450

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(input_texture, in_uv);
    
    // Your effect here
    // Example: invert colors
    out_color = vec4(1.0 - color.rgb, color.a);
}
```

## Common Patterns

### Toggle Effects
```rust
if settings.enable_effect {
    chain.add_pass(Box::new(my_effect));
}
```

### Chain Multiple Effects
```rust
chain.add_pass(Box::new(effect_1));
chain.add_pass(Box::new(effect_2));
chain.add_pass(Box::new(effect_3));
```

### Clear and Rebuild Chain
```rust
chain.clear_passes();
chain.add_pass(Box::new(new_effect));
```

## Performance Tips

1. **Always use RenderTargetPool** - Avoid creating render targets every frame
2. **Reuse descriptor set allocators** - Create once, use throughout lifetime
3. **Minimize texture samples** - Each sample has a cost
4. **Use separable filters** - Split 2D filters into horizontal + vertical passes
5. **Profile effects** - Use GPU profiling tools to identify bottlenecks

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Black screen | Check descriptor set bindings, verify texture formats |
| Crash on render | Ensure render targets are same size as viewport |
| Memory leak | Always release render targets back to pool |
| Poor performance | Use render target pooling, optimize shaders |
| Wrong colors | Check texture format, sampler settings |

## See Also

- `POST_PROCESSING.md` - Full documentation
- `examples/post_process_demo.rs` - Complete example
- `src/post_process/` - Source code
