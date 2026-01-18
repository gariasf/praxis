# Line Renderer

Efficient line primitive rendering for debug visualization, editor gizmos, and visual feedback.

## Features

- Colored line segments with per-vertex colors
- Depth testing for proper z-ordering
- Batched rendering for performance
- Visual feedback utilities (grids, axes, bounding boxes)
- Editor integration support

## Quick Start

```rust
use praxis_graphics::{RenderContext, LineBatch, Line};
use praxis_math::Vec3;

// Initialize line renderer
let render_pass = render_context.create_render_pass_with_depth(
    vulkano::format::Format::R8G8B8A8_UNORM
)?;
render_context.initialize_line_renderer(render_pass, [800, 600])?;

// Create line batch
let mut batch = LineBatch::new();

// Add lines
batch.add(
    Vec3::new(0.0, 0.0, 0.0),  // start
    Vec3::new(1.0, 1.0, 1.0),  // end
    Vec3::new(1.0, 0.0, 0.0),  // color (red)
);

// Render
if let Some(line_renderer) = render_context.line_renderer_mut() {
    line_renderer.update_view_projection(view, proj, camera_pos)?;
    // Rendering happens within command buffer recording
}
```

## Visual Feedback Utilities

### Grid Floor

```rust
use praxis_graphics::{create_grid, GridConfig};

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

let outline = create_selection_outline(
    &transform_matrix,
    Vec3::splat(0.6),           // size
    Vec3::new(1.0, 0.5, 0.0),   // orange
);
```

## Rendering Pipeline

### Proper Render Order

For correct depth testing:

1. Clear color and depth buffers
2. Render opaque 3D meshes (write depth)
3. Render lines (test and write depth)
4. Render transparent objects (optional)

### Depth Buffer Setup

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
    renderer.render(&batch)?;  // Don't do this!
}
```

## Shaders

**`line.vert`**: Vertex shader with view/projection transform
**`line.frag`**: Fragment shader with per-vertex color interpolation

Both shaders support depth testing for proper 3D integration.

## Examples

- `examples/line_rendering_demo.rs` - Comprehensive showcase
- `examples/editor_demo.rs` - Editor gizmo integration
- `examples/selection_demo.rs` - Selection visualization

## See Also

- [Line Rendering Guide](../../docs/guides/line-rendering.md)
- Implementation: `crates/praxis_graphics/src/line_renderer.rs`
- Visual feedback: `crates/praxis_graphics/src/visual_feedback.rs`
