# Post-Processing Framework

The Praxis post-processing framework provides a flexible, efficient system for applying screen-space effects to rendered scenes.

## Overview

The post-processing system consists of several key components:

- **`PostProcessPass`**: Trait defining a single post-processing effect
- **`RenderTarget`**: Offscreen framebuffer for render-to-texture operations
- **`RenderTargetPool`**: Manages reusable render targets to reduce allocations
- **`FullScreenQuad`**: Renders a full-screen textured quad for applying effects
- **`PostProcessChain`**: Chains multiple post-processing passes together

## Architecture

### Render-to-Texture Flow

```text
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Main      │────▶│    Pass 1   │────▶│    Pass 2   │────▶ Swapchain
│   Render    │     │  (Texture)  │     │  (Texture)  │
└─────────────┘     └─────────────┘     └─────────────┘
   Render to          Apply effect        Apply effect
   texture A          A → B               B → screen
```

### Component Details

#### PostProcessPass

A trait that all post-processing effects must implement:

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

Each pass reads from an input texture, applies an effect, and writes to an output texture.

#### RenderTarget

Represents an offscreen render target consisting of:
- Color attachment image
- Image view for accessing the texture
- Framebuffer for rendering
- Sampler for reading in shaders

```rust
let target = RenderTarget::new(
    memory_allocator,
    render_pass,
    [1920, 1080],
    Format::R8G8B8A8_UNORM,
)?;
```

#### RenderTargetPool

Manages render target lifecycle to avoid repeated allocations:

```rust
let mut pool = RenderTargetPool::new(
    memory_allocator,
    render_pass,
    Format::R8G8B8A8_UNORM,
);

// Acquire a target
let target = pool.acquire([1920, 1080])?;

// Use target...

// Release back to pool
pool.release(target);
```

The pool automatically reuses targets with matching dimensions, creating new ones only when necessary.

#### FullScreenQuad

Provides geometry for rendering full-screen effects:

```rust
let quad = FullScreenQuad::new(memory_allocator)?;

// In render pass:
builder
    .bind_vertex_buffers(0, quad.vertex_buffer().clone())
    .bind_index_buffer(quad.index_buffer().clone())
    .draw_indexed(quad.index_count(), 1, 0, 0, 0)?;
```

The quad covers the entire viewport in clip space [-1, 1] with UV coordinates [0, 1].

#### PostProcessChain

Manages multiple passes and handles ping-pong buffering:

```rust
let mut chain = PostProcessChain::new(
    command_buffer_allocator,
    graphics_queue,
);

// Add passes
chain.add_pass(Box::new(grayscale_pass));
chain.add_pass(Box::new(blur_pass));

// Process texture through chain
chain.process(&input_texture, &output_texture, &mut pool)?;
```

## Built-in Passes

### CopyPass

Simple passthrough that copies input to output. Useful for testing.

```rust
let pass = CopyPass::new(device, memory_allocator, format)?;
```

### GrayscalePass

Converts color to grayscale using standard luminance formula:
`luminance = 0.299*R + 0.587*G + 0.114*B`

```rust
let pass = GrayscalePass::new(device, memory_allocator, format)?;
```

## Creating Custom Passes

### Step 1: Write Shaders

Create vertex and fragment shaders for your effect.

**Vertex Shader** (`my_effect.vert`):
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

**Fragment Shader** (`my_effect.frag`):
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

### Step 2: Register Shaders

Add to `src/shaders.rs`:

```rust
pub mod my_effect_vs {
    vulkano_shaders::shader! {
        ty: "vertex",
        path: "src/shaders/my_effect.vert"
    }
}

pub mod my_effect_fs {
    vulkano_shaders::shader! {
        ty: "fragment",
        path: "src/shaders/my_effect.frag"
    }
}
```

