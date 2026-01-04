# Visual Feedback Systems

This document describes the visual feedback systems implemented for the Praxis engine editor.

## Overview

The visual feedback systems provide essential visual cues for spatial reference, entity selection, and transform manipulation in the 3D viewport. These systems are built on top of an efficient line rendering pipeline.

## Components

### 1. Line Rendering System

**Location**: `crates/praxis_graphics/src/line_renderer.rs`

The line renderer provides efficient batched rendering of colored lines in 3D space.

#### Features:
- **Batched rendering**: Multiple lines rendered in a single draw call
- **Colored vertices**: Each line can have its own color
- **Depth testing**: Lines respect depth for proper 3D visualization
- **Efficient pipeline**: Dedicated shaders for line rendering

#### Usage Example:
```rust
use praxis_graphics::{LineRenderer, LineBatch, Line};
use praxis_math::Vec3;

// Create line renderer
let mut line_renderer = LineRenderer::new(
    device.clone(),
    render_pass.clone(),
    memory_allocator.clone(),
    [1920, 1080],
)?;

// Create a batch of lines
let mut batch = LineBatch::new();
batch.add(Vec3::ZERO, Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0));
batch.add(Vec3::ZERO, Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, 0.0));
batch.add(Vec3::ZERO, Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, 1.0));

// Update camera matrices
line_renderer.update_view_projection(view, proj, camera_pos)?;

// Render in command buffer
line_renderer.render(&mut builder, &batch)?;
```

### 2. Visual Feedback Helpers

**Location**: `crates/praxis_graphics/src/visual_feedback.rs`

Helper functions for creating common visual feedback elements.

#### Grid Floor Display

Creates a grid on the ground plane for spatial reference.

```rust
use praxis_graphics::{create_grid, GridConfig};

let config = GridConfig {
    size: 20.0,           // 20 units across
    divisions: 20,        // 20x20 grid
    line_color: Vec3::new(0.3, 0.3, 0.3),
    axis_color: Vec3::new(0.5, 0.5, 0.5),
    height: 0.0,          // Y=0
};

let grid_batch = create_grid(&config);
```

**Features**:
- Configurable size and division count
- Center lines (X=0, Z=0) highlighted with different color
- Adjustable height for placement at any Y coordinate

#### Axis Indicator

Creates colored axis lines showing X, Y, Z directions.

```rust
use praxis_graphics::{create_axis_indicator, AxisIndicatorConfig};

let config = AxisIndicatorConfig {
    length: 1.0,
    position: Vec3::ZERO,
    show_labels: false,
};

let axis_batch = create_axis_indicator(&config);
```

**Color Convention**:
- X axis: Red (1.0, 0.0, 0.0)
- Y axis: Green (0.0, 1.0, 0.0)
- Z axis: Blue (0.0, 0.0, 1.0)

#### Entity Selection Highlighting

Creates wireframe outlines for selected entities.

**Bounding Box (World-Aligned)**:
```rust
use praxis_graphics::create_bounding_box;

let bbox_batch = create_bounding_box(
    entity_position,
    Vec3::new(0.5, 0.5, 0.5),  // half-extents
    Vec3::new(1.0, 0.5, 0.0),  // orange color
);
```

**Selection Outline (Transform-Aligned)**:
```rust
use praxis_graphics::create_selection_outline;

let outline_batch = create_selection_outline(
    &entity_transform_matrix,
    Vec3::ONE,                  // size
    Vec3::new(1.0, 0.5, 0.0),  // orange color
);
```

The selection outline respects the entity's rotation and scale, creating a properly aligned wireframe box.

#### Gizmo Rendering

Integration with the editor's gizmo system for transform manipulation.

```rust
use praxis_editor::GizmoSystem;
use praxis_graphics::create_gizmo_lines;

let gizmo_system = /* get from resources */;
if let Some(gizmo) = gizmo_system.active_gizmo() {
    let lines = gizmo.get_lines(gizmo_system.mode(), gizmo_system.space());
    let gizmo_batch = create_gizmo_lines(lines);
    // Render gizmo_batch
}
```

### 3. Shaders

**Location**: `crates/praxis_graphics/src/shaders/`

- `line.vert`: Vertex shader for line rendering
- `line.frag`: Fragment shader for line rendering

The shaders are simple and efficient:
- Transform vertices using view-projection matrix
- Pass through vertex colors for per-line coloring
- No lighting calculations (pure color)

## Integration with Editor

The visual feedback systems integrate seamlessly with the editor:

1. **Grid Floor**: Always visible in the viewport for spatial reference
2. **Axis Indicator**: Can be placed at origin or any location
3. **Selection Highlighting**: Automatically rendered for selected entities
4. **Gizmo Rendering**: Displayed when entities are selected and manipulated

### Rendering Order

