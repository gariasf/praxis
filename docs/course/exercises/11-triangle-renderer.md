# Exercise 11: Triangle Renderer

**Difficulty**: 🟢 Beginner | **Estimated Time**: 2-3h | **Subsystem**: Graphics

## Overview

Render a colored triangle using Vulkan - the "Hello World" of graphics programming. This establishes the foundation for all rendering in the engine.

## Learning Objectives

- Understand Vulkan graphics pipeline setup
- Learn vertex buffer creation and management
- Implement basic vertex and fragment shaders
- Grasp the render pass concept

## Requirements

### Functional Requirements

1. **Vertex Buffer**
   - Create buffer with 3 vertices (triangle)
   - Each vertex: position (vec3) and color (vec3)
   - Upload to GPU memory

2. **Shaders**
   - Vertex shader: transform vertices, pass through color
   - Fragment shader: output interpolated color
   - Compile GLSL to SPIR-V

3. **Pipeline**
   - Create graphics pipeline with shaders
   - Configure vertex input state
   - Set up rasterization and blending

4. **Rendering**
   - Clear screen to black
   - Draw triangle
   - Present to screen

### Non-Functional Requirements

- **Performance**: Render at 60+ FPS
- **Correctness**: Colors interpolated smoothly across triangle
- **Safety**: No validation layer errors

## API Design

```rust
pub struct TriangleRenderer {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pipeline: Arc<GraphicsPipeline>,
}

#[derive(Default, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl TriangleRenderer {
    pub fn new(device: Arc<Device>, render_pass: Arc<RenderPass>) -> Result<Self>;
    pub fn render(&self, command_buffer: &mut AutoCommandBufferBuilder);
}
```

## Validation Criteria

### Correctness
- [ ] Triangle visible on screen
- [ ] Colors interpolate correctly (red at top, green/blue at bottom corners)
- [ ] Triangle centered in viewport
- [ ] No Vulkan validation errors

### Performance
- [ ] 60+ FPS on integrated GPU
- [ ] GPU usage < 10%
- [ ] Frame time < 1ms for rendering

### Code Quality
- [ ] Clean resource management
- [ ] Proper error handling
- [ ] Shader code documented

## Expected Behavior

1. **Window Opens**: Black background
2. **Triangle Renders**: 
   - Top vertex: Red (1, 0, 0)
   - Bottom-left: Green (0, 1, 0)  
   - Bottom-right: Blue (0, 0, 1)
3. **Smooth Gradients**: Colors blend smoothly across surface
4. **Stable**: No flickering, consistent frame rate

## Shaders

### Vertex Shader (GLSL)
```glsl
#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 color;

layout(location = 0) out vec3 fragColor;

void main() {
    gl_Position = vec4(position, 1.0);
    fragColor = color;
}
```

### Fragment Shader (GLSL)
```glsl
#version 450

layout(location = 0) in vec3 fragColor;
layout(location = 0) out vec4 outColor;

void main() {
    outColor = vec4(fragColor, 1.0);
}
```

## Test Cases

```rust
#[test]
fn test_vertex_buffer_creation() {
    let (device, queue) = create_test_device();
    
    let vertices = [
        Vertex { position: [0.0, -0.5, 0.0], color: [1.0, 0.0, 0.0] },
        Vertex { position: [-0.5, 0.5, 0.0], color: [0.0, 1.0, 0.0] },
        Vertex { position: [0.5, 0.5, 0.0], color: [0.0, 0.0, 1.0] },
    ];
    
    let buffer = CpuAccessibleBuffer::from_iter(
        device.clone(),
        BufferUsage::vertex_buffer(),
        false,
        vertices.iter().cloned(),
    );
    
    assert!(buffer.is_ok());
}

#[test]
fn test_pipeline_creation() {
    let (device, _) = create_test_device();
    let render_pass = create_test_render_pass(device.clone());
    
    let renderer = TriangleRenderer::new(device, render_pass);
    assert!(renderer.is_ok());
}
```

## Performance Targets