### Step 3: Implement PostProcessPass

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
        // Create render pass
        let render_pass = create_post_process_render_pass(device.clone(), format)?;

        // Load shaders
        let vs_module = shaders::my_effect_vs::load(device.clone())?;
        let fs_module = shaders::my_effect_fs::load(device.clone())?;

        // Create pipeline (see CopyPass for full example)
        // ...

        // Create full-screen quad
        let quad = FullScreenQuad::new(memory_allocator)?;

        // Create descriptor set allocator
        let descriptor_set_allocator = Arc::new(
            StandardDescriptorSetAllocator::new(device, Default::default())
        );

        Ok(Self {
            pipeline,
            quad,
            descriptor_set_allocator,
            render_pass,
        })
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
        let layout = self.pipeline.layout().set_layouts()[0].clone();
        let descriptor_set = DescriptorSet::new(
            self.descriptor_set_allocator.clone(),
            layout,
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

        // Bind pipeline and descriptor set
        builder
            .bind_pipeline_graphics(self.pipeline.clone())?
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline.layout().clone(),
                0,
                descriptor_set,
            )?;

        // Draw full-screen quad
        builder
            .bind_vertex_buffers(0, self.quad.vertex_buffer().clone())?
            .bind_index_buffer(self.quad.index_buffer().clone())?
            .draw_indexed(self.quad.index_count(), 1, 0, 0, 0)?;

        // End render pass
        builder.end_render_pass(SubpassEndInfo::default())?;

        Ok(())
    }

    fn name(&self) -> &str {
        "MyEffect"
    }
}
```

## Usage Example

### Basic Setup

```rust
use praxis_graphics::{
    PostProcessChain, RenderTargetPool, GrayscalePass, CopyPass,
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

// Create and add passes
let grayscale = GrayscalePass::new(
    device.clone(),
    memory_allocator.clone(),
    Format::R8G8B8A8_UNORM,
)?;
chain.add_pass(Box::new(grayscale));
```

### Render Loop

```rust
// 1. Render scene to texture
let scene_texture = pool.acquire([width, height])?;
render_scene_to_texture(&scene_texture);

// 2. Apply post-processing
let output_texture = pool.acquire([width, height])?;
chain.process(&scene_texture, &output_texture, &mut pool)?;

// 3. Blit to swapchain
blit_to_screen(&output_texture);

// 4. Release render targets
pool.release(scene_texture);
pool.release(output_texture);
```

## Performance Considerations

### Render Target Pooling

Always use `RenderTargetPool` to avoid repeated GPU memory allocations:

```rust
// ❌ Bad: Creates new render target every frame
let target = RenderTarget::new(...)?;

// ✅ Good: Reuses render targets
let target = pool.acquire([width, height])?;
// ... use target ...
pool.release(target);
```

### Pass Ordering

Order passes from cheapest to most expensive to fail fast if there are issues:

```rust
chain.add_pass(Box::new(cheap_pass));
chain.add_pass(Box::new(medium_pass));
chain.add_pass(Box::new(expensive_pass));
```

### Shader Optimization

- Use efficient algorithms (e.g., separable blur instead of 2D convolution)
- Minimize texture samples
- Take advantage of GPU features (texture filtering, etc.)

## Common Patterns

### Conditional Effects

```rust
// Enable/disable effects based on settings
chain.clear_passes();
if settings.grayscale {
    chain.add_pass(Box::new(grayscale_pass));
}
if settings.blur {
    chain.add_pass(Box::new(blur_pass));
}
```

### Multi-Pass Effects

Chain multiple passes for complex effects:

```rust
// Bloom effect: blur + combine
chain.add_pass(Box::new(extract_bright_pass));
chain.add_pass(Box::new(blur_horizontal_pass));
chain.add_pass(Box::new(blur_vertical_pass));
chain.add_pass(Box::new(combine_pass));
```

### Push Constants

Use push constants for effect parameters:

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

## Shader Reference

### Standard Vertex Shader

All post-processing passes can use the standard vertex shader:

```glsl
#version 450

layout(location = 0) in vec2 position;  // [-1, 1]
layout(location = 1) in vec2 uv;        // [0, 1]

layout(location = 0) out vec2 out_uv;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    out_uv = uv;
}
```

### Fragment Shader Template

```glsl
#version 450

layout(location = 0) in vec2 in_uv;

layout(set = 0, binding = 0) uniform sampler2D input_texture;

layout(location = 0) out vec4 out_color;

void main() {
    vec4 color = texture(input_texture, in_uv);
    
    // Apply effect
    // ...
    
    out_color = color;
}
```

## Troubleshooting

### Black Screen

- Check that render targets have correct format
- Verify descriptor sets are bound correctly
- Ensure viewport matches render target dimensions

### Performance Issues

- Use `RenderTargetPool` to avoid allocations
- Profile with GPU debugging tools
- Consider lower-resolution intermediate targets

### Incorrect Colors

- Check texture format matches shader expectations
- Verify sampler settings (clamp vs. repeat, filtering)
- Check color space conversions

## Future Enhancements

Potential additions to the framework:

- [ ] Depth-aware effects (edge detection, SSAO)
- [ ] Compute shader support for parallel effects
- [ ] Temporal effects (motion blur, TAA)
- [ ] HDR rendering and tone mapping
- [ ] Effect composition (blending multiple effects)
