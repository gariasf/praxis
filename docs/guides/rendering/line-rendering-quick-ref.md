# Line Rendering Quick Reference

Quick reference for using line primitive rendering in Praxis engine.

## Setup (One-Time)

```rust
// Create render pass with depth buffer
let render_pass = render_context.create_render_pass_with_depth(
    vulkano::format::Format::R8G8B8A8_UNORM
)?;

// Initialize line renderer
render_context.initialize_line_renderer(render_pass, [800, 600])?;
```

## Basic Line Rendering

```rust
use praxis_graphics::{LineBatch, Line};
use praxis_math::Vec3;

// Create batch
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
    line_renderer.render(&mut command_buffer, &batch)?;
}
```

## Visual Feedback Patterns

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

### Axis Indicator
```rust
use praxis_graphics::{create_axis_indicator, AxisIndicatorConfig};

let axes = create_axis_indicator(&AxisIndicatorConfig {
    length: 1.0,
    position: Vec3::ZERO,
    show_labels: false,
});
// X = red, Y = green, Z = blue
```

### Bounding Box
```rust
use praxis_graphics::create_bounding_box;

let bbox = create_bounding_box(
    Vec3::new(0.0, 1.0, 0.0),  // center
    Vec3::new(0.5, 0.5, 0.5),  // half-extents
    Vec3::new(1.0, 1.0, 0.0),  // yellow
);
```

### Selection Outline
```rust
use praxis_graphics::create_selection_outline;

let outline = create_selection_outline(
    &entity_transform,           // Mat4
    Vec3::splat(0.6),           // size
    Vec3::new(1.0, 0.5, 0.0),   // orange
);
```

### Gizmo Lines
```rust
use praxis_graphics::create_gizmo_lines;

let lines = vec![
    (Vec3::ZERO, Vec3::X, Vec3::new(1.0, 0.0, 0.0)),
    (Vec3::ZERO, Vec3::Y, Vec3::new(0.0, 1.0, 0.0)),
    (Vec3::ZERO, Vec3::Z, Vec3::new(0.0, 0.0, 1.0)),
];
let batch = create_gizmo_lines(lines);
```

## Editor Integration

```rust
// In ViewportPanel
pub fn get_gizmo_lines(&self, world: &World) -> Option<LineBatch> {
    if !self.show_gizmos {
        return None;
    }
    
    let gizmo_system = world.get_resource::<GizmoSystem>()?;
    let gizmo = gizmo_system.active_gizmo()?;
    
    let lines = gizmo.get_lines(gizmo_system.mode(), gizmo_system.space());
    Some(create_gizmo_lines(lines))
}
```

## Common Patterns

### Debug Draw System
```rust
pub struct DebugDraw {
    lines: LineBatch,
}

impl DebugDraw {
    pub fn draw_ray(&mut self, origin: Vec3, dir: Vec3, len: f32, color: Vec3) {
        self.lines.add(origin, origin + dir * len, color);
    }
    
    pub fn draw_circle(&mut self, center: Vec3, radius: f32, color: Vec3) {
        const SEGMENTS: u32 = 32;
        for i in 0..SEGMENTS {
            let a1 = (i as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            let a2 = ((i + 1) as f32 / SEGMENTS as f32) * std::f32::consts::TAU;
            
            let p1 = center + Vec3::new(radius * a1.cos(), 0.0, radius * a1.sin());
            let p2 = center + Vec3::new(radius * a2.cos(), 0.0, radius * a2.sin());
            
            self.lines.add(p1, p2, color);
        }
    }
    
    pub fn clear(&mut self) {
        self.lines.clear();
    }
}
```

### Color Constants
```rust
const RED: Vec3 = Vec3::new(1.0, 0.0, 0.0);
const GREEN: Vec3 = Vec3::new(0.0, 1.0, 0.0);
const BLUE: Vec3 = Vec3::new(0.0, 0.0, 1.0);
const YELLOW: Vec3 = Vec3::new(1.0, 1.0, 0.0);
const CYAN: Vec3 = Vec3::new(0.0, 1.0, 1.0);
const MAGENTA: Vec3 = Vec3::new(1.0, 0.0, 1.0);
const WHITE: Vec3 = Vec3::new(1.0, 1.0, 1.0);
const GRAY: Vec3 = Vec3::new(0.5, 0.5, 0.5);
const ORANGE: Vec3 = Vec3::new(1.0, 0.5, 0.0);
```

## Performance Tips

### ✅ Do
- Batch lines together
- Pre-allocate with `with_capacity()`
- Clear and reuse batches
- Render after opaque geometry

### ❌ Don't
- Create batch per line
- Create new batch every frame
- Render before depth buffer is written

## Render Order

```rust
// Correct order
command_buffer
    .begin_render_pass(...)
    // 1. Render opaque meshes (write depth)
    .bind_pipeline(mesh_pipeline)
    // ... mesh rendering ...
    // 2. Render lines (test depth)
    .bind_pipeline(line_pipeline)
    // ... line rendering ...
    .end_render_pass();
```

## See Also

- Full Guide: `docs/guides/line-rendering.md`
- Example: `examples/line_rendering_demo.rs`
- Module Docs: `crates/praxis_graphics/src/line_renderer.rs`