For proper visual appearance, render in this order:

1. Regular 3D scene (opaque objects)
2. Grid floor (drawn on ground plane)
3. Axis indicator (small, at origin)
4. Selection outlines (around selected objects)
5. Gizmo lines (on top of everything)

### Performance Considerations

- **Batching**: All lines are batched into a single draw call per batch
- **Static Data**: Grid and axis indicator can be pre-computed and cached
- **Dynamic Data**: Selection outlines and gizmos are rebuilt each frame (small cost)
- **Memory**: Vertex buffers are host-visible for efficient CPU-to-GPU transfer

## API Reference

### LineRenderer

```rust
pub struct LineRenderer { /* ... */ }

impl LineRenderer {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        memory_allocator: Arc<StandardMemoryAllocator>,
        extent: [u32; 2],
    ) -> Result<Self>;

    pub fn update_view_projection(
        &mut self,
        view: Mat4,
        proj: Mat4,
        camera_position: Vec3,
    ) -> Result<()>;

    pub fn render(
        &self,
        builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        batch: &LineBatch,
    ) -> Result<()>;
}
```

### LineBatch

```rust
pub struct LineBatch { /* ... */ }

impl LineBatch {
    pub fn new() -> Self;
    pub fn with_capacity(capacity: usize) -> Self;
    pub fn add_line(&mut self, line: Line);
    pub fn add(&mut self, start: Vec3, end: Vec3, color: Vec3);
    pub fn add_lines(&mut self, lines: impl IntoIterator<Item = Line>);
    pub fn clear(&mut self);
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

### Visual Feedback Functions

```rust
// Grid floor
pub fn create_grid(config: &GridConfig) -> LineBatch;

// Axis indicator
pub fn create_axis_indicator(config: &AxisIndicatorConfig) -> LineBatch;

// Bounding box (world-aligned)
pub fn create_bounding_box(center: Vec3, size: Vec3, color: Vec3) -> LineBatch;

// Selection outline (transform-aligned)
pub fn create_selection_outline(
    transform: &Mat4,
    size: Vec3,
    color: Vec3,
) -> LineBatch;

// Gizmo lines (from editor gizmo system)
pub fn create_gizmo_lines<I>(lines: I) -> LineBatch
where
    I: IntoIterator<Item = (Vec3, Vec3, Vec3)>;
```

## Example: Complete Viewport Rendering

```rust
use praxis_graphics::*;
use praxis_math::{Mat4, Vec3};

fn render_viewport(
    line_renderer: &mut LineRenderer,
    builder: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
    view: Mat4,
    proj: Mat4,
    camera_pos: Vec3,
) -> Result<()> {
    // Update camera matrices
    line_renderer.update_view_projection(view, proj, camera_pos)?;
    
    // 1. Grid floor
    let grid_config = GridConfig::default();
    let grid_batch = create_grid(&grid_config);
    line_renderer.render(builder, &grid_batch)?;
    
    // 2. Axis indicator at origin
    let axis_config = AxisIndicatorConfig::default();
    let axis_batch = create_axis_indicator(&axis_config);
    line_renderer.render(builder, &axis_batch)?;
    
    // 3. Selection highlights for selected entities
    for selected_entity in selected_entities {
        let transform = get_entity_transform(selected_entity);
        let outline_batch = create_selection_outline(
            &transform,
            Vec3::ONE,
            Vec3::new(1.0, 0.5, 0.0), // Orange
        );
        line_renderer.render(builder, &outline_batch)?;
    }
    
    // 4. Gizmo (if active)
    if let Some(gizmo) = gizmo_system.active_gizmo() {
        let lines = gizmo.get_lines(gizmo_system.mode(), gizmo_system.space());
        let gizmo_batch = create_gizmo_lines(lines);
        line_renderer.render(builder, &gizmo_batch)?;
    }
    
    Ok(())
}
```

## Future Enhancements

Possible future additions to the visual feedback systems:

1. **Text Rendering**: Add labels to axis indicator, grid coordinates, etc.
2. **Thickness Control**: Variable line thickness for emphasis
3. **Dashed Lines**: Support for dashed or dotted line patterns
4. **Anti-aliasing**: MSAA or FXAA for smoother line appearance
5. **Frustum Visualization**: Render camera frustums for debugging
6. **Bone Visualization**: Skeletal hierarchy rendering for animation debugging
7. **Physics Debug Draw**: Collider shapes, contact points, velocities
8. **Profiler Overlays**: Performance graphs and statistics

## Testing

All visual feedback functions include comprehensive unit tests:

```bash
cargo test --package praxis_graphics line_renderer
cargo test --package praxis_graphics visual_feedback
```

Tests verify:
- Correct vertex count for each primitive type
- Proper batching behavior
- Configuration defaults
- Transform application
