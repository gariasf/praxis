# Line Rendering Guide

This guide covers the line primitive rendering system in Praxis, which enables debug visualization, gizmo drawing, and visual feedback for editor tools.

## Overview

The line rendering system provides:

- **Efficient batched rendering** of colored line segments
- **Depth testing** for proper z-ordering with 3D geometry
- **Visual feedback utilities** for common patterns (grids, axes, bounding boxes)
- **Editor integration** for gizmo and selection visualization

## Architecture

### Core Components

1. **`LineVertex`** - Vertex format with position (Vec3) and color (Vec3)
2. **`Line`** - Single line segment with start, end, and color
3. **`LineBatch`** - Collection of lines for efficient batch rendering
4. **`LineRenderer`** - GPU renderer with its own pipeline and depth testing

### Rendering Pipeline

```
LineVertex → LineBatch → LineRenderer → GPU
   (CPU)        (CPU)      (Vulkan)    (Render)
```

## Basic Usage

### 1. Initialize the Line Renderer

The line renderer requires a render pass with depth buffer support:

```rust
use praxis_graphics::RenderContext;

// Create render pass with depth buffer
let render_pass = render_context.create_render_pass_with_depth(
    vulkano::format::Format::R8G8B8A8_UNORM
)?;

// Initialize line renderer
render_context.initialize_line_renderer(render_pass, [800, 600])?;
```

### 2. Create Line Batches

Lines are grouped into batches for efficient rendering:

```rust
use praxis_graphics::{LineBatch, Line};
use praxis_math::Vec3;

let mut batch = LineBatch::new();

// Add individual line segments
batch.add(
    Vec3::new(0.0, 0.0, 0.0), // start
    Vec3::new(1.0, 1.0, 1.0), // end
    Vec3::new(1.0, 0.0, 0.0), // color (red)
);

// Or add Line objects
let line = Line::new(
    Vec3::new(0.0, 0.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0),
    Vec3::new(0.0, 1.0, 0.0), // green
);
batch.add_line(line);

// Add multiple lines at once
batch.add_lines(vec![
    Line::new(Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0)),
    Line::new(Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 1.0, 0.0)),
    Line::new(Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 1.0)),
]);
```

### 3. Render Lines

Update camera matrices and render within your render pass:

```rust
// Update view/projection matrices
if let Some(line_renderer) = render_context.line_renderer_mut() {
    line_renderer.update_view_projection(view, proj, camera_pos)?;
    
    // Render is done by calling render() with a command buffer
    // (See integration examples below)
}
```

## Visual Feedback Utilities

The `visual_feedback` module provides helper functions for common visualization patterns:

### Grid Floor

```rust
use praxis_graphics::{create_grid, GridConfig};
use praxis_math::Vec3;

let config = GridConfig {
    size: 20.0,                         // 20x20 world units
    divisions: 20,                      // 20 divisions per axis
    line_color: Vec3::new(0.3, 0.3, 0.3), // gray lines
    axis_color: Vec3::new(0.5, 0.5, 0.5), // brighter axis lines
    height: 0.0,                        // at y=0
};

let grid_batch = create_grid(&config);
```

### Axis Indicators

```rust
use praxis_graphics::{create_axis_indicator, AxisIndicatorConfig};
use praxis_math::Vec3;

let config = AxisIndicatorConfig {
    length: 1.0,
    position: Vec3::ZERO,
    show_labels: false,
};

let axis_batch = create_axis_indicator(&config);
// Creates X (red), Y (green), Z (blue) axes
```

### Bounding Boxes

```rust
use praxis_graphics::create_bounding_box;
use praxis_math::Vec3;

let bbox_batch = create_bounding_box(
    Vec3::new(0.0, 1.0, 0.0), // center
    Vec3::new(0.5, 0.5, 0.5), // half-extents
    Vec3::new(1.0, 1.0, 0.0), // yellow
);
```

### Selection Outlines

```rust
use praxis_graphics::create_selection_outline;
use praxis_math::{Mat4, Vec3};

let transform = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
let outline_batch = create_selection_outline(
    &transform,
    Vec3::new(0.6, 0.6, 0.6), // size
    Vec3::new(1.0, 0.5, 0.0), // orange
);
```

