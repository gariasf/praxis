# Line Renderer

Efficient line primitive rendering system for debug visualization, editor gizmos, and visual feedback.

## Features

- ✅ **Colored line segments** with per-vertex colors
- ✅ **Depth testing** for proper z-ordering with 3D geometry
- ✅ **Batched rendering** for performance
- ✅ **Visual feedback utilities** for common patterns
- ✅ **Editor integration** ready for gizmos and selection

## Quick Start

### 1. Initialize Line Renderer

```rust
use praxis_graphics::RenderContext;

// Create render pass with depth buffer
let render_pass = render_context.create_render_pass_with_depth(
    vulkano::format::Format::R8G8B8A8_UNORM
)?;

// Initialize line renderer
render_context.initialize_line_renderer(render_pass, [800, 600])?;
```

### 2. Create and Render Lines

```rust
use praxis_graphics::{LineBatch, Line};
use praxis_math::Vec3;

// Create line batch
let mut batch = LineBatch::new();

// Add lines
batch.add(
    Vec3::new(0.0, 0.0, 0.0),  // start
    Vec3::new(1.0, 1.0, 1.0),  // end
    Vec3::new(1.0, 0.0, 0.0),  // color (red)
);

// Update camera and render
if let Some(line_renderer) = render_context.line_renderer_mut() {
    line_renderer.update_view_projection(view, proj, camera_pos)?;
    // Render call happens within command buffer recording
}
```

## Visual Feedback Utilities

### Grid Floor

```rust
use praxis_graphics::{create_grid, GridConfig};
use praxis_math::Vec3;

let grid = create_grid(&GridConfig {
    size: 20.0,
    divisions: 20,
    line_color: Vec3::new(0.3, 0.3, 0.3),
    axis_color: Vec3::new(0.5, 0.5, 0.5),
    height: 0.0,
});
```

### Axis Indicators

```rust
use praxis_graphics::{create_axis_indicator, AxisIndicatorConfig};

let axes = create_axis_indicator(&AxisIndicatorConfig {
    length: 1.0,
    position: Vec3::ZERO,
    show_labels: false,
});
// Creates X (red), Y (green), Z (blue) axes
```

### Bounding Boxes

```rust
use praxis_graphics::create_bounding_box;

let bbox = create_bounding_box(
    Vec3::new(0.0, 1.0, 0.0),  // center
    Vec3::new(0.5, 0.5, 0.5),  // half-extents
    Vec3::new(1.0, 1.0, 0.0),  // yellow
);
```

### Selection Outlines

```rust
use praxis_graphics::create_selection_outline;
use praxis_math::Mat4;

let outline = create_selection_outline(
    &transform_matrix,
    Vec3::splat(0.6),           // size
    Vec3::new(1.0, 0.5, 0.0),   // orange
);
```

## Editor Integration

### Viewport Gizmos

```rust
use praxis_graphics::{create_gizmo_lines, LineBatch};

impl ViewportPanel {
    pub fn get_gizmo_lines(&self, world: &World) -> Option<LineBatch> {
        if !self.show_gizmos {
            return None;
        }

        let gizmo_system = world.get_resource::<GizmoSystem>()?;
        let gizmo = gizmo_system.active_gizmo()?;
        
        let lines = gizmo.get_lines(gizmo_system.mode(), gizmo_system.space());
        Some(create_gizmo_lines(lines))
    }
}
```

## Rendering Pipeline

### Proper Render Order

For correct depth testing:

1. Clear color and depth buffers
2. Render opaque 3D meshes (write depth)
3. Render lines (test and write depth)
4. Render transparent objects (optional)

### With Depth Buffer

```rust
// Create depth buffer
let depth_image = Image::new(
    memory_allocator.clone(),
    ImageCreateInfo {
        image_type: ImageType::Dim2d,
        format: Format::D32_SFLOAT,
        extent: [width, height, 1],
        usage: ImageUsage::DEPTH_STENCIL_ATTACHMENT,
        ..Default::default()
    },
    AllocationCreateInfo::default(),
)?;

// Create framebuffer with color and depth
let framebuffer = Framebuffer::new(
    render_pass.clone(),
    FramebufferCreateInfo {
        attachments: vec![color_view, depth_view],
        ..Default::default()
    },
)?;
```

## Performance Tips

### ✅ Batch Lines Together

```rust
// Good: 1 batch = 1 draw call
let mut batch = LineBatch::with_capacity(1000);
for line in lines {
    batch.add_line(line);
}
```

### ✅ Pre-allocate Capacity

```rust
// Avoid reallocations
let mut batch = LineBatch::with_capacity(expected_count);
```

### ❌ Avoid Multiple Small Batches

```rust
// Bad: Many draw calls
for line in lines {
    let mut batch = LineBatch::new();
    batch.add_line(line);
    renderer.render(&batch)?; // Don't do this!
}
```

## Examples

- `examples/line_rendering_demo.rs` - Comprehensive showcase
- `examples/editor_demo.rs` - Editor gizmo integration
- `examples/selection_demo.rs` - Selection visualization

## Documentation

- [Line Rendering Guide](../../docs/guides/line-rendering.md)
- Module documentation: `crates/praxis_graphics/src/line_renderer.rs`
- Visual feedback: `crates/praxis_graphics/src/visual_feedback.rs`

## Architecture

```
Application Code
    ↓
LineBatch (CPU)
    ↓
LineRenderer (GPU)
    ↓
Line Shader Pipeline
    ↓
Framebuffer with Depth
```

## Shaders

- `src/shaders/line.vert` - Vertex shader with view/projection transform
- `src/shaders/line.frag` - Fragment shader with per-vertex color interpolation

Both shaders support depth testing for proper 3D integration.