| Metric | Target |
|--------|--------|
| Frame rate | 60+ FPS |
| Draw call time | < 0.5ms |
| GPU memory | < 1MB |
| CPU overhead | < 0.1ms |

## Reference Implementation

### Rust (with vulkano)

<details>
<summary>Click to reveal Rust implementation</summary>

```rust
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::device::{Device, Queue};
use vulkano::pipeline::graphics::vertex_input::BuffersDefinition;
use vulkano::pipeline::graphics::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, Pipeline};
use vulkano::render_pass::{Framebuffer, RenderPass, Subpass};
use std::sync::Arc;

#[derive(Default, Copy, Clone)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

vulkano::impl_vertex!(Vertex, position, color);

pub struct TriangleRenderer {
    vertex_buffer: Arc<CpuAccessibleBuffer<[Vertex]>>,
    pipeline: Arc<GraphicsPipeline>,
}

impl TriangleRenderer {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        viewport: Viewport,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Define triangle vertices
        let vertices = [
            Vertex {
                position: [0.0, -0.5, 0.0],
                color: [1.0, 0.0, 0.0], // Red
            },
            Vertex {
                position: [-0.5, 0.5, 0.0],
                color: [0.0, 1.0, 0.0], // Green
            },
            Vertex {
                position: [0.5, 0.5, 0.0],
                color: [0.0, 0.0, 1.0], // Blue
            },
        ];

        // Create vertex buffer
        let vertex_buffer = CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::vertex_buffer(),
            false,
            vertices.iter().cloned(),
        )?;

        // Load shaders
        mod vs {
            vulkano_shaders::shader! {
                ty: "vertex",
                src: "
                    #version 450
                    layout(location = 0) in vec3 position;
                    layout(location = 1) in vec3 color;
                    layout(location = 0) out vec3 fragColor;
                    
                    void main() {
                        gl_Position = vec4(position, 1.0);
                        fragColor = color;
                    }
                "
            }
        }

        mod fs {
            vulkano_shaders::shader! {
                ty: "fragment",
                src: "
                    #version 450
                    layout(location = 0) in vec3 fragColor;
                    layout(location = 0) out vec4 outColor;
                    
                    void main() {
                        outColor = vec4(fragColor, 1.0);
                    }
                "
            }
        }

        let vs = vs::load(device.clone())?;
        let fs = fs::load(device.clone())?;

        // Create pipeline
        let pipeline = GraphicsPipeline::start()
            .vertex_input_state(BuffersDefinition::new().vertex::<Vertex>())
            .vertex_shader(vs.entry_point("main").unwrap(), ())
            .viewport_state(viewport.clone())
            .fragment_shader(fs.entry_point("main").unwrap(), ())
            .render_pass(Subpass::from(render_pass, 0).unwrap())
            .build(device)?;

        Ok(Self {
            vertex_buffer,
            pipeline,
        })
    }

    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    ) {
        builder
            .bind_pipeline_graphics(self.pipeline.clone())
            .bind_vertex_buffers(0, self.vertex_buffer.clone())
            .draw(self.vertex_buffer.len() as u32, 1, 0, 0)
            .unwrap();
    }
}

// Example main loop integration
fn main() {
    // Initialize Vulkan (device, queue, surface, swapchain)
    // ...
    
    let renderer = TriangleRenderer::new(device, render_pass, viewport).unwrap();
    
    loop {
        // Acquire swapchain image
        // Begin render pass
        
        let mut builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            queue.family(),
            CommandBufferUsage::OneTimeSubmit,
        ).unwrap();
        
        renderer.render(&mut builder);
        
        // End render pass
        // Submit and present
    }
}
```

</details>

## Related Resources

- [Vulkan Tutorial](https://vulkan-tutorial.com/)
- [Vulkano Guide](https://vulkano.rs/guide/introduction)
- [Learn OpenGL - Hello Triangle](https://learnopengl.com/Getting-started/Hello-Triangle)

## Next Steps

- Add texture mapping (Exercise 12)
- Implement transformations with matrices
- Study `praxis_graphics` rendering pipeline