### Gizmo Lines

```rust
use praxis_graphics::create_gizmo_lines;
use praxis_math::Vec3;

// Convert (start, end, color) tuples to LineBatch
let gizmo_lines = vec![
    (Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0)),
    (Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 1.0, 0.0)),
    (Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 1.0)),
];

let batch = create_gizmo_lines(gizmo_lines);
```

## Integration Patterns

### Viewport Editor Integration

For editor viewports with gizmos:

```rust
use praxis_graphics::{LineBatch, create_gizmo_lines};

pub struct ViewportPanel {
    // ... other fields
    
    pub fn get_gizmo_lines(&self, world: &World) -> Option<LineBatch> {
        if !self.show_gizmos {
            return None;
        }

        let gizmo_system = world.get_resource::<GizmoSystem>()?;
        let gizmo = gizmo_system.active_gizmo()?;
        
        let lines = gizmo.get_lines(
            gizmo_system.mode(),
            gizmo_system.space()
        );
        
        Some(create_gizmo_lines(lines))
    }
}
```

### Debug Visualization

For runtime debug drawing:

```rust
use praxis_graphics::LineBatch;
use praxis_math::Vec3;

pub struct DebugDraw {
    lines: LineBatch,
}

impl DebugDraw {
    pub fn new() -> Self {
        Self {
            lines: LineBatch::new(),
        }
    }
    
    pub fn draw_ray(&mut self, origin: Vec3, direction: Vec3, length: f32, color: Vec3) {
        self.lines.add(origin, origin + direction * length, color);
    }
    
    pub fn draw_circle(&mut self, center: Vec3, radius: f32, color: Vec3, segments: u32) {
        for i in 0..segments {
            let angle1 = (i as f32 / segments as f32) * std::f32::consts::TAU;
            let angle2 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
            
            let start = center + Vec3::new(
                radius * angle1.cos(),
                0.0,
                radius * angle1.sin(),
            );
            let end = center + Vec3::new(
                radius * angle2.cos(),
                0.0,
                radius * angle2.sin(),
            );
            
            self.lines.add(start, end, color);
        }
    }
    
    pub fn clear(&mut self) {
        self.lines.clear();
    }
    
    pub fn batch(&self) -> &LineBatch {
        &self.lines
    }
}
```

## Render Pass Integration

### With Depth Testing

Lines respect the depth buffer for proper z-ordering:

```rust
// 1. Create render pass with depth attachment
let render_pass = render_context.create_render_pass_with_depth(
    vulkano::format::Format::R8G8B8A8_UNORM
)?;

// 2. Create depth buffer image
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

// 3. Create framebuffer with color and depth attachments
let framebuffer = Framebuffer::new(
    render_pass.clone(),
    FramebufferCreateInfo {
        attachments: vec![color_view, depth_view],
        ..Default::default()
    },
)?;

// 4. Render solid geometry first, then lines
// Both will write to the depth buffer
```

### Render Order

For correct depth testing, render in this order:

1. **Clear** color and depth buffers
2. **Render opaque meshes** (writes depth)
3. **Render lines** (tests and writes depth)
4. **Render transparent objects** (tests depth, may not write)

```rust
// Pseudo-code for render order
command_buffer_builder
    .begin_render_pass(...)
    .bind_pipeline_graphics(mesh_pipeline)
    // ... render meshes ...
    .bind_pipeline_graphics(line_pipeline)
    // ... render lines ...
    .end_render_pass();
```

## Performance Considerations

### Batching

Group lines into batches to minimize draw calls:

```rust
// Good: One batch with 1000 lines = 1 draw call
let mut batch = LineBatch::with_capacity(1000);
for line in lines {
    batch.add_line(line);
}
renderer.render(&batch)?;

// Bad: 1000 batches = 1000 draw calls
for line in lines {
    let mut batch = LineBatch::new();
    batch.add_line(line);
    renderer.render(&batch)?;
}
```

### Dynamic Updates

Lines are dynamic - vertex buffers are created each frame:

