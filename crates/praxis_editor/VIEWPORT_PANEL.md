# Viewport Panel

The `ViewportPanel` provides a 3D scene viewport for the Praxis editor with full camera controls and grid rendering.

## Features

- **Offscreen Rendering**: Renders the 3D scene to an offscreen framebuffer that can be displayed in egui
- **Camera Controls**: 
  - Orbit camera with mouse right-click + drag
  - Pan camera with mouse middle-click + drag
  - Zoom with mouse scroll wheel
  - Move camera target with WASD/QE keys
- **Grid Floor**: Renders a grid floor with axis indicators (X=red, Z=blue)
- **Viewport-Specific Camera**: Each viewport has its own camera entity
- **Event Handling**: Mouse and keyboard events are properly handled within viewport bounds

## Usage

### Basic Setup

```rust
use praxis_editor::ViewportPanel;
use praxis_graphics::RenderContext;

// Create the viewport panel
let mut viewport = ViewportPanel::new();

// Initialize with render context
viewport.initialize(&mut render_context)?;
```

### Integration with Editor

```rust
use praxis_editor::{EditorState, ViewportPanel};

// Add viewport to editor state
let mut editor = EditorState::new();

// The viewport can be added as a tab in the dock system
// (See EditorState documentation for dock integration)
```

### Camera Control

The viewport uses an orbit camera system with the following controls:

- **Right-Click + Drag**: Orbit around the target point
- **Middle-Click + Drag**: Pan the camera target
- **Mouse Wheel**: Zoom in/out (adjusts camera distance)
- **WASD**: Move the camera target horizontally
- **Q/E**: Move the camera target vertically

### Programmatic Camera Control

```rust
// Set camera distance
viewport.set_camera_distance(15.0);

// Set camera target
viewport.set_camera_target(Vec3::new(0.0, 0.0, 0.0));

// Reset camera to default position
viewport.reset_camera();

// Get current camera transform
let camera_transform = viewport.compute_camera_transform();
```

### Grid Rendering

The viewport includes a grid floor for spatial reference:

```rust
// Toggle grid visibility
viewport.set_show_grid(true);

// Check if grid is shown
let is_visible = viewport.show_grid();
```

The grid features:
- 50x50 unit grid by default
- Center lines highlighted in brighter color
- Every 5th line slightly emphasized
- X-axis in red, Z-axis in blue
- Configurable size and divisions

## Architecture

### Offscreen Rendering

The viewport renders to an offscreen `RenderTarget` which is a Vulkan framebuffer. This allows:
- Multiple viewports in the same editor
- Independent rendering from the main swapchain
- Flexible viewport sizing and positioning

### Camera System

The camera uses spherical coordinates for orbit control:
- **Distance**: Radius from target point
- **Pitch**: Vertical angle (up/down)
- **Yaw**: Horizontal angle (left/right)
- **Target**: Center point of orbit

The camera transform is computed as:
```rust
position = target + Vec3(
    distance * cos(pitch) * sin(yaw),
    distance * sin(pitch),
    distance * cos(pitch) * cos(yaw)
)
```

### Event Handling

Mouse and keyboard events are filtered to only respond when interacting within the viewport bounds:
- `handle_camera_input()`: Processes mouse events within viewport rect
- `handle_keyboard_input()`: Processes keyboard events (typically called when viewport is focused)

### Rendering Pipeline

```text
┌─────────────────────┐
│  ViewportPanel      │
│  - Camera state     │
│  - Grid renderer    │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Offscreen Target   │
│  - Framebuffer      │
│  - Render pass      │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Vulkan Image       │
│  (800x600 RGBA)     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  egui::Image        │
│  (Display in UI)    │
└─────────────────────┘
```

## API Reference

### Creation and Initialization

- `ViewportPanel::new()` - Creates a new viewport with default settings
- `initialize(&mut RenderContext)` - Initializes offscreen rendering and grid

### Camera Control

- `compute_camera_transform() -> Transform` - Gets current camera transform
- `camera_distance() -> f32` - Gets camera distance from target
- `set_camera_distance(f32)` - Sets camera distance (clamped 1.0 to 1000.0)
- `camera_target() -> Vec3` - Gets camera target position
- `set_camera_target(Vec3)` - Sets camera target position
- `reset_camera()` - Resets camera to default position

### Grid Control

- `set_show_grid(bool)` - Shows or hides the grid
- `show_grid() -> bool` - Returns whether grid is visible

### Entity Management

- `set_camera_entity(Entity)` - Associates an ECS camera entity with this viewport
- `camera_entity() -> Option<Entity>` - Gets the associated camera entity

### Rendering

- `render_viewport(&mut RenderContext)` - Renders the viewport contents
- `resize_viewport(&RenderContext, [u32; 2])` - Resizes the render target
- `render_target() -> Option<&RenderTarget>` - Gets the offscreen render target

### Input Handling

- `handle_keyboard_input(&InputState, f32)` - Processes keyboard input for camera movement

## Implementation Notes

### Current Limitations

1. **Texture Display**: The rendered texture is not yet displayed in egui (requires egui-vulkano integration)
2. **Scene Rendering**: Scene entities are not yet queried and rendered (requires ECS integration)
3. **Lighting**: Scene lighting is not yet included in viewport rendering
4. **Multiple Viewports**: While architecturally supported, simultaneous multiple viewports not fully tested

### Future Enhancements

- [ ] Display rendered texture in egui
- [ ] Query and render scene entities
- [ ] Support scene lighting in viewport
- [ ] Add gizmos for object manipulation
- [ ] Add viewport-specific rendering settings
- [ ] Add camera projection controls (FOV, near/far planes)
- [ ] Add viewport statistics overlay (FPS, triangle count, etc.)
- [ ] Add viewport screenshot functionality
- [ ] Add multiple camera preset positions
- [ ] Add grid customization UI

## Examples

### Basic Viewport

```rust
use praxis_editor::ViewportPanel;

let mut viewport = ViewportPanel::new();
// Use in editor...
```

### Custom Grid Size

```rust
use praxis_editor::ViewportPanel;

let mut viewport = ViewportPanel::new();
viewport.initialize(&mut render_context)?;

// Grid is automatically created with default size (50 units)
// To customize, modify viewport_grid::GridRenderer parameters
```

### Viewport with Custom Camera

```rust
use praxis_editor::ViewportPanel;
use praxis_math::Vec3;

let mut viewport = ViewportPanel::new();
viewport.initialize(&mut render_context)?;

// Position camera looking at a specific point
viewport.set_camera_target(Vec3::new(5.0, 2.0, 3.0));
viewport.set_camera_distance(20.0);
```

## Testing

The viewport panel includes unit tests for core functionality:

```bash
cargo test -p praxis_editor
```

Tests cover:
- Grid mesh generation
- Camera transform computation
- Grid renderer initialization
- Viewport creation and configuration

## Performance Considerations

- **Offscreen Rendering**: Each viewport requires GPU memory for the render target
- **Grid Complexity**: Grid with 50x50 divisions creates ~200 lines (400 vertices)
- **Frame Rate**: Viewport rendering should be throttled to avoid unnecessary GPU work
- **Memory**: Each 800x600 RGBA texture requires ~1.8 MB GPU memory

## See Also

- [EditorPanel Trait](panels/mod.rs) - Base trait for editor panels
- [RenderTarget](../praxis_graphics/src/post_process/render_target.rs) - Offscreen rendering
- [Camera Components](../praxis_ecs/src/components.rs) - ECS camera components
- [Transform System](../praxis_scene/) - Transform hierarchy system