```rust
// Create line batch per frame
let mut batch = LineBatch::new();

// Update lines based on current state
for entity in entities {
    if needs_debug_visualization(entity) {
        batch.add_lines(get_debug_lines(entity));
    }
}

// Render updated batch
renderer.render(&batch)?;
```

### Memory Management

- Vertex buffers use `HOST_VISIBLE` memory for fast CPU updates
- Buffers are allocated per-frame and automatically cleaned up
- Pre-allocate batches with `with_capacity()` when size is known

```rust
// Pre-allocate for known line count
let expected_lines = 100;
let mut batch = LineBatch::with_capacity(expected_lines);
```

## Common Patterns

### Conditional Visualization

```rust
pub struct DebugRenderer {
    show_physics: bool,
    show_navmesh: bool,
    show_bounds: bool,
}

impl DebugRenderer {
    pub fn get_lines(&self, world: &World) -> LineBatch {
        let mut batch = LineBatch::new();
        
        if self.show_physics {
            batch.add_lines(get_physics_debug_lines(world));
        }
        
        if self.show_navmesh {
            batch.add_lines(get_navmesh_debug_lines(world));
        }
        
        if self.show_bounds {
            batch.add_lines(get_bounds_debug_lines(world));
        }
        
        batch
    }
}
```

### Color Coding

Use colors to convey information:

```rust
const COLOR_SELECTED: Vec3 = Vec3::new(1.0, 0.5, 0.0);  // Orange
const COLOR_HOVERED: Vec3 = Vec3::new(0.5, 0.5, 1.0);   // Light blue
const COLOR_ERROR: Vec3 = Vec3::new(1.0, 0.0, 0.0);     // Red
const COLOR_SUCCESS: Vec3 = Vec3::new(0.0, 1.0, 0.0);   // Green
const COLOR_WARNING: Vec3 = Vec3::new(1.0, 1.0, 0.0);   // Yellow

fn visualize_entity_state(entity: Entity, state: EntityState) -> LineBatch {
    let color = match state {
        EntityState::Selected => COLOR_SELECTED,
        EntityState::Hovered => COLOR_HOVERED,
        EntityState::Error => COLOR_ERROR,
        _ => Vec3::new(0.5, 0.5, 0.5), // Gray
    };
    
    create_bounding_box(entity.position(), entity.bounds(), color)
}
```

## Troubleshooting

### Lines Not Visible

1. **Check depth testing**: Ensure lines aren't behind geometry
2. **Verify camera matrices**: Lines use view/projection from `update_view_projection()`
3. **Check line positions**: Lines might be outside view frustum
4. **Verify colors**: Very dark colors (near black) may not be visible

### Lines Rendering Behind Geometry

1. **Ensure depth buffer exists**: Use `create_render_pass_with_depth()`
2. **Check render order**: Render meshes before lines
3. **Verify depth state**: Line renderer has depth testing enabled by default

### Performance Issues

1. **Reduce line count**: Consider LOD for distant lines
2. **Batch efficiently**: Combine lines into fewer batches
3. **Pre-allocate batches**: Use `with_capacity()` to avoid reallocations
4. **Cull invisible lines**: Don't add lines outside view frustum

## Examples

See these examples for complete implementations:

- `examples/line_rendering_demo.rs` - Comprehensive line rendering showcase
- `examples/editor_demo.rs` - Editor integration with gizmos
- `examples/selection_demo.rs` - Selection outlines and highlighting

## API Reference

### Core Types

- `LineVertex` - Vertex with position and color
- `Line` - Single line segment
- `LineBatch` - Collection of lines
- `LineRenderer` - GPU renderer

### Visual Feedback

- `create_grid()` - Grid floor
- `create_axis_indicator()` - XYZ axes
- `create_bounding_box()` - Wireframe box
- `create_selection_outline()` - Transformed outline
- `create_gizmo_lines()` - Convert tuples to batch
- `batch_to_lines()` - Convert batch to individual lines

### Configuration

- `GridConfig` - Grid appearance
- `AxisIndicatorConfig` - Axis indicator settings

## See Also

- [Visual Feedback Guide](visual-feedback.md)
- [Editor Tools Guide](../editor/tools.md)
- [Debugging and Profiling](debugging.md)
